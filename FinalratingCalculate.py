
import numpy as np
import pandas as pd

df = pd.read_csv(r'C:\Users\karaj\OneDrive\Desktop\merged_books_updated.csv')
X1 = df[["goodreads_rating","amazon_rating","google_rating"]]

X = X1.iloc[:,:].values

#Imputing Missing values with median
from sklearn.impute import SimpleImputer
imputer = SimpleImputer(missing_values=np.nan, strategy='median')
imputer.fit(X[:, :])
X[:, :] = imputer.transform(X[:, :])

#Feature scaling with MinMaxScaler
from sklearn import preprocessing
min_max_scaler = preprocessing.MinMaxScaler()
X_minmax = min_max_scaler.fit_transform(X)

#Calculating final rating as the sum of all the three ratings
Y = X_minmax[:,0]+X_minmax[:,1]+X_minmax[:,2]
Y =pd.DataFrame(Y)
Y.columns =["FinalRating"]
