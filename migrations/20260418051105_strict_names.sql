-- Add migration script here
ALTER TABLE score ALTER COLUMN uploader TYPE VARCHAR(32);
ALTER TABLE score_history ALTER COLUMN uploader TYPE VARCHAR(32);
