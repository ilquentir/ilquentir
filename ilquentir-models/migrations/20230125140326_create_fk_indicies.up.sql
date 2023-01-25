-- Add up migration script here
CREATE INDEX poll_chat_fk ON polls (chat_tg_id);
CREATE INDEX poll_answers_poll_fk ON poll_answers (poll_tg_id);
