-- Add migration script here
CREATE TABLE score_history
(
    id          UUID PRIMARY KEY          DEFAULT gen_random_uuid(),
    uploader    TEXT             NOT NULL,
    created_at  TIMESTAMP        NOT NULL DEFAULT now(),
    value       DOUBLE PRECISION NOT NULL
);
