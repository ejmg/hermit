#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use serde::Serialize;

use rocket_contrib::templates::Template;

#[derive(Serialize)]
pub struct TemplateCtx {
    name: &'static str,
}

#[get("/")]
fn index() -> Template {
    let users = ["ghostface killah", "spook", "elias"];
    Template::render("index", &TemplateCtx { name: users[0] })
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .launch();
}
