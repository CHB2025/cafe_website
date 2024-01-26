ALTER TABLE email
ADD COLUMN address varchar;

UPDATE email SET address = worker.email
    FROM worker
    WHERE worker.id = email.recipient;

ALTER TABLE email ALTER COLUMN address SET NOT NULL;
ALTER TABLE email ALTER COLUMN recipient DROP NOT NULL;
