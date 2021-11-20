# book_reviews

## Application

>Goal: Map New York Time's best sellers to Goodreads Reviews.

Top authors / top books,


https://developer.nytimes.com/docs/books-product/1/overview  Connor
Backup: https://www.kaggle.com/dhruvildave/new-york-times-best-sellers

https://www.goodreads.com/api Simon - Karaj
Backup:
https://www.kaggle.com/jealousleopard/goodreadsbooks

## TODO

- split tables
- scrape missing values amazon / goodreads
- scrape google api
- merge tables
- drop useless columns, aggregate duplicates (normlaize values)



https://developers.google.com/books![image](https://user-images.githubusercontent.com/92374756/139418242-3cd0e984-9885-472f-a1ff-ee047ba6f323.png)

## Idea:

Take **Combined Print and E-Book Fiction** and **Combined Print and E-Book Nonfiction** and calculate a score for each book according to how long it was on the list and at what rank it had.

## Run

```sh
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python main.py
```
