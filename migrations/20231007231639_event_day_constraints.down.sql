ALTER TABLE day ADD id uuid DEFAULT gen_random_uuid();

ALTER TABLE shift ADD day_id uuid;
UPDATE shift SET (day_id) = (SELECT id FROM day WHERE (shift.event_id, shift.date) = (day.event_id, day.date));
ALTER TABLE shift DROP CONSTRAINT date_event_fk;
ALTER TABLE shift DROP date;
ALTER TABLE shift DROP event_id;

ALTER TABLE day DROP CONSTRAINT date_event_pk;
ALTER TABLE day ADD CONSTRAINT id_pk PRIMARY KEY(id);

ALTER TABLE shift ADD CONSTRAINT day_id_fk FOREIGN KEY (day_id) REFERENCES day (id);


ALTER TABLE event ADD start_date date NOT NULL DEFAULT CURRENT_DATE;
ALTER TABLE event ADD end_date date NOT NULL DEFAULT CURRENT_DATE;

UPDATE event SET start_date = dates.sd, end_date = dates.ed 
FROM (
    SELECT event_id, min(date) as sd, max(date) as ed
    FROM day 
    GROUP BY event_id
) as dates
WHERE dates.event_id = event.id;

ALTER TABLE event ALTER COLUMN start_date DROP DEFAULT;
ALTER TABLE event ALTER COLUMN end_date DROP DEFAULT;