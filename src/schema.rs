table! {
    posts (id) {
        id -> Int4,
        user_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
    }
}

joinable!(posts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
