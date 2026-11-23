CREATE TABLE items (
    id         INTEGER PRIMARY KEY,
    title      TEXT    NOT NULL,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
