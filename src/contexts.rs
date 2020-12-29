use crate::models::Post;
use lazy_static;
use regex::Regex;
use rocket::request::Form;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

lazy_static! {
    static ref USERNAME_RGX: Regex =
        Regex::new(r"[[:alph:]]{3,12}").expect("ERROR: SOMETHING IS REALLY WRONG");
}

#[derive(Debug, Deserialize)]
pub struct LoginCtx {}

#[derive(Debug, Deserialize)]
pub struct RegisterCtx {}

#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardCtx {
    pub username: String,
    pub posts: Vec<Post>,
}
