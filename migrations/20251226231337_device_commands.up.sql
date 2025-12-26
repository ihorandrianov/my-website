CREATE TABLE device_commands (
    id SERIAL PRIMARY KEY,
    command_type TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
