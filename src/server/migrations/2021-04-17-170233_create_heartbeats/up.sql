CREATE TABLE tokens (
   id INTEGER NOT NULL PRIMARY KEY,
   token VARCHAR(32) NOT NULL UNIQUE,
   expires_on INTEGER NOT NULL
);

CREATE TABLE heartbeats (
    id INTEGER NOT NULL PRIMARY KEY,
    token VARCHAR(64) NOT NULL,
    cpu_usage FLOAT NOT NULL,
    mem_usage FLOAT NOT NULL,
    client_timestamp INTEGER NOT NULL,
    server_timestamp INTEGER NOT NULL
);

CREATE TABLE workers (
    id INTEGER NOT NULL PRIMARY KEY,
    token VARCHAR(64) NOT NULL UNIQUE,
    cpu_total FLOAT NOT NULL,
    mem_total FLOAT NOT NULL,
    client_timestamp INTEGER NOT NULL,
    server_timestamp INTEGER NOT NULL
);

CREATE TABLE binaries (
    id INTEGER NOT NULL PRIMARY KEY,
    sha256 VARCHAR(64) NOT NULL UNIQUE,
    signature VARCHAR(1024) NOT NULL UNIQUE
)
