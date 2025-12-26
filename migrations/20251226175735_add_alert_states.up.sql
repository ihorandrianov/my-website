CREATE TABLE alert_states (
    alert_kind TEXT PRIMARY KEY,
    active BOOLEAN NOT NULL DEFAULT FALSE,
    last_sent_at TIMESTAMP
);
