-- Add up migration script here
CREATE TABLE users (
    tg_id BIGINT PRIMARY KEY NOT NULL,
    date_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    active BOOLEAN NOT NULL
);