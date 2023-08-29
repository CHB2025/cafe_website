CREATE TABLE event (
    id uuid default gen_random_uuid() primary key,
    name varchar not null,
    start_date date not null,
    end_date date not null,
    allow_signups boolean not null default false
);

CREATE TABLE day (
    id uuid default gen_random_uuid() primary key,
    date date not null,
    entertainment varchar,
    event_id uuid not null references event(id) ON DELETE CASCADE
);

CREATE TABLE worker (
    id uuid default gen_random_uuid() primary key,
    email varchar unique not null,
    phone varchar,
    name_first varchar not null,
    name_last varchar not null
);

CREATE TABLE shift (
    id uuid default gen_random_uuid() primary key,
    day_id uuid not null references day(id) ON DELETE CASCADE,
    start_time time not null,
    end_time time not null,
    title varchar not null,
    description text,
    public_signup boolean not null default true,
    worker_id uuid references worker(id) ON DELETE SET NULL
);

CREATE TABLE users (
    id uuid default gen_random_uuid() primary key,
    email varchar unique not null,
    password varchar not null,
    name varchar not null
);