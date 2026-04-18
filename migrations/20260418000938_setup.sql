-- Add migration script here
CREATE TABLE leaderboard
(
    id           UUID PRIMARY KEY   DEFAULT gen_random_uuid(),
    display_name TEXT      NOT NULL,
    private      BOOL      NOT NULL DEFAULT FALSE,
    secret       TEXT      NOT NULL,
    created_at   TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE score
(
    id          UUID PRIMARY KEY   DEFAULT gen_random_uuid(),
    leaderboard UUID      NOT NULL references leaderboard (id),
    uploader    TEXT      NOT NULL DEFAULT 'Anonymous',
    created_at  TIMESTAMP NOT NULL DEFAULT now()
);
