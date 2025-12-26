CREATE TABLE power_outages (
    id SERIAL PRIMARY KEY,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP,
    duration_minutes INTEGER
);

CREATE INDEX idx_power_outages_started_at ON power_outages(started_at DESC);
