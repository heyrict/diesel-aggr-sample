use diesel::prelude::*;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
}

#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
}
