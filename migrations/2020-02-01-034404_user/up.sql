create table users (
       id serial primary key,
       username varchar not null,
       email varchar not null,
       password_hash varchar not null
);
