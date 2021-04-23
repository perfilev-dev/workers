table! {
    binaries (id) {
        id -> Nullable<Integer>,
        sha256 -> Text,
        signature -> Text,
    }
}

table! {
    heartbeats (id) {
        id -> Nullable<Integer>,
        token -> Text,
        cpu_usage -> Float,
        mem_usage -> Float,
        client_timestamp -> Integer,
        server_timestamp -> Integer,
    }
}

table! {
    tokens (id) {
        id -> Nullable<Integer>,
        token -> Text,
        expires_on -> Integer,
    }
}

table! {
    workers (id) {
        id -> Nullable<Integer>,
        token -> Text,
        cpu_total -> Float,
        mem_total -> Float,
        client_timestamp -> Integer,
        server_timestamp -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    binaries,
    heartbeats,
    tokens,
    workers,
);
