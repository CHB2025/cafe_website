ALTER TABLE shift ADD event_id uuid;
ALTER TABLE shift ADD date date;
UPDATE shift SET (date, event_id) = (SELECT date, event_id FROM day WHERE shift.day_id = day.id);
ALTER TABLE shift DROP day_id;
ALTER TABLE shift ALTER COLUMN event_id SET NOT NULL;
ALTER TABLE shift ALTER COLUMN date SET NOT NULL;

ALTER TABLE day DROP id;
ALTER TABLE day ADD CONSTRAINT date_event_pk PRIMARY KEY(date, event_id);
ALTER TABLE shift ADD CONSTRAINT date_event_fk FOREIGN KEY (date, event_id) REFERENCES day (date, event_id);


ALTER TABLE event DROP start_date;
ALTER TABLE event DROP end_date; -- How do I undo this?

