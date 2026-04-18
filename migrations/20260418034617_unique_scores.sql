-- Add migration script here
ALTER TABLE score DROP COLUMN id;
ALTER TABLE score ADD CONSTRAINT score_uploader_pkey PRIMARY KEY (uploader);
