use chrono::NaiveDateTime;
use rocket_contrib::databases::diesel::{Associations, Identifiable, Queryable};

#[derive(Queryable)]
pub struct User {
    id: u32,
    username: String,
    email: String,
    password_hash: String,
}

#[derive(Queryable)]
pub struct Post {
    id: u32,
    body: String,
    timestamp: NaiveDateTime,
    user_id: u32,
}
