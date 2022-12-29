-- Add up migration script here
-- add overdue field
ALTER TABLE polls ADD COLUMN overdue BOOLEAN NOT NULL DEFAULT False;

CREATE INDEX actual_polls ON polls ((1))
WHERE NOT overdue;

ALTER TABLE polls ADD COLUMN tg_message_id INTEGER;
