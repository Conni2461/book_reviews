# book_reviews

Information Integration class Project

## Application

>Goal: Map New York Time's best sellers to Goodreads Reviews.

## Idea:

Take **Combined Print and E-Book Fiction** and **Combined Print and E-Book
Nonfiction** and calculate a score for each book according to how long it was
on the list and at what rank it had.

## Run scripts

Python and jupyter notebook scripts to generate the current database can be
found under `./scripts`. You might need python packages which are specified
unter `./requirements.txt`

## Run backend/frontend
After pulling repository on Ubuntu 20.04 (other version should work, this is just what we tested):

1. Setup
  1.0. `sudo apt update && sudo apt upgrade`
  1.1. Install Build tools: `sudo apt install build-essential`
  1.2. Install rust https://www.rust-lang.org/tools/install
  1.3. Install Sqlite with `sudo apt install libsqlite3-dev`
2. In repository folder: `cargo run`
3. should run on `localhost:8080` in browser
