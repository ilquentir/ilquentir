-- Add up migration script here
CREATE TABLE poll_settings (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    poll_kind VARCHAR(20) NOT NULL,
    user_tg_id BIGINT NOT NULL,
    send_at_utc TIME,

    UNIQUE (poll_kind, user_tg_id),

    CONSTRAINT fk_users
        FOREIGN KEY(user_tg_id) REFERENCES users(tg_id)
)
