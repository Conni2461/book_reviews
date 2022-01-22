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
After pulling repository on Ubuntu 20.04: 

1. Install Rust for your machine. https://www.rust-lang.org/tools/install  
2. Install C compiler if not present (possibly with `sudo apt install build-essential` or `sudo apt install gcc-multilib`)
3. Install Sqplite with `sudo apt install libsqlite3-dev`if not present
4. In repository folder: `cargo run` 
5. should run on `localhost:8080` in browser
