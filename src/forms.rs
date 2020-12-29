use lazy_static;
use regex::Regex;
use rocket::request::LenientForm;
use serde::Deserialize;
use validator::{Validate, ValidationError};

lazy_static! {
    static ref USERNAME_RGX: Regex =
        Regex::new(r"[[:alpha:]]{3,12}").expect("ERROR: SOMETHING IS REALLY WRONG");
}

#[derive(FromForm, Debug, Validate, Deserialize)]
pub struct LoginForm {
    #[validate(length(min = 3, max = 12))]
    #[validate(regex = "USERNAME_RGX")]
    pub username: String,
    #[validate(length(min = 14))]
    pub password: String,
}

#[derive(FromForm, Debug, Validate, Deserialize)]
pub struct RegisterForm {
    #[validate(length(min = 3, max = 12))]
    #[validate(regex = "USERNAME_RGX")]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 14))]
    pub password: String,
    #[validate(must_match = "password")]
    pub verify_password: String,
}

#[derive(FromForm, Debug, Validate, Deserialize)]
pub struct PostForm {
    #[validate(length(min = 1, max = 480))]
    pub text: String,
}
