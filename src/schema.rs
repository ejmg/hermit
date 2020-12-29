table! {
    post (id) {
        id -> Int4,
        author_id -> Nullable<Int4>,
        text -> Text,
        date_created -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
        username -> Varchar,
        pw_hash -> Text,
        bio -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
        email -> Varchar,
        date_created -> Timestamptz,
    }
}

joinable!(post -> users (author_id));

allow_tables_to_appear_in_same_query!(
    post,
    users,
);
