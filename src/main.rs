#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate validator_derive;
extern crate argon2;
extern crate chrono;
extern crate validator;

#[macro_use]
extern crate lazy_static;

use argon2::{Config as ArgonConfig, ThreadMode, Variant as ArgonVariant, Version as ArgonVersion};
use csrf::{AesGcmCsrfProtection, CsrfCookie, CsrfProtection, CsrfToken};
use data_encoding::BASE64;
use rocket_contrib::databases::diesel;

#[database("hermit_dev")]
struct DevDbConn(diesel::PgConnection);

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
pub mod models;
pub mod util;

struct AppConfig<'a> {
    pub aes_generator: AesGcmCsrfProtection,
    pub csrf_auth_tokens: HashMap<CsrfCookie, CsrfToken>,
    pub pw_config: ArgonConfig<'a>,
}

type HermitConfig<'a> = RwLock<AppConfig<'a>>;

const CSRF_COOKIE_ID: &str = "hermit-cookie";
const SESSION_COOKIE_ID: &str = "hermit-session";

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
    csrf_token: String,
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

    cookies.add_private(Cookie::new(CSRF_COOKIE_ID, cookie_str));

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

    // TODO: this match logic/block can and should probably be broken down because it's a bit
    // much.
    match cookies.get_private(CSRF_COOKIE_ID) {
        Some(c) => {
            let decoded_token = BASE64
                .decode(login.csrf_token.as_bytes())
                .expect("csrf token not base64");
            let decoded_cookie = BASE64
                .decode(c.value().as_bytes())
                .expect("csrf cookie not base64");

            let parsed_token = cfg
                .aes_generator
                .parse_token(&decoded_token)
                .expect("token could not be parsed");

            let parsed_cookie = cfg
                .aes_generator
                .parse_cookie(&decoded_cookie)
                .expect("cookie could not be parsed");

            // TODO use cfg.csrf_auth_tokens.get(&decoded_cookie) to make egregious
            // check that token-pair wasn't spoofed or something, idk? seems unnecessary afterall?
            if cfg
                .aes_generator
                .verify_token_pair(&parsed_token, &parsed_cookie)
            {
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
            } else {
                panic!("Cookie and Token do not match, get out")
            }
        }
        None => panic!("No Csrf Cookie found in headers"),
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
    // TODO: possibly declare in a lazy static?
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
        .attach(DevDbConn::fairing())
        .attach(Template::custom(move |engine| {
            engine
                .tera
                .register_function("url_for", make_url_for(url_for_map.clone()))
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
                        // while i'm not using argonautica, it's demo config has very nice
                        // documentation that helps explain the underlying API/design of Argon2 and
                        // the parameter values to consider:
                        // https://docs.rs/argonautica/0.2.0/argonautica/#configuration
                        pw_config: ArgonConfig {
                            variant: ArgonVariant::Argon2i,
                            version: ArgonVersion::Version13,
                            // x4 default of 4096kb, so 16mb
                            mem_cost: 16384,
                            // x4 default of 3, so 12 passes
                            time_cost: 12,
                            thread_mode: ThreadMode::Parallel,
                            // my dev machine (i7-8550U) has 4 physical cores with 8 threads, so
                            lanes: 4,
                            // default is 32 and that seems strong enough with above choices.
                            hash_length: 32,
                            // this ideally shouldn't be elided but simultaneously i don't know
                            // where to store this value as opposed to my hash in production????
                            secret: &[],
                            // i simply don't fully understand what ad is for
                            ad: &[],
                        },
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
