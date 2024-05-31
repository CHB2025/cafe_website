CREATE TABLE session (
    id uuid default gen_random_uuid() primary key,
    created_at timestamp default now() not null,
    expires_at timestamp,
    user_id uuid references users(id) ON DELETE CASCADE
);
