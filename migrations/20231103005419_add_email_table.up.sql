CREATE TYPE email_status AS ENUM ('draft', 'pending', 'sent', 'failed');
CREATE TYPE email_kind AS ENUM ('html', 'text');
CREATE TABLE email (
    id uuid default gen_random_uuid() primary key,
    created_at timestamp with time zone default now() not null,
    sent_at timestamp with time zone,
    status email_status default 'draft' not null,
    kind email_kind default 'html' not null,
    recipient uuid not null references worker(id) ON DELETE CASCADE,
    subject text not null,
    message text not null,
    event_id uuid not null references event(id) ON DELETE CASCADE
);
