import requests
from bs4 import BeautifulSoup
from time import sleep
from random import randint
import pandas as pd
import urllib.request, json

# First Download the .csv formatted file of Books.sqlite and pass it as input in the read csv.
# Give the link for the ISBN 
df = pd.read_csv(r'C:\Users\karaj\OneDrive\Desktop\Books.csv')
headers ={"Accept-Language": "en-US,en;q=0.5"}

book_data=[]
authors=[]
averageRating=[]
canonicalVolumeLink = []
infoLink = []
language=[]
maturityRating=[]
pageCount =[]
publishedDate = []
publisher = []
subtitle = []
title = []
ISBN_10 = []
ISBN_13 = []
description =[]
categories =[]
ratingsCount =[]

"ENter more lists which you want in the data"

for page in range(int(df.shape[0])): 
    #page = requests.get("https://www.googleapis.com/books/v1/volumes?q={"+str(df["isbn"][page])+"}")
    with urllib.request.urlopen("https://www.googleapis.com/books/v1/volumes?q={"+str(df["isbn"][page])+"}") as url:
        data = json.loads(url.read().decode())
        try:    
            authors.append(data['items'][0]['volumeInfo']['authors'][0])
        except:
            authors.append("NA")
        try:
            averageRating.append(data['items'][0]['volumeInfo']['averageRating'])
        except:    
            averageRating.append("NA")
        try:
            canonicalVolumeLink.append(data['items'][0]['volumeInfo']['canonicalVolumeLink'])
        except:
            canonicalVolumeLink.append("NA")
        try:
            infoLink.append(data['items'][0]['volumeInfo']['infoLink'])
        except:
            infoLink.append("NA")
        try:
            language.append(data['items'][0]['volumeInfo']['language'])
        except:
            language.append("NA")
        try:
            maturityRating.append(data['items'][0]['volumeInfo']['maturityRating'])
        except:
            maturityRating.append("NA")                             
        try:
            pageCount.append(data['items'][0]['volumeInfo']['pageCount'])
        except:
            pageCount.append("NA")
        try:
            publishedDate.append(data['items'][0]['volumeInfo']['publishedDate'])
        except:
            publishedDate.append("NA")             
        try:
            publisher.append(data['items'][0]['volumeInfo']['publisher'])
        except:
            publisher.append("NA")
        try:
            subtitle.append(data['items'][0]['volumeInfo']['subtitle'])
        except:
            subtitle.append("NA")                         
        try:
            title.append(data['items'][0]['volumeInfo']['title'])
        except:
            title.append("NA")            
        try:
            ISBN_10.append(data['items'][0]['volumeInfo']['industryIdentifiers'][0]['identifier'])
        except:
            ISBN_10.append("NA")              
        try:
            ISBN_13.append(data['items'][0]['volumeInfo']['industryIdentifiers'][1]['identifier'])
        except:
            ISBN_13.append("NA")
        try:
            description.append(data['items'][0]['volumeInfo']['description'])
        except:
            description.append("NA")
        try:
            categories.append(data['items'][0]['volumeInfo']['categories'])
        except:
            categories.append("NA")            
        try:
            ratingsCount.append(data['items'][0]['volumeInfo']['ratingsCount'])
        except:
            ratingsCount.append("NA")            

        
            
    #soup = BeautifulSoup(page.content, 'html.parser') 
    #book_data = soup.findAll("Inspect and add the class and div") 
        sleep(randint(2,8))     
   # for store in book_data:
   #     name = store.h3.a.text
   #     book_name.append(name)
   
data_list = pd.DataFrame({ "authors": authors, "averageRating" : averageRating, "canonicalVolumeLink": canonicalVolumeLink,"infoLink": infoLink, "language": language, "maturityRating" : maturityRating, "pageCount": pageCount, "publishedDate": publishedDate, "publisher": publisher, "subtitle": subtitle , "title": title, "ISBN_10": ISBN_10, "ISBN_13": ISBN_13, "description": description, "categories": categories, "ratingsCount": ratingsCount})        
data_list.to_csv("GoogleAPIdataset.csv")