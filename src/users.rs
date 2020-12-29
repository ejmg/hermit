use crate::models::{NewUser, User};
use crate::schema::{users, users::dsl::*};
use diesel::prelude::*;
use pwhash::bcrypt::verify;

pub fn create_user(db: &PgConnection, new_user: NewUser) -> User {
    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(db)
        .expect(&format!("Could not insert new user: {:?}", &new_user))
}

pub fn login_user(db: &PgConnection, username_: String, password_: String) -> Result<User, String> {
    let user = users.filter(username.eq(username_)).first::<User>(db);

    match user {
        Ok(u) => {
            if verify(&password_, &u.pw_hash) {
                Ok(u)
            } else {
                Err(String::from("Password incorrect"))
            }
        }
        Err(e) => Err(String::from("Username does not exist")),
    }
}

pub fn get_user_by_id(db: &PgConnection, uid: i32) -> Result<User, String> {
    let user = users.find(uid).first::<User>(db);

    match user {
        Ok(u) => Ok(u),
        Err(e) => Err(String::from("User ID does not exist")),
    }
}
