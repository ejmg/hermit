-- Your SQL goes here
create table posts (
       id serial primary key,
       body text not null,
       timestamp timestamp not null,
       user_id serial references users(id)
);
