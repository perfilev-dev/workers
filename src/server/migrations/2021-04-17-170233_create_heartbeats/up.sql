CREATE TABLE challenges (
   id INTEGER NOT NULL PRIMARY KEY,
   ip VARCHAR(16) NOT NULL,
   bytes VARCHAR(32) NOT NULL,
   nonce INTEGER NOT NULL
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
