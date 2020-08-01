table! {
    tokens (id) {
        id -> Uuid,
        username -> Text,
        expires -> Timestamp,
    }
}

table! {
    users (username) {
        username -> Text,
        password -> Text,
    }
}

joinable!(tokens -> users (username));

allow_tables_to_appear_in_same_query!(
    tokens,
    users,
);
