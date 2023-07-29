CREATE TABLE day (
    id serial primary key,
    date date not null,
    entertainment varchar
);

CREATE TABLE worker (
    id serial primary key,
    email varchar unique not null,
    phone varchar,
    name_first varchar not null,
    name_last varchar not null
);

CREATE TABLE shift (
    id serial primary key,
    day_id int not null references day(id),
    start_time time not null,
    end_time time not null,
    title varchar not null,
    description text,
    worker_id int references worker(id)
);

CREATE TABLE users (
    id serial primary key,
    email varchar unique not null,
    password varchar not null,
    name varchar not null
);