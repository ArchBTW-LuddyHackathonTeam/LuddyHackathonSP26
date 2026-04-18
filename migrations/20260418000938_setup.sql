-- Add migration script here
CREATE TABLE score
(
    id          UUID PRIMARY KEY   DEFAULT gen_random_uuid(),
    leaderboard UUID      NOT NULL references leaderboard (id),
    uploader    TEXT      NOT NULL DEFAULT 'Anonymous',
    created_at  TIMESTAMP NOT NULL DEFAULT now()
);
