-- Your SQL goes here
CREATE TABLE users (
  username TEXT PRIMARY KEY,
  password TEXT NOT NULL
);

CREATE TABLE tokens (
  id UUID PRIMARY KEY,
  username TEXT REFERENCES users(username) NOT NULL
);
