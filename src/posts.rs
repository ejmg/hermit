use crate::models::{NewPost, Post};
use crate::schema::{post, post::dsl::*};
use diesel::prelude::*;

pub fn create_new_post(db: &PgConnection, new_post: NewPost) -> Post {
    diesel::insert_into(post::table)
        .values(new_post)
        .get_result::<Post>(db)
        .expect("ERROR: COULD NOT INSERT POST")
}

pub fn get_posts_by_user(db: &PgConnection, uid: i32) -> Vec<Post> {
    post.filter(author_id.eq(uid))
        .order(date_created.desc())
        .load::<Post>(db)
        .expect("COULD NOT GET USER POSTS BY ID")
}
