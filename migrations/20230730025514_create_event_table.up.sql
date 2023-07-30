CREATE TABLE event (
    id serial primary key,
    name varchar not null,
    start_date date not null,
    end_date date not null,
    allow_signups boolean not null default false
);

ALTER TABLE day ADD COLUMN event_id int references event(id) not null;