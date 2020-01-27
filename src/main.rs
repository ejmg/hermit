#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate validator_derive;
extern crate validator;

#[macro_use]
extern crate lazy_static;

use csrf::{AesGcmCsrfProtection, CsrfCookie, CsrfProtection, CsrfToken};
use data_encoding::BASE64;
use rocket_contrib::templates::Template;
use serde_json::value::from_value;
use serde_json::value::to_value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::RwLock;

use rocket::{
    fairing::AdHoc,
    http::{Cookie, Cookies, RawStr},
    request::{FlashMessage, Form, FromFormValue, LenientForm},
    response::{Flash, Redirect},
    State,
};

use regex::Regex;
use validator::{Validate, ValidationError};

use rocket_contrib::templates::tera::{GlobalFn, Result as TeraResult};

struct AppConfig {
    pub aes_generator: AesGcmCsrfProtection,
    pub csrf_auth_tokens: HashMap<CsrfCookie, CsrfToken>,
}

type HermitConfig = RwLock<AppConfig>;

lazy_static! {
    static ref VALID_USERNAME_REGEX: Regex =
        Regex::new(r"^[[:word:]-]{3,10}").expect("Failed to build user name regex! Wth!?");
}
struct CsrfSecret(String);

// TODO impl explicit username and password types
// TODO impl FromFormValue impl's that enforce validation constraints on explicit types from
// above Todo item
#[derive(Validate, FromForm)]
pub struct LoginForm {
    #[validate(regex(
        path = "VALID_USERNAME_REGEX",
        message = "Invalid username. A-Za-z0-9, '-', and '_' characters and of 3 to 10 characters long."
    ))]
    username: String,
    #[validate(length(
        min = 12,
        max = 64,
        message = "Invalid password. Minimum length of 12, maximum of 64."
    ))]
    password: String,
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
    error: bool,
}

#[get("/login")]
fn login(
    flash: Option<FlashMessage>,
    mut cookies: Cookies,
    state: State<HermitConfig>,
) -> Template {
    let mut cfg = state
        .write()
        .expect("Cannot write, config locked by Readers");

    let (token, cookie) = cfg
        .aes_generator
        .generate_token_pair(None, 300)
        .expect("couldn't generate token-cookie pair");

    cfg.csrf_auth_tokens.insert(cookie.clone(), token.clone());

    let token_str: String = token.b64_string();
    let cookie_str = cookie.b64_string();

    cookies.add_private(Cookie::new("hermit-session", cookie_str));

    // Rocket cannot currently handly 2 instances of Cookies in a single handler. Must drop one
    // instance within the handler for it to work as intended:
    // https://rocket.rs/v0.4/guide/requests/#one-at-a-time
    // https://github.com/SergioBenitez/Rocket/issues/1090#issuecomment-522619500
    // https://github.com/SergioBenitez/Rocket/issues/934
    drop(cookies);

    let mut s = String::new();
    let mut err_msg = false;

    // If we were redirected via a Flash Redirect, handle that here.
    if let Some(ref msg) = flash {
        s = String::from(msg.msg());
        if msg.name() == "error" {
            err_msg = true;
        }
    }

    Template::render(
        "login",
        &LoginContext {
            csrf_token: token_str,
            flash: s,
            error: err_msg,
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

    match login.validate() {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!(login)),
            "Successfully logged in!",
        )),
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(login)),
            "Login failed: Invalid username or password.",
        )),
    }
}

#[get("/index")]
fn index_redir() -> Redirect {
    Redirect::permanent("/")
}

#[get("/")]
fn index(flash: Option<FlashMessage>) -> Template {
    let users = ["ghostface killah", "spook", "elias"];
    let ghost_posts = vec![
        Post {
            body: r"
Listen, you could never match my velocity
Too much stamina, glitter in front of cameras
On the red carpet, still clean your clock like a janitor",
        },
        Post {
            body: r"
That night, yo, I was hittin' like a spiked bat
And then you thought I was bugged out, and crazy
Strapped for nonsense, after me became lazy
Yo, nobody budge while I shot slugs
Never shot thugs, I'm runnin' with thugs that flood mugs",
        },
    ];
    Template::render(
        "index",
        &PageContext {
            title: "Home",
            name: users[0],
            posts: ghost_posts,
        },
    )
}

fn make_url_for(urls: BTreeMap<String, String>) -> GlobalFn {
    Box::new(move |args| -> TeraResult<serde_json::Value> {
        match args.get("name") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) => Ok(to_value(urls.get(&v).unwrap()).unwrap()),
                Err(_) => Err("oops".into()),
            },
            None => Err("oops".into()),
        }
    })
}

fn main() {
    let app_routes = routes![index, index_redir, login, login_submit,];
    let mut url_for_map = BTreeMap::new();
    for route in app_routes {
        match (route.name, route.uri.path()) {
            (Some(name), path) => {
                url_for_map.insert(String::from(name), String::from(path));
            }
            (_, path) => panic!(
                "Could not generate a name for each path provided by route!, path: {}",
                path
            ),
        }
    }

    rocket::ignite()
        .attach(Template::custom(|engine| {
            engine
                .tera
                .register_function("url_for", make_url_for(url_for_map))
        }))
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
        .mount("/", routes![index, index_redir, login, login_submit,])
        .launch();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        assert!(VALID_USERNAME_REGEX.is_match("elias"));
        assert!(VALID_USERNAME_REGEX.is_match("e_j-m_g"));
        assert!(VALID_USERNAME_REGEX.is_match("0e_j-mg1"));
        assert!(!VALID_USERNAME_REGEX.is_match("😬ejmg"));
        assert!(!VALID_USERNAME_REGEX.is_match(r"e\jmg"));
        assert!(!VALID_USERNAME_REGEX.is_match(" "));
        assert!(!VALID_USERNAME_REGEX.is_match(""));
    }
}
