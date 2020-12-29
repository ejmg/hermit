use crate::schema::{post, users};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub name: Option<String>,
    pub username: String,
    pub pw_hash: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub email: String,
    pub date_created: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub pw_hash: String,
    pub email: String,
}

#[derive(Queryable, Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: i32,
    pub author_id: Option<i32>,
    pub text: String,
    pub date_created: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "post"]
pub struct NewPost {
    pub author_id: i32,
    pub text: String,
}
