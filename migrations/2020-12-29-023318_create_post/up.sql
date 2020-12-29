-- Your SQL goes here

CREATE TABLE post (
  id SERIAL PRIMARY KEY,
  author_id INT,
  text TEXT NOT NULL,
  date_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
);
