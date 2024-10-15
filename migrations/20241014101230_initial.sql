CREATE TABLE IF NOT EXISTS artists (
    id       INTEGER PRIMARY KEY,
    username TEXT    UNIQUE      NOT NULL,
    twitter  TEXT
);

CREATE TABLE IF NOT EXISTS images (
    id       INTEGER PRIMARY KEY,
    path     TEXT    UNIQUE      NOT NULL,
    artist   INTEGER REFERENCES artists (id),
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

