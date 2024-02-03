DROP TABLE admin_invite;

DELETE FROM email WHERE event_id IS NULL;

ALTER TABLE email ALTER COLUMN event_id SET NOT NULL;
