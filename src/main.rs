#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate lazy_static;

extern crate pwhash;

use crate::contexts::DashboardCtx;
use rocket::http::{Cookie, Cookies};
use rocket::request::{FlashMessage, LenientForm};
use rocket::response::{Flash, Redirect};

use rocket_contrib::templates::Template;

use serde::Deserialize;
use validator::{Validate, ValidationError};

pub mod contexts;
pub mod forms;
pub mod models;
pub mod posts;
pub mod schema;
pub mod users;

#[database("hermitdb")]
struct HermitDb(diesel::PgConnection);

use pwhash::bcrypt;
use std::collections::hash_map::HashMap;

use crate::forms::{LoginForm, PostForm, RegisterForm};
use crate::models::{NewPost, NewUser, Post, User};
use crate::posts::{create_new_post, get_posts_by_user};
use crate::users::{create_user, get_user_by_id, login_user};

#[get("/")]
fn index() -> Template {
    let mut context: HashMap<String, String> = HashMap::new();
    Template::render("base", &context)
}

#[get("/login")]
fn login() -> Template {
    let mut context: HashMap<String, String> = HashMap::new();
    Template::render("login", &context)
}

#[post("/login", data = "<login>")]
fn login_post(mut cookies: Cookies, login: LenientForm<LoginForm>, dbctx: HermitDb) -> String {
    match &login.validate() {
        Ok(_) => {
            println!("valid login form! {:?}", login);
            let res = login_user(&dbctx, login.username.clone(), login.password.clone());
            match res {
                Ok(u) => {
                    cookies.add_private(Cookie::new("hermit_id", u.id.to_string()));
                    format!("User logged in! {:?}", u)
                }
                Err(e) => format!("error, could not login user: {:?}", e),
            }
        }
        Err(e) => format!("Error, could not login user: {:?}", e),
    }
}

#[get("/logout")]
fn logout() -> Template {
    let mut context: HashMap<String, String> = HashMap::new();
    Template::render("logout", &context)
}

#[get("/register")]
fn register() -> Template {
    let mut context: HashMap<String, String> = HashMap::new();
    Template::render("register", &context)
}

#[post("/register", data = "<new_user>")]
fn register_post(new_user: LenientForm<RegisterForm>, dbctx: HermitDb) -> String {
    match &new_user.validate() {
        Ok(_) => {
            println!("Valid user form!");
            let pw_hash = pwhash::bcrypt::hash(new_user.password.clone()).unwrap();
            println!("new user password: {}", &new_user.password);
            println!("new user pw_hash: {:?}", &pw_hash);
            let new_user = NewUser {
                username: new_user.username.clone(),
                pw_hash,
                email: new_user.email.clone(),
            };
            let res = create_user(&dbctx, new_user);
            format!("New user created! {:?}", res)
        }
        Err(e) => format!("invalid user registration: {:?}", e),
    }
}

#[get("/dashboard")]
fn dashboard(mut cookies: Cookies, dbctx: HermitDb) -> Result<Template, Flash<Redirect>> {
    match cookies.get_private("hermit_id") {
        Some(c) => {
            let uid = c
                .value()
                .parse::<i32>()
                .expect("ERROR: COULD NOT PARSE COOKIE VALUE");
            let user = get_user_by_id(&dbctx, uid);

            match user {
                Ok(u) => {
                    println!("User authenticated!");
                    let user_posts = get_posts_by_user(&dbctx, uid);

                    Ok(Template::render(
                        "dashboard",
                        &DashboardCtx {
                            username: u.username.clone(),
                            posts: user_posts,
                        },
                    ))
                }
                Err(e) => {
                    println!("YO! YOU AIN'T LOGGED IN, DUMB ASS");
                    Err(Flash::error(Redirect::to("/"), format!("Error: {:?}", e)))
                }
            }
        }
        None => {
            println!("YO! YOU AIN'T LOGGED IN, DUMB ASS");

            Err(Flash::error(
                Redirect::to("/"),
                "You must be logged in to view dashboard",
            ))
        }
    }
}

#[post("/create_post", data = "<new_post>")]
fn create_post(
    mut cookies: Cookies,
    new_post: LenientForm<PostForm>,
    dbctx: HermitDb,
) -> Result<Template, Flash<Redirect>> {
    match cookies.get_private("hermit_id") {
        Some(c) => {
            let uid = c
                .value()
                .parse::<i32>()
                .expect("ERROR: COULD NOT PARSE COOKIE VALUE");
            let user = get_user_by_id(&dbctx, uid);

            match new_post.validate() {
                Ok(_) => (),
                Err(e) => {
                    println!("Error: Your post must be 480 characters or less.");
                    return Err(Flash::error(
                        Redirect::to("/"),
                        "Error: Your post must be 480 characters or less.",
                    ));
                }
            }

            match user {
                Ok(u) => {
                    println!("User authenticated!");
                    let new_post = create_new_post(
                        &dbctx,
                        NewPost {
                            author_id: uid,
                            text: new_post.text.clone(),
                        },
                    );
                    println!("New post created! {:?}", new_post.text);
                    let user_posts = get_posts_by_user(&dbctx, uid);

                    Ok(Template::render(
                        "dashboard",
                        &DashboardCtx {
                            username: u.username.clone(),
                            posts: user_posts,
                        },
                    ))
                }
                Err(e) => {
                    println!("Error creating post: {:?}", e);
                    Err(Flash::error(Redirect::to("/"), format!("Error: {:?}", e)))
                }
            }
        }
        None => {
            println!("Error: You must be logged in to create posts");
            Err(Flash::error(
                Redirect::to("/"),
                "You must be logged in to create posts",
            ))
        }
    }
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .attach(HermitDb::fairing())
        .mount(
            "/",
            routes![
                index,
                create_post,
                login,
                login_post,
                register,
                register_post,
                logout,
                dashboard
            ],
        )
        .launch();
}
