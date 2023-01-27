-- Add down migration script here
DROP INDEX IF EXISTS diary_entries_user_fk;
DROP TABLE diary_entries;
