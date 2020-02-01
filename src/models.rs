use rocket_contrib::databases::diesel::Queryable;

#[derive(Queryable)]
pub struct User {
    id: u32,
    username: String,
    email: String,
    password_hash: String,
}
