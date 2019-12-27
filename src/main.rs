#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use csrf::{AesGcmCsrfProtection, CsrfCookie, CsrfProtection, CsrfToken};
use data_encoding::BASE64;
use rocket::fairing::AdHoc;
use rocket::http::RawStr;
use rocket::http::{Cookie, Cookies};
use rocket::request::FlashMessage;
use rocket::request::{Form, FromFormValue, LenientForm};
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::RwLock;

struct AppConfig {
    pub aes_generator: AesGcmCsrfProtection,
    pub csrf_auth_tokens: HashMap<CsrfCookie, CsrfToken>,
}

type HermitConfig = RwLock<AppConfig>;
struct CsrfSecret(String);

// TODO impl explicit username and password types
// TODO impl FromFormValue impl's that enforce validation constraints on explicit types from
// above Todo item
#[derive(FromForm)]
pub struct LoginForm<'f> {
    username: &'f RawStr,
    password: &'f RawStr,
    remember_me: bool,
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
    csrf_token: String,
    flash: String,
}

#[get("/login", rank = 0)]
fn login(
    flash: Option<FlashMessage>,
    mut cookies: Cookies,
    state: State<HermitConfig>,
) -> Template {
    let mut cfg = state
        .write()
        .expect("Cannot write, config locked by Readers");

    let (token, cookie) = (cfg)
        .aes_generator
        .generate_token_pair(None, 300)
        .expect("couldn't generate token-cookie pair");

    cfg.csrf_auth_tokens.insert(cookie.clone(), token.clone());

    let token_str: String = token.b64_string();
    let cookie_str = cookie.b64_string();

    cookies.add_private(Cookie::new("hermit-session", cookie_str));

    drop(cookies);

    let mut s = String::new();

    if let Some(ref msg) = flash {
        println!("value of msg.msg(): {:?}", msg.msg());
        s = String::from(msg.msg());
    }

    Template::render(
        "login",
        &LoginContext {
            csrf_token: token_str,
            flash: s,
        },
    )
}

#[post("/login", data = "<login>")]
fn login_submit(
    state: State<HermitConfig>,
    login: LenientForm<LoginForm>,
    mut cookies: Cookies,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let mut cfg = state.read().expect("Cannot read, config locked by Writer");
    println!("value of login.username: {:?}", login.username.as_str());
    println!("value of login.password: {:?}", login.password.as_str());

    if login.username.as_str() != "" && login.password.as_str() != "" {
        println!("About to redirect...");
        Ok(Flash::success(
            Redirect::to(uri!(login)),
            "Successfully logged in!",
        ))
    } else {
        println!("Error on login...");
        Err(Flash::error(
            Redirect::to(uri!(login)),
            "Invalid username/password.",
        ))
    }
}
#[get("/index")]
fn index_redir() -> Redirect {
    Redirect::permanent("/")
}

#[get("/")]
fn index(flash: Option<FlashMessage>) -> Template {
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

                    Ok(rocket.manage(RwLock::new(AppConfig {
                        aes_generator: AesGcmCsrfProtection::from_key(arr_secret),
                        csrf_auth_tokens: HashMap::new(),
                    })))
                }
                None => panic!("No CsrfSecret, unable to generate AppConfig struct"),
            }
        }))
        .mount(
            "/",
            routes![
                index,
                index_redir,
                login,
                login_submit,
            ],
        )
        .launch();
}
