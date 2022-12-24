-- Add up migration script here
CREATE TABLE poll_custom_options (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    poll_kind VARCHAR(20) NOT NULL,
    user_tg_id BIGINT NOT NULL,
    option_text VARCHAR NOT NULL,

    UNIQUE (poll_kind, user_tg_id, option_text),

    CONSTRAINT fk_users
        FOREIGN KEY(user_tg_id) REFERENCES users(tg_id)
)
