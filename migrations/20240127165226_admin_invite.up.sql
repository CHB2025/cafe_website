CREATE TABLE admin_invite (
    id uuid default gen_random_uuid() primary key,
    created_at timestamp with time zone default now() not null,
    accepted_at timestamp with time zone,
    email varchar not null
);

ALTER TABLE email ALTER COLUMN event_id DROP NOT NULL;
