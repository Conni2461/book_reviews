from sqlalchemy import create_engine, Column, String, Integer, exc
from sqlalchemy.orm import Session, declarative_base

import requests

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

    # TODO
    # goodreads kaggle csv file
    # goodreads_userscore

    # TODO
    # amazon score

    def __repr__(self) -> str:
        return f"Book(isbn={self.isbn!r}, title={self.title!r}, author={self.author!r}, score={self.score!r})"

Base.metadata.create_all(engine)
getAll()
