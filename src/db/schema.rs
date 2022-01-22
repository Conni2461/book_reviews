table! {
    merged_books_aggregated_updated_finished (isbn) {
        isbn -> Text,
        title -> Nullable<Text>,
        author -> Nullable<Text>,
        publisher -> Nullable<Text>,
        published_date -> Nullable<Text>,
        description -> Nullable<Text>,
        maturity_rating -> Nullable<Integer>,
        page_count -> Nullable<Integer>,
        book_type -> Nullable<Text>,
        price -> Nullable<Float>,
        score_nyt -> Nullable<Float>,
        goodreads_rating -> Nullable<Float>,
        goodreads_count -> Nullable<Integer>,
        amazon_rating -> Nullable<Float>,
        amazon_count -> Nullable<Integer>,
        google_rating -> Nullable<Float>,
        google_count -> Nullable<Integer>,
        book_image -> Nullable<Text>,
        amazon_link -> Nullable<Text>,
        score -> Nullable<Float>,
    }
}
