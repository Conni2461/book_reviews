# book_reviews

## Application

>Goal: Map New York Time's best sellers to Goodreads Reviews.


## TODO

- split tables
- scrape missing values amazon / goodreads
- scrape google api
- merge tables
- drop useless columns, aggregate duplicates (normalize values)




## Idea:

Take **Combined Print and E-Book Fiction** and **Combined Print and E-Book Nonfiction** and calculate a score for each book according to how long it was on the list and at what rank it had.

## Run

```sh
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python main.py
```
