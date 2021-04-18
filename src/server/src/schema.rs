table! {
    challenges (id) {
        id -> Nullable<Integer>,
        ip -> Text,
        bytes -> Text,
        nonce -> Integer,
    }
}

table! {
    heartbeats (id) {
        id -> Nullable<Integer>,
        cpu_usage -> Float,
        cpu_total -> Float,
        mem_usage -> Float,
        mem_total -> Float,
        client_timestamp -> Integer,
        server_timestamp -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    challenges,
    heartbeats,
);
