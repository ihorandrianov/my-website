CREATE TABLE authorized_users (
    id SERIAL PRIMARY KEY,
    telegram_user_id BIGINT NOT NULL UNIQUE,
    username TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_authorized_users_telegram_id ON authorized_users(telegram_user_id);
