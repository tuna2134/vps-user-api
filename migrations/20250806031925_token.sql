-- Add migration script here
CREATE TABLE session_token(
    id SERIAL NOT NULL PRIMARY KEY,
    nonce TEXT NOT NULL,
    user_id SERIAL NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)