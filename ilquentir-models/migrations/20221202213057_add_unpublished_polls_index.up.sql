-- Add up migration script here
CREATE INDEX unpublished_polls ON polls ((1))
WHERE NOT published;
