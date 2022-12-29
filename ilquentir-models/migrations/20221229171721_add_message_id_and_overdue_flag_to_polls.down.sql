-- Add down migration script here
DROP INDEX actual_polls;

ALTER TABLE polls DROP COLUMN overdue;
ALTER TABLE polls DROP COLUMN tg_message_id;
