-- Users table.
CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,

    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,

    admin BOOLEAN NOT NULL DEFAULT FALSE,

    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER update_users_modified
AFTER UPDATE ON users
BEGIN
    UPDATE users SET modified = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Sessions table.
CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Books table.
CREATE TABLE books (
    id INTEGER PRIMARY KEY NOT NULL,

    title TEXT NOT NULL,
    author TEXT NOT NULL,
    cover BLOB,

    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER update_book_modified
AFTER UPDATE ON books
BEGIN
    UPDATE books SET modified = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Default admin account (password: admin).
INSERT INTO users (name, password, admin)
VALUES ('admin', '$argon2i$v=19$m=4096,t=3,p=1$c2FsdEl0V2l0aFNhbHQ$xTGvQNICqetaNA0Wu1GwFmYhQjAreRcjBz6ornhaFXA', TRUE);