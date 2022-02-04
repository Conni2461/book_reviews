#[macro_use]
extern crate diesel;
extern crate dotenv;

use actix_files::Files;
use actix_web::{web, App, Error, HttpResponse, HttpServer, ResponseError};
use handlebars::{
  Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext,
  RenderError,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

use diesel::r2d2::{self, ConnectionManager};
use diesel::{prelude::*, sql_query};
use dotenv::dotenv;

use log::{error, info};

mod db;
use db::models::{Book, Count};

const COUNT_PER_PAGE: i64 = 20;

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
  page: Option<i64>,
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

  page: Option<i64>,
}

#[derive(Serialize, Debug)]
struct CountInformation {
  total: i64,
  sindex: i64,
  eindex: i64,
  page: i64,
}

impl CountInformation {
  pub fn new(sindex: i64, count: i64, page: Option<i64>) -> Self {
    let mut eindex = sindex + COUNT_PER_PAGE - 1;
    eindex = std::cmp::min(eindex, count);

    Self {
      total: count,
      sindex: sindex + 1,
      eindex,
      page: page.unwrap_or(1),
    }
  }
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
  page: Option<i64>,
) -> Result<(Vec<Book>, i64, i64), diesel::result::Error> {
  let conn = pool
    .get()
    .expect("Couldn't get database connection from pool");

  let start_limit = match page {
    Some(p) => (p - 1) * COUNT_PER_PAGE,
    None => 0,
  };

  let mut results = sql_query(format!(
    "
    WITH ranked_books AS (
      SELECT *, RANK () OVER (
        ORDER BY score DESC
      ) rank FROM merged_books_aggregated_updated_finished
    )
    SELECT * FROM ranked_books
    {}
    ORDER BY score DESC LIMIT {}, {}
  ",
    where_query, start_limit, COUNT_PER_PAGE
  ))
  .load::<Book>(&conn)?;
  format_books(&mut results);

  let count = sql_query(format!(
    "
      SELECT COUNT(*) as count FROM merged_books_aggregated_updated_finished
      {}
      ORDER BY score DESC
    ",
    where_query
  ))
  .load::<Count>(&conn)?
  .pop()
  .expect("No rows")
  .count;

  Ok((results, start_limit, count))
}

fn get_books_simple(
  pool: web::Data<Pool>,
  query: Search,
) -> Result<(Vec<Book>, i64, i64), diesel::result::Error> {
  match &query.search {
    Some(s) => get_books(
      pool,
      &format!("WHERE title LIKE '%{}%' OR author LIKE '%{}%'", s, s),
      query.page,
    ),
    None => get_books(pool, "", query.page),
  }
}

async fn index(
  db: web::Data<Pool>,
  hb: web::Data<Handlebars<'_>>,
  query: web::Query<Search>,
  req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
  let query = query.into_inner();
  let search = query.clone();
  Ok(
    web::block(move || get_books_simple(db, search))
      .await?
      .map(|(books, sindex, count)| {
        let data = json!({
          "count": CountInformation::new(sindex, count, query.page),
          "books": books,
          "query": query.search,
          "uri": req.uri().path_and_query().unwrap().as_str()
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

fn as_query(
  con: Option<String>,
  lhs: Option<String>,
  c: Option<String>,
  rhs: Option<String>,
) -> String {
  let mut res = String::new();
  if let Some(s) = con {
    res = res + " " + s.as_str();
  }

  let comparision = c.unwrap();
  if comparision == "=" {
    res = res + &lhs.unwrap() + " LIKE '%" + &rhs.unwrap() + "%'";
  } else {
    res = res + &lhs.unwrap() + " " + &comparision + " '" + &rhs.unwrap() + "'";
  }

  res
}

fn get_books_extended(
  pool: web::Data<Pool>,
  query: ExtendedSearch,
) -> Result<(Vec<Book>, i64, i64), diesel::result::Error> {
  // TODO check for wrong input
  let mut where_query = String::from("WHERE ");
  if query.s1v.is_some() {
    where_query = where_query + &as_query(None, query.s1v, query.s1c, query.s1);
  }
  if query.s2s.is_some() && query.s2s.as_ref().unwrap() != "" {
    where_query =
      where_query + &as_query(query.s2s, query.s2v, query.s2c, query.s2);
  }
  if query.s3s.is_some() && query.s3s.as_ref().unwrap() != "" {
    where_query =
      where_query + &as_query(query.s3s, query.s3v, query.s3c, query.s3);
  }
  get_books(pool, &where_query, query.page)
}

async fn search(
  db: web::Data<Pool>,
  hb: web::Data<Handlebars<'_>>,
  query: web::Query<ExtendedSearch>,
  req: actix_web::HttpRequest,
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
    let search = query.clone();
    Ok(
      web::block(move || get_books_extended(db, search))
        .await?
        .map(|(books, sindex, count)| {
          let data = json!({
            "count": CountInformation::new(sindex, count, query.page),
            "books": books,
            "query": "",
            "uri": req.uri().path_and_query().unwrap().as_str()
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

async fn stats(hb: web::Data<Handlebars<'_>>) -> Result<HttpResponse, Error> {
  let data = json!({});
  let body = hb.render("stats", &data).unwrap();
  Ok(HttpResponse::Ok().body(body))
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

  handlebars.register_helper(
    "paging",
    Box::new(
      |h: &Helper,
       _r: &Handlebars,
       _: &Context,
       _rc: &mut RenderContext,
       out: &mut dyn Output|
       -> HelperResult {
        let p0 = h.param(0);
        let p1 = h.param(1);
        let p2 = h.param(2);
        if p0.is_none() || p1.is_none() || p2.is_none() {
          return Err(RenderError::new("param not found"));
        }

        let mut uri = String::from(p0.unwrap().value().as_str().unwrap());
        let page = p1.unwrap().value().as_i64().unwrap();
        let total = p2.unwrap().value().as_i64().unwrap();
        let last_page = (total as f64 / COUNT_PER_PAGE as f64).ceil() as i64;

        // remove page=... from the uri because it needs to be replaced
        if let Some(mut f) = uri.find("page=") {
          f -= 1; // also remove & || ?
          loop {
            uri.remove(f);
            if uri.len() == f || uri.chars().nth(f).unwrap() == '&' {
              break;
            }
          }
        }
        if uri.contains("/?") || uri.contains("/search?") {
          uri.push_str("&page=");
        } else {
          uri.push_str("?page=");
        }

        // previous 1, (current) m, ..., n, next
        // write beginning
        out.write(
          format!(
            r###"
              <div class="col justify-content-end">
                <ul class="pagination justify-content-end">
                  <li class="page-item {2}">
                    <a class="page-link" href="{0}{1}">Previous</a>
                  </li>
                  <li class="page-item {3}"><a class="page-link" href="{0}1">1</a></li>
            "###, uri, page - 1, if page == 1 { "disabled" } else { "" },
                                 if page == 1 { "active" } else { "" }
          ).as_str()
        )?;

        // and we have more than 1 page
        if last_page > 4 {
          if page < last_page / 2 {
            out.write(
              format!(
                r###"
                  <li class="page-item {3}"><a class="page-link" href="{0}{1}">{1}</a></li>
                  <li class="page-item disabled"><a class="page-link">...</a></li>
                  <li class="page-item"><a class="page-link" href="{0}{2}">{2}</a></li>
                "###, uri, if page == 1 { page + 1 } else { page }, last_page,
                           if page != 1 { "active" } else { "" }
              ).as_str()
            )?;
          } else {
            out.write(
              format!(
                r###"
                  <li class="page-item disabled"><a class="page-link">...</a></li>
                  <li class="page-item {3}"><a class="page-link" href="{0}{1}">{1}</a></li>
                  <li class="page-item {4}"><a class="page-link" href="{0}{2}">{2}</a></li>
                "###, uri, if page == last_page { page - 1 } else { page }, last_page,
                           if page != last_page { "active" } else { "" },
                           if page == last_page { "active" } else { "" },
              ).as_str()
            )?;
          }
        } else {
          let mut i = 2;
          while i <= last_page {
            out.write(
              format!(
                r###"
                  <li class="page-item {2}"><a class="page-link" href="{0}{1}">{1}</a></li>
                "###, uri, i, if page == i { "active" } else { "" }
              ).as_str()
            )?;
            i += 1;
          }
        }

        out.write(
          format!(
            r###"
              <li class="page-item {1}">
                <a class="page-link" href="{0}{2}">Next</a>
              </li>
          "###,
            uri, if page == last_page { "disabled" } else { "" }, page + 1
          )
          .as_str(),
        )?;

        // write end
        out.write(
            r###"
            </ul>
          </div>
            "###)?;
        Ok(())
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
    .register_template_file("stats", "./static/stats.hbs")
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
      .route("/stats", web::get().to(stats))
  })
  .bind("127.0.0.1:8080")?
  .run()
  .await
}
