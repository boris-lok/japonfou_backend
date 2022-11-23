-- Add migration script here

CREATE TABLE users
(
    id            uuid NOT NULL,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    PRIMARY KEY (id)
);
