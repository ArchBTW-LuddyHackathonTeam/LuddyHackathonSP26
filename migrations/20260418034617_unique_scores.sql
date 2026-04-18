-- Add migration script here
ALTER TABLE score DROP COLUMN id;
ALTER TABLE score ADD CONSTRAINT score_name_pkey PRIMARY KEY (name);
