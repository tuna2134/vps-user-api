-- Add migration script here
CREATE TABLE setup_script (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    script TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    author_id INTEGER NULL REFERENCES users(id) ON DELETE CASCADE
)