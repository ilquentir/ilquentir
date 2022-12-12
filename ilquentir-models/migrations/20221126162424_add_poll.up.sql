-- Add up migration script here
CREATE TABLE polls (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    tg_id VARCHAR(50) UNIQUE,
    chat_tg_id BIGINT NOT NULL,
    kind VARCHAR(20) NOT NULL,
    publication_date TIMESTAMP WITH TIME ZONE NOT NULL,
    published BOOLEAN NOT NULL DEFAULT false,

-- meta fields
    date_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_users
        FOREIGN KEY(chat_tg_id) REFERENCES users(tg_id)
);