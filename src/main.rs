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

#[derive(Deserialize, Debug, Clone)]
pub struct ExtendedSearch {
  s1v: Option<String>,
  s1c: Option<String>,
  s1: Option<String>,

  s2s: Option<String>,
  s2v: Option<String>,
  s2c: Option<String>,
  s2: Option<String>,

  s3s: Option<String>,
  s3v: Option<String>,
  s3c: Option<String>,
  s3: Option<String>,
}

impl ExtendedSearch {
  fn has_value(&self) -> bool {
    self.s1v.is_some()
      || self.s1c.is_some()
      || self.s1.is_some()
      || self.s2s.is_some()
      || self.s2v.is_some()
      || self.s2c.is_some()
      || self.s2.is_some()
      || self.s3s.is_some()
      || self.s3v.is_some()
      || self.s3c.is_some()
      || self.s3.is_some()
  }
}

fn format_books(books: &mut Vec<Book>) {
  for b in books.iter_mut() {
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
}

fn get_books(
  pool: web::Data<Pool>,
  where_query: &str,
) -> Result<(Vec<Book>, i64), diesel::result::Error> {
  let conn = pool
    .get()
    .expect("Couldn't get database connection from pool");

  info!("{}", where_query);
  let mut results = sql_query(format!(
    "
    WITH ranked_books AS (
      SELECT *, RANK () OVER (
        ORDER BY score DESC
      ) rank FROM merged_books_aggregated_updated_finished
    )
    SELECT * FROM ranked_books
    {}
    ORDER BY score DESC LIMIT 10
  ",
    where_query
  ))
  .load::<Book>(&conn)?;
  format_books(&mut results);

  let count = sql_query(format!(
    "
      SELECT COUNT(*) as count FROM merged_books_aggregated_updated_finished
      {}
      ORDER BY score DESC LIMIT 10
    ",
    where_query
  ))
  .load::<Count>(&conn)?
  .pop()
  .expect("No rows")
  .count;

  Ok((results, count))
}

fn get_books_simple(
  pool: web::Data<Pool>,
  query: Search,
) -> Result<(Vec<Book>, i64), diesel::result::Error> {
  match &query.search {
    Some(s) => get_books(
      pool,
      &format!("WHERE title LIKE '%{}%' OR author LIKE '%{}%'", s, s),
    ),
    None => get_books(pool, ""),
  }
}

async fn index(
  db: web::Data<Pool>,
  hb: web::Data<Handlebars<'_>>,
  query: web::Query<Search>,
) -> Result<HttpResponse, Error> {
  let query = query.into_inner();
  let search = query.clone();
  Ok(
    web::block(move || get_books_simple(db, search))
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

fn get_books_extended(
  pool: web::Data<Pool>,
  query: ExtendedSearch,
) -> Result<(Vec<Book>, i64), diesel::result::Error> {
  // TODO check for wrong input
  let mut where_query = String::from("WHERE ");
  if query.s1v.is_some() {
    where_query = where_query
      + &query.s1v.unwrap()
      + " "
      + &query.s1c.unwrap()
      + " '"
      + &query.s1.unwrap()
      + "'"
  }
  if query.s2s.is_some() && query.s2s.as_ref().unwrap() != "" {
    where_query = where_query
      + " "
      + &query.s2s.unwrap()
      + " "
      + &query.s2v.unwrap()
      + " "
      + &query.s2c.unwrap()
      + " '"
      + &query.s2.unwrap()
      + "'";
  }
  if query.s3s.is_some() && query.s3s.as_ref().unwrap() != "" {
    where_query = where_query
      + " "
      + &query.s3s.unwrap()
      + " "
      + &query.s3v.unwrap()
      + " "
      + &query.s3c.unwrap()
      + " '"
      + &query.s3.unwrap()
      + "'";
  }
  get_books(pool, &where_query)
}

async fn search(
  db: web::Data<Pool>,
  hb: web::Data<Handlebars<'_>>,
  query: web::Query<ExtendedSearch>,
) -> Result<HttpResponse, Error> {
  let data = json!({
    "logic": [
      { "value": "", "text": "--Please choose an option--" },
      { "value": "AND", "text": "AND" },
      { "value": "OR", "text": "OR" },
      { "value": "NOT", "text": "NOT" },
    ],
    "option": [
      { "value": "", "text": "" },
      { "value": "title", "text": "Title" },
      { "value": "author", "text": "Author" },
      { "value": "publisher", "text": "Publisher" },
      { "value": "date", "text": "Date" },
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

  let query = query.into_inner();
  if !query.has_value() {
    let body = hb.render("search", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
  } else {
    info!("{:?}", query);
    let search = query.clone();
    Ok(
      web::block(move || get_books_extended(db, search))
        .await?
        .map(|(books, count)| {
          let current = std::cmp::min(count, 10);
          info!("Showing {} out of {}", current, count);
          let data = json!({
            "count": { "curr": current, "total": count },
            "books": books,
            "query": ""
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
