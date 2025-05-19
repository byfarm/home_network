CREATE TABLE IF NOT EXISTS location (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT
);

CREATE TABLE IF NOT EXISTS data (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    location_id INTEGER NOT NULL,
    timestamp TIMESTAMP DEFAULT (strftime('%s', 'now')) NOT NULL,
    value FLOAT,
    measurand TEXT,
    units TEXT,
    FOREIGN KEY (location_id) REFERENCES location(id)
);

INSERT INTO
    location (name)
VALUES
    ("kitchen");
