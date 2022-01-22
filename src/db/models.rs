use super::schema::merged_books_aggregated_updated_finished;

use diesel::sql_types::{BigInt, Integer};
use serde::Serialize;

#[derive(Serialize, Queryable, QueryableByName, Debug)]
#[table_name = "merged_books_aggregated_updated_finished"]
pub struct Book {
  pub isbn: String,
  pub title: Option<String>,
  pub author: Option<String>,
  pub publisher: Option<String>,
  pub published_date: Option<String>,
  pub description: Option<String>,
  pub maturity_rating: Option<i32>,
  pub page_count: Option<i32>,
  pub book_type: Option<String>,
  pub price: Option<f32>,
  pub score_nyt: Option<f32>,
  pub goodreads_rating: Option<f32>,
  pub goodreads_count: Option<i32>,
  pub amazon_rating: Option<f32>,
  pub amazon_count: Option<i32>,
  pub google_rating: Option<f32>,
  pub google_count: Option<i32>,
  pub book_image: Option<String>,
  pub amazon_link: Option<String>,
  pub score: Option<f32>,
  #[sql_type = "Integer"]
  pub rank: i32,
}

#[derive(QueryableByName)]
pub struct Count {
  #[sql_type = "BigInt"]
  pub count: i64,
}
