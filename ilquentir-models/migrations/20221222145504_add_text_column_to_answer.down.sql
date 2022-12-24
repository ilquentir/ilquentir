-- Add down migration script here
ALTER TABLE poll_answers DROP COLUMN selected_value_text;
