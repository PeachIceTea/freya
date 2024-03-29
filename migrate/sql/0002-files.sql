-- Files table.
CREATE TABLE files (
    id INTEGER PRIMARY KEY NOT NULL,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,

    path TEXT NOT NULL,

    name TEXT NOT NULL,
    position INTEGER NOT NULL,
    duration REAL NOT NULL,

    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);