-- Add migration script here
CREATE TABLE server (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    ip_address TEXT NOT NULL UNIQUE,
    plan INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    author_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(name, ip_address)
);