CREATE TABLE IF NOT EXISTS branch (
    name TEXT NOT NULL PRIMARY KEY,
    ticket TEXT,
    data BLOB,
    created TEXT NOT NULL
);