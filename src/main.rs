#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::templates::Template;

#[derive(Serialize)]
pub struct Post {
    body: &'static str,
}

#[derive(Serialize)]
pub struct UserContext {
    title: &'static str,
    posts: Vec<Post>,
    name: &'static str,
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
        &UserContext {
            title: "Home",
            posts: ghost_face_posts,
            name: users[0],
        },
    )
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .launch();
}
