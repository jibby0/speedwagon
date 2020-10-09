-- Your SQL goes here
CREATE TABLE users (
  username TEXT PRIMARY KEY,
  password TEXT NOT NULL
);

CREATE TABLE tokens (
  id UUID PRIMARY KEY,
  username TEXT REFERENCES users(username) NOT NULL,
  expires TIMESTAMP NOT NULL
);

CREATE TABLE sources (
  id UUID PRIMARY KEY,
  title TEXT NOT NULL,
  source_data JSON NOT NULL,
  post_filter TEXT NOT NULL,
  last_post TIMESTAMP NOT NULL,
  last_successful_fetch TIMESTAMP NOT NULL,
  fetch_errors TEXT[] NOT NULL,
  creator TEXT REFERENCES users(username) NOT NULL,
  public BOOLEAN NOT NULL
);

CREATE TABLE articles (
  id UUID PRIMARY KEY,
  title TEXT,
  published TIMESTAMP,
  source_info JSON NOT NULL,
  summary TEXT,
  content JSON NOT NULL,
  rights TEXT,
  links JSON NOT NULL,
  authors JSON NOT NULL,
  categories JSON NOT NULL,
  comments_url TEXT,
  extensions JSON NOT NULL,
  source UUID REFERENCES sources(id) NOT NULL
);

CREATE TABLE tags (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  owner TEXT REFERENCES users(username) NOT NULL
);

CREATE TABLE tagged_sources (
  id UUID PRIMARY KEY,
  tag UUID REFERENCES tags(id) NOT NULL,
  source UUID REFERENCES sources(id) NOT NULL
);
