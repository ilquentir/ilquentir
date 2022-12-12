-- Add up migration script here
CREATE TABLE poll_answers (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    poll_tg_id VARCHAR(50) NOT NULL,
    selected_value INT NOT NULL,

-- meta fields
    date_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_poll
        FOREIGN KEY(poll_tg_id) REFERENCES polls(tg_id)
);