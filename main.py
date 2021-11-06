from sqlalchemy import create_engine, Column, String, Integer, Float, exc
from sqlalchemy.orm import Session, declarative_base

import requests

from selectorlib import Extractor
import requests
import json
from time import sleep
# Create an Extractor by reading from the YAML file
def scrape(url, file):
    e = Extractor.from_yaml_file(file)
    headers = {
        'authority': 'www.amazon.com',
        'pragma': 'no-cache',
        'cache-control': 'no-cache',
        'dnt': '1',
        'upgrade-insecure-requests': '1',
        'user-agent': 'Mozilla/5.0 (X11; CrOS x86_64 8172.45.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.64 Safari/537.36',
        'accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9',
        'sec-fetch-site': 'none',
        'sec-fetch-mode': 'navigate',
        'sec-fetch-dest': 'document',
        'accept-language': 'en-GB,en-US;q=0.9,en;q=0.8',
    }
    # Download the page using requests
    r = requests.get(url, headers=headers)
    # Simple check to check if page was blocked (Usually 503)
    if r.status_code > 500:
        return None

    # Pass the HTML of the page and create
    return e.extract(r.text)


engine = create_engine("sqlite+pysqlite:///books.sqlite")
Base = declarative_base()
Base.metadata.create_all(engine)
session = Session(engine)

NYT_KEY = "TODO"

if NYT_KEY == "TODO":
    print("Don't forget to put in your NYT_KEY")
    exit(1)

def getList(date, name_list):
    response = requests.get(f"https://api.nytimes.com/svc/books/v3/lists/{date}/{name_list}.json?api-key={NYT_KEY}")
    json = response.json()
    if "results" in json:
        results = json["results"]
        return results["previous_published_date"], results["books"]
    return None, None

def getAll():
    cursor = "current"
    for i in range(10):
        lhs, books = getList(cursor, "hardcover-fiction")
        if lhs == None or books == None:
            i = i - 1
            continue
        cursor = lhs
        for book in books:
            score = 15 - book["rank"] + 1
            try:
                b = Book(isbn=book["primary_isbn13"], title=book["title"], author=book["author"], score=score, publisher=book["publisher"], book_image=book["book_image"], amazon_product_url=book["amazon_product_url"])
                session.add(b)
                session.commit()
                amazon = scrape(book["amazon_product_url"], "definitions/amazon.yml")
                if amazon != None:
                    rating = amazon["rating"]
                    if rating != None:
                        b.amazon_rating = rating.split()[0]
                    reviewCount = amazon["reviewCount"]
                    if reviewCount != None:
                        b.amazon_review_count = reviewCount.split()[0].replace(",", "")
                    session.commit()

                good = scrape(f"https://www.goodreads.com/search?q={book['primary_isbn13']}", "definitions/goodreads.yml")
                if good != None:
                    b.goodreads_rating = good["rating"]
                    b.goodreads_rating_count = good["ratingCount"]
                    b.goodreads_review_count = good["reviewCount"]
                    session.commit()

            except exc.IntegrityError:
                session.rollback()
                b = session.query(Book).get(book["primary_isbn13"])
                b.score += score
                session.commit()

class Book(Base):
    __tablename__ = "Books"

    # New York Times API
    isbn = Column(String, primary_key=True, nullable=False)
    title = Column(String, nullable=False)
    author = Column(String, nullable=False)
    score = Column(Integer, nullable=False)
    publisher = Column(String, nullable=False)
    book_image = Column(String, nullable=True)
    amazon_product_url = Column(String, nullable=True)

    # TODO
    # Google books api
    # descriptions
    # genre

    amazon_rating = Column(Float, nullable=True)
    amazon_review_count = Column(Integer, nullable=True)

    goodreads_rating = Column(Float, nullable=True)
    goodreads_rating_count = Column(Integer, nullable=True)
    goodreads_review_count = Column(Integer, nullable=True)

    def __repr__(self) -> str:
        return f"Book(isbn={self.isbn!r}, title={self.title!r}, author={self.author!r}, score={self.score!r})"

Base.metadata.create_all(engine)
getAll()
