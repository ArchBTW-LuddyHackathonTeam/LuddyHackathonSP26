-- Add migration script here
CREATE TABLE score
(
    id          UUID PRIMARY KEY          DEFAULT gen_random_uuid(),
    uploader    TEXT             NOT NULL DEFAULT 'Anonymous',
    created_at  TIMESTAMP        NOT NULL DEFAULT now(),
    value       DOUBLE PRECISION NOT NULL
);
