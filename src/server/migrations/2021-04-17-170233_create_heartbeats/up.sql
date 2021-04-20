CREATE TABLE tokens (
   id INTEGER NOT NULL PRIMARY KEY,
   token VARCHAR(32) NOT NULL UNIQUE,
   expires_on INTEGER NOT NULL
);

CREATE TABLE heartbeats (
    id INTEGER NOT NULL PRIMARY KEY,
    cpu_usage FLOAT NOT NULL,
    cpu_total FLOAT NOT NULL,
    mem_usage FLOAT NOT NULL,
    mem_total FLOAT NOT NULL,
    client_timestamp INTEGER NOT NULL,
    server_timestamp INTEGER NOT NULL
);
