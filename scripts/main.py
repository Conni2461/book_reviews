from sqlalchemy import create_engine, Column, Date, String, Integer, Float, exc, and_
from sqlalchemy.orm import Session, declarative_base

import requests
from datetime import datetime

from selectorlib import Extractor
import requests
from time import sleep

# Create an Extractor by reading from the YAML file
def scrape(url, file):
    e = Extractor.from_yaml_file(file)
    headers = {
        "authority": "www.amazon.com",
        "pragma": "no-cache",
        "cache-control": "no-cache",
        "dnt": "1",
        "upgrade-insecure-requests": "1",
        "user-agent": "Mozilla/5.0 (X11; CrOS x86_64 8172.45.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.64 Safari/537.36",
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9",
        "sec-fetch-site": "none",
        "sec-fetch-mode": "navigate",
        "sec-fetch-dest": "document",
        "accept-language": "en-GB,en-US;q=0.9,en;q=0.8",
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
    response = requests.get(
        f"https://api.nytimes.com/svc/books/v3/lists/{date}/{name_list}.json?api-key={NYT_KEY}"
    )
    json = response.json()
    if "results" in json:
        results = json["results"]
        return results["previous_published_date"], results["books"]
    return None, None


def scrapeAmazon():
    books = (
        session.query(Amazon)
        .filter(and_(Amazon.rating == None, Amazon.url != None))
        .all()
    )
    for book in books:
        amazon = scrape(book.url, "definitions/amazon.yml")
        print(amazon, book.url)
        if amazon != None:
            rating = amazon["rating"]
            if rating != None:
                book.rating = rating.split()[0]
            review_count = amazon["review_count"]
            if review_count != None:
                book.review_count = review_count.split()[0].replace(",", "")
            session.commit()


def scrapeGoodreads():
    books = session.query(Goodreads).filter(Goodreads.rating == None).all()
    for book in books:
        good = scrape(
            f"https://www.goodreads.com/search?q={book.isbn}",
            "definitions/goodreads.yml",
        )
        print(good, book.isbn)
        if good != None:
            book.rating = good["rating"]
            book.rating_count = good["ratingCount"]
            book.review_count = good["reviewCount"]
            session.commit()


def fetchGoogleBooks():
    books = session.query(GoogleBooks).all()
    for book in books:
        r = requests.get(
            f"https://www.googleapis.com/books/v1/volumes?q=isbn:{book.isbn}"
        )
        if not r.ok:
            continue
        data = r.json()
        if "items" not in data:
            continue
        found = False
        for item in data["items"]:
            if found:
                break
            if "volumeInfo" not in item:
                continue
            info = item["volumeInfo"]
            if "title" in info:
                book.title = info["title"]
            if "subtitle" in info:
                book.subtitle = info["subtitle"]
            if "authors" in info:
                book.authors = ";".join(info["authors"])
            if "description" in info:
                book.description = info["description"]
            if "categories" in info:
                book.categories = ";".join(info["categories"])
            if "averageRating" in info:
                book.average_rating = info["averageRating"]
            if "ratingsCount" in info:
                book.rating_count = info["ratingsCount"]
            if "maturityRating" in info:
                book.maturity_rating = info["maturityRating"]
            if "language" in info:
                book.language = info["language"]
            if "pageCount" in info:
                book.page_count = info["pageCount"]
            if "publisher" in info:
                book.publisher = info["publisher"]
            if "publishedDate" in info:
                book.published_date = info["publishedDate"]
            session.commit()
            print(book.title)
            found = True


def getAllNYT():
    cursor = "current"
    while True:
        sleep(6)
        if cursor == "" or cursor == None:
            break
        print(f"getting list: {cursor}")
        lhs, books = getList(cursor, "Combined Print and E-Book Fiction")
        if lhs == None or books == None:
            continue
        cursor = lhs
        for book in books:
            if book["rank"] > 15:
                continue
            score = 15 - book["rank"] + 1
            try:
                amazon_url = book["amazon_product_url"]
                b = NYT(
                    isbn=book["primary_isbn13"],
                    title=book["title"],
                    author=book["author"],
                    score=score,
                    publisher=book["publisher"],
                    book_image=book["book_image"],
                    amazon_product_url=amazon_url,
                )
                session.add(b)
                session.add(Amazon(isbn=book["primary_isbn13"], url=amazon_url))
                session.add(Goodreads(isbn=book["primary_isbn13"]))
                session.add(GoogleBooks(isbn=book["primary_isbn13"]))
                session.commit()

            except exc.IntegrityError:
                session.rollback()
                b = session.query(NYT).get(book["primary_isbn13"])
                b.score += score
                session.commit()


def exportToCsv():
    books = session.query(NYT).all()
    with open("books.csv", "w") as f:
        for book in books:
            f.write(f"{book.isbn}\n")


def normalizeScore():
    import numpy as np
    from sklearn.impute import SimpleImputer
    from sklearn import preprocessing

    imputer = SimpleImputer(missing_values=np.nan, strategy="median")
    books = session.query(MergedBooksUpdated).all()
    x1 = np.empty((0, 4))
    for book in books:
        x1 = np.append(
            x1,
            np.array(
                [
                    [
                        book.isbn,
                        book.goodreads_rating,
                        book.amazon_rating,
                        book.google_rating,
                    ]
                ]
            ),
            axis=0,
        )

    imputer.fit(x1[:, 1:])
    x1[:, 1:] = imputer.transform(x1[:, 1:])
    min_max_scaler = preprocessing.MinMaxScaler()
    x_minmax = min_max_scaler.fit_transform(x1[:, 1:])
    x1 = np.insert(x1, 4, x_minmax[:, 0] + x_minmax[:, 1] + x_minmax[:, 2], axis=1)
    for x in x1:
        book = (
            session.query(MergedBooksUpdated)
            .filter(MergedBooksUpdated.isbn == x[0])
            .first()
        )
        book.score = x[4]
    session.commit()


class NYT(Base):
    __tablename__ = "NYT"

    isbn = Column(String, primary_key=True, nullable=False)
    title = Column(String, nullable=False)
    author = Column(String, nullable=False)
    score = Column(Integer, nullable=False)
    publisher = Column(String, nullable=False)
    book_image = Column(String, nullable=True)
    amazon_product_url = Column(String, nullable=True)

    def __repr__(self) -> str:
        return (
            f"NYT(isbn={self.isbn!r}, "
            f"title={self.title!r}, "
            f"score={self.author!r}, "
            f"publisher={self.score!r}, "
            f"book_image={self.book_image}, "
            f"amazon_product_url={self.amazon_product_url!r})"
        )


class GoogleBooks(Base):
    __tablename__ = "GoogleBooks"

    isbn = Column(String, primary_key=True, nullable=False)
    title = Column(String, nullable=True)
    subtitle = Column(String, nullable=True)
    authors = Column(String, nullable=True)
    description = Column(String, nullable=True)
    categories = Column(String, nullable=True)
    average_rating = Column(Float, nullable=True)
    rating_count = Column(Integer, nullable=True)
    maturity_rating = Column(String, nullable=True)
    language = Column(Integer, nullable=True)
    page_count = Column(Integer, nullable=True)
    publisher = Column(String, nullable=True)
    published_date = Column(String, nullable=True)

    def __repr__(self) -> str:
        return (
            f"GoogleBooks(isbn={self.isbn!r}, "
            f"title={self.title!r}, "
            f"subtitle={self.subtitle!r}, "
            f"authors={self.authors!r}, "
            f"description={self.description!r}, "
            f"categories={self.categories!r}, "
            f"average_rating={self.average_rating!r}, "
            f"rating_count={self.rating_count!r}, "
            f"maturity_rating={self.maturity_rating!r}, "
            f"language={self.language!r}, "
            f"page_count={self.page_count!r}, "
            f"publisher={self.publisher!r}, "
            f"published_date={self.published_date!r})"
        )


class Amazon(Base):
    __tablename__ = "Amazon"

    isbn = Column(String, primary_key=True, nullable=False)
    title = Column(String, nullable=True)
    subtitle = Column(String, nullable=True)
    author = Column(String, nullable=True)
    description = Column(String, nullable=True)
    price = Column(Float, nullable=True)
    rating = Column(Float, nullable=True)
    review_count = Column(Integer, nullable=True)
    page_count = Column(Integer, nullable=True)
    language = Column(String, nullable=True)
    publisher = Column(String, nullable=True)
    published_date = Column(String, nullable=True)
    url = Column(String, nullable=True)

    def __repr__(self) -> str:
        return (
            f"Amazon(isbn={self.isbn!r}, "
            f"title={self.title!r}, "
            f"subtitle={self.subtitle!r}, "
            f"author={self.author!r}, "
            f"description={self.description!r}, "
            f"price={self.price!r}, "
            f"rating={self.rating!r}, "
            f"review_count={self.review_count!r}, "
            f"page_count={self.page_count!r}, "
            f"language={self.language!r}, "
            f"publisher={self.publisher!r}, "
            f"published_date={self.published_date!r}, "
            f"url={self.url!r})"
        )


class Goodreads(Base):
    __tablename__ = "Goodreads"

    isbn = Column(String, primary_key=True, nullable=False)
    rating = Column(Float, nullable=True)
    rating_count = Column(Integer, nullable=True)
    review_count = Column(Integer, nullable=True)

    def __repr__(self) -> str:
        return (
            f"Goodreads(isbn={self.isbn!r}, "
            f"rating={self.rating!r}, "
            f"rating_count={self.rating_count!r}, "
            f"review_count={self.review_count!r})"
        )


class MergedBooksUpdated(Base):
    __tablename__ = "merged_books_aggregated_updated_finished"

    isbn = Column(String, primary_key=True)
    title = Column(String, nullable=False)
    author = Column(String, nullable=False)
    publisher = Column(String)
    published_date = Column(String)
    description = Column(String)
    maturity_rating = Column(Integer)
    book_type = Column(String)
    price = Column(Float)
    page_count = Column(Integer)
    score_nyt = Column(Integer)
    goodreads_rating = Column(Float)
    goodreads_count = Column(Integer)
    amazon_rating = Column(Float)
    amazon_count = Column(Integer)
    google_rating = Column(Float)
    google_count = Column(Integer)
    book_image = Column(String)
    amazon_link = Column(String)
    score = Column(Float)


Base.metadata.create_all(engine)
# getAllNYT()
# exportToCsv()
# scrapeAmazon()
# scrapeGoodreads()
# fetchGoogleBooks()
normalizeScore()
