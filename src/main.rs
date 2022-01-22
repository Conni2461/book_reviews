#[macro_use]
extern crate diesel;
extern crate dotenv;

use actix_files::Files;
use actix_web::{web, App, Error, HttpResponse, HttpServer, ResponseError};
use handlebars::{
  Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext,
  RenderError,
};
use serde::Deserialize;
use serde_json::json;
use std::env;

use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types::Text;
use diesel::{prelude::*, sql_query};
use dotenv::dotenv;

use log::{error, info};

mod db;
use db::models::{Book, Count};

#[derive(Debug)]
pub struct MyError(String); // <-- needs debug and display

impl std::fmt::Display for MyError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "A validation error occured on the input.")
  }
}
impl ResponseError for MyError {}

#[derive(Deserialize, Debug, Clone)]
pub struct Search {
  search: Option<String>,
}

fn get_books(
  pool: web::Data<Pool>,
  query: Search,
) -> Result<(Vec<Book>, i64), diesel::result::Error> {
  let conn = pool
    .get()
    .expect("Couldn't get database connection from pool");
  let mut results = match &query.search {
    Some(s) => sql_query(
      "WITH ranked_books AS (
          SELECT *, RANK () OVER (
            ORDER BY score DESC
          ) rank FROM merged_books_aggregated_updated_finished
        )
        SELECT * FROM ranked_books
        WHERE title LIKE ?
        OR author Like ?
        ORDER BY score DESC LIMIT 10",
    )
    .bind::<Text, _>(format!("%{}%", s))
    .bind::<Text, _>(format!("%{}%", s))
    .load::<Book>(&conn)?,
    None => sql_query(
      "SELECT *, RANK () OVER (
          ORDER BY score DESC
        ) rank FROM merged_books_aggregated_updated_finished
        ORDER BY score DESC
        LIMIT 10",
    )
    .load::<Book>(&conn)?,
  };

  for b in results.iter_mut() {
    if let Some(t) = &b.title {
      b.title = Some(t.split('\n').take(1).collect());
    };

    if let Some(a) = &b.author {
      b.author = Some(a.split('\n').take(1).collect());
    };

    if let Some(p) = &b.publisher {
      b.publisher = Some(p.split('\n').take(1).collect());
    };
  }

  let count = match &query.search {
    Some(s) => sql_query(
      "SELECT COUNT(*) as count FROM merged_books_aggregated_updated_finished
      WHERE title LIKE ?
      OR author Like ?",
    )
    .bind::<Text, _>(format!("%{}%", s))
    .bind::<Text, _>(format!("%{}%", s))
    .load::<Count>(&conn)?,
    None => sql_query(
      "SELECT COUNT(*) as count FROM merged_books_aggregated_updated_finished",
    )
    .load::<Count>(&conn)?,
  }
  .pop()
  .expect("No rows")
  .count;

  Ok((results, count))
}

async fn index(
  db: web::Data<Pool>,
  hb: web::Data<Handlebars<'_>>,
  query: web::Query<Search>,
) -> Result<HttpResponse, Error> {
  let query = query.into_inner();
  let search = query.clone();
  Ok(
    web::block(move || get_books(db, search))
      .await?
      .map(|(books, count)| {
        let current = std::cmp::min(count, 10);
        info!("Showing {} out of {}", current, count);
        let data = json!({
          "count": { "curr": current, "total": count },
          "books": books,
          "query": query.search,
        });

        let body = hb.render("index", &data).unwrap();
        HttpResponse::Ok().body(body)
      })
      .map_err(|e| {
        error!("{}", e);
        MyError(String::from("oh no"))
      })?,
  )
}

async fn search(
  _db: web::Data<Pool>,
  hb: web::Data<Handlebars<'_>>,
) -> HttpResponse {
  let data = json!({
    "logic": [
      { "value": "", "text": "--Please choose an option--" },
      { "value": "AND", "text": "AND" },
      { "value": "OR", "text": "OR" },
      { "value": "NOT", "text": "NOT" },
    ],
    "option": [
      { "value": "", "text": "" },
      { "value": "Title", "text": "Title" },
      { "value": "Author", "text": "Author" },
      { "value": "Publisher", "text": "Publisher" },
      { "value": "Date", "text": "Date" },
      { "value": "ISBN", "text": "ISBN" },
      { "value": "nytscore", "text": "nytscore" },
      { "value": "score", "text": "score" },
    ],
    "compare": [
      { "value": "=", "text": "=" },
      { "value": "<", "text": "<" },
      { "value": ">", "text": ">" },
    ],
  });
  let body = hb.render("search", &data).unwrap();
  HttpResponse::Ok().body(body)
}

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  std::env::set_var("RUST_LOG", "info,actix_web=debug");
  dotenv().ok();
  env_logger::init();
  let database_url =
    env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let manager = ConnectionManager::<SqliteConnection>::new(database_url);
  let pool: Pool = r2d2::Pool::builder()
    .build(manager)
    .expect("Failed to create pool");

  let mut handlebars = Handlebars::new();

  handlebars.register_helper(
    "distanceFixed",
    Box::new(
      |h: &Helper,
       _r: &Handlebars,
       _: &Context,
       _rc: &mut RenderContext,
       out: &mut dyn Output|
       -> HelperResult {
        match h.param(0) {
          Some(p) => {
            let v = p.value();
            if v.is_f64() {
              out.write(&format!("{:.2}", v.as_f64().unwrap()))?;
            } else {
              out.write(v.render().as_ref())?;
            }

            Ok(())
          }
          None => Err(RenderError::new("param not found")),
        }
      },
    ),
  );

  handlebars
    .register_template_file("index", "./static/index.hbs")
    .unwrap();
  handlebars
    .register_template_file("search", "./static/search.hbs")
    .unwrap();
  handlebars
    .register_template_file("base", "./static/base.hbs")
    .unwrap();
  let handlebars_ref = web::Data::new(handlebars);
  let pool_ref = web::Data::new(pool);

  info!("listening on port 8080");
  HttpServer::new(move || {
    App::new()
      .app_data(pool_ref.clone())
      .app_data(handlebars_ref.clone())
      .service(Files::new("/static", "static").show_files_listing())
      .route("/", web::get().to(index))
      .route("/search", web::get().to(search))
  })
  .bind("127.0.0.1:8080")?
  .run()
  .await
}
