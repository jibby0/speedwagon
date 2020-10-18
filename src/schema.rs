table! {
    articles (id) {
        id -> Uuid,
        title -> Nullable<Text>,
        published -> Nullable<Timestamp>,
        source_info -> Json,
        summary -> Nullable<Text>,
        content -> Json,
        rights -> Nullable<Text>,
        links -> Json,
        authors -> Json,
        categories -> Json,
        comments_url -> Nullable<Text>,
        extensions -> Json,
        source -> Uuid,
        id_from_source -> Nullable<Text>,
    }
}

table! {
    sources (id) {
        id -> Uuid,
        title -> Text,
        source_data -> Json,
        post_filter -> Text,
        last_post -> Timestamp,
        last_successful_fetch -> Timestamp,
        fetch_errors -> Array<Text>,
        creator -> Text,
        public -> Bool,
    }
}

table! {
    tagged_sources (id) {
        id -> Uuid,
        tag -> Uuid,
        source -> Uuid,
    }
}

table! {
    tags (id) {
        id -> Uuid,
        name -> Text,
        owner -> Text,
    }
}

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

joinable!(articles -> sources (source));
joinable!(sources -> users (creator));
joinable!(tagged_sources -> sources (source));
joinable!(tagged_sources -> tags (tag));
joinable!(tags -> users (owner));
joinable!(tokens -> users (username));

allow_tables_to_appear_in_same_query!(
    articles,
    sources,
    tagged_sources,
    tags,
    tokens,
    users,
);
