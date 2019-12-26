#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::templates::Template;

use rocket::fairing::AdHoc;
use rocket::http::{Cookie, Cookies};
struct CsrfSecret(String);

#[derive(Serialize)]
pub struct LoginForm {
    username: String,
    password: String,
    remember_me: bool,
    submit: &'static str,
}

#[derive(Serialize)]
pub struct Post {
    body: &'static str,
}

#[derive(Serialize)]
pub struct PageContext {
    title: &'static str,
    posts: Vec<Post>,
    name: &'static str,
}

#[derive(Serialize)]
pub struct LoginContext {
    csrf_token: &'static str,
}

#[get("/login")]
fn login(mut cookies: Cookies) -> Template {
    // TODO set private cookie
    cookies.add_private(Cookie::new("user", "value"));
    // TODO progmatically generate csrf token
    // TODO associate private cookie with csrf token and store it
    Template::render(
        "login",
        &LoginContext {
            csrf_token: "FOOBAR",
        },
    )
}

#[get("/index")]
fn index_redir() -> Redirect {
    Redirect::permanent("/")
}

#[get("/")]
fn index() -> Template {
    let users = ["ghostface killah", "spook", "elias"];
    let ghost_face_posts = vec![
        Post {
            body: r"Listen, you could never match my velocity
Too much stamina, glitter in front of cameras
On the red carpet, still clean your clock like a janitor",
        },
        Post {
            body: r"Got the Montana booth in front of the store
Made my usual gun check, safety off, come on Frank
The moment is here, take your fuckin' hood off
And tell the driver to stay put",
        },
    ];
    Template::render(
        "index",
        &PageContext {
            title: "Home",
            posts: ghost_face_posts,
            name: users[0],
        },
    )
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("CSRF Secret Key", |rocket| {
            let csrf_secret = rocket
                .config()
                .get_str("csrf_secret_key")
                .unwrap_or("csrf-secret-key-here")
                .to_string();

            Ok(rocket.manage(CsrfSecret(csrf_secret)))
        }))
        .mount("/", routes![index, index_redir, login])
        .launch();
}
