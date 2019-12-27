#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use csrf::{AesGcmCsrfProtection, CsrfCookie, CsrfProtection, CsrfToken};
use data_encoding::BASE64;
use rocket::fairing::AdHoc;
use rocket::http::{Cookie, Cookies};
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct AppConfig {
    pub aes_generator: AesGcmCsrfProtection,
    pub csrf_auth_tokens: HashMap<CsrfCookie, CsrfToken>,
}

type HermitConfig = Arc<RwLock<AppConfig>>;
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
fn login<'r>(mut cookies: Cookies, mut state: State<'r, HermitConfig>) -> Template {
    // TODO: correctly impl RwLock logic over Csrf hashmap
    // let (token, cookie) = state
    //     .aes_generator
    //     .generate_token_pair(None, 300)
    //     .expect("couldn't generate token/cookie pair");

    // let token_str = token.b64_string();
    // let cookie_str = cookie.b64_string();

    // cookies.add_private(Cookie::new("user", "cookie"));

    // state.csrf_auth_tokens.insert(cookie, token);

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
                .unwrap_or("You-dont-have-a-csrf-secret-configured!")
                .to_string();
            Ok(rocket.manage(CsrfSecret(csrf_secret)))
        }))
        .attach(AdHoc::on_attach("AppConfig", |rocket| {
            let csrf_secret = rocket.state::<CsrfSecret>();

            let mut arr_secret: [u8; 32] = Default::default();

            match csrf_secret {
                Some(secret) => {
                    arr_secret.copy_from_slice(&secret.0.as_bytes()[0..32]);

                    Ok(rocket.manage(Arc::new(RwLock::new(AppConfig {
                        aes_generator: AesGcmCsrfProtection::from_key(arr_secret),
                        csrf_auth_tokens: HashMap::new(),
                    }))))
                }
                None => panic!("No CsrfSecret, unable to generate AppConfig struct"),
            }
        }))
        .mount("/", routes![index, index_redir, login])
        .launch();
}
