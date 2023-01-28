-- Add up migration script here
CREATE TABLE diary_entries (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    user_tg_id BIGINT NOT NULL,
    text TEXT NOT NULL,

-- meta fields
    date_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_user
        FOREIGN KEY(user_tg_id) REFERENCES users(tg_id)
);

CREATE INDEX diary_entries_user_fk ON diary_entries (user_tg_id);
