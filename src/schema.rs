table! {
    posts (id) {
        id -> Int4,
        body -> Text,
        timestamp -> Timestamp,
        user_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
    }
}

joinable!(posts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
