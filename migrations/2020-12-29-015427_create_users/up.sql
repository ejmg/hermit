-- Your SQL goes here

CREATE TABLE users (
       id SERIAL PRIMARY KEY,
       name VARCHAR(16),
       username VARCHAR(12) NOT NULL UNIQUE,
       pw_hash TEXT NOT NULL,
       bio VARCHAR(120),
       location VARCHAR(80),
       email VARCHAR(100) NOT NULL UNIQUE,
       date_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
