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
        cpu_usage -> Float,
        cpu_total -> Float,
        mem_usage -> Float,
        mem_total -> Float,
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

allow_tables_to_appear_in_same_query!(binaries, heartbeats, tokens,);
