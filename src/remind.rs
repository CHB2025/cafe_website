use askama::Template;
use cafe_website::{filters, AppError};
use sqlx::QueryBuilder;
use tracing::debug;
use uuid::Uuid;

use crate::{config, config::Admin, models::Shift, worker::Worker};

#[derive(Debug, Clone, Template)]
#[template(path = "email/messages/reminder.html")]
pub struct Reminder {
    worker: Worker,
    shifts: Vec<Shift>,
    admin: &'static Admin,
    domain: String,
    locked: bool,
}

pub async fn remind_one(
    event_id: Uuid,
    worker: Worker,
    locked: bool,
) -> Result<Reminder, AppError> {
    let shifts = sqlx::query_as!(
        Shift,
        "SELECT * FROM shift WHERE event_id = $1 AND worker_id = $2 ORDER BY date, start_time",
        event_id,
        worker.id
    )
    .fetch_all(config().pool())
    .await?;

    Ok(Reminder {
        worker,
        shifts,
        admin: &config().admin,
        domain: config().url(),
        locked,
    })
}

pub async fn remind_all(event_id: Uuid, locked: bool) -> Result<Vec<Reminder>, AppError> {
    let workers = sqlx::query_as!(
        Worker,
        "SELECT w.* FROM worker as w 
        INNER JOIN shift as s ON s.worker_id = w.id 
        INNER JOIN event as e ON s.event_id = e.id 
        WHERE e.id = $1
        GROUP BY w.id",
        event_id
    )
    .fetch_all(config().pool())
    .await?;
    debug!("Creating reminders for {} workers", workers.len());
    let mut res = vec![];
    for worker in workers {
        let shifts = sqlx::query_as!(
            Shift,
            "SELECT * FROM shift WHERE event_id = $1 AND worker_id = $2 ORDER BY date, start_time",
            event_id,
            worker.id
        )
        .fetch_all(config().pool())
        .await?;
        res.push(Reminder {
            worker,
            shifts,
            admin: &config().admin,
            domain: config().url(),
            locked,
        })
    }

    Ok(res)
}

pub async fn send_all_reminders(event_id: Uuid) -> Result<(), AppError> {
    // If reminders can be sent without the event being hidden to the public,
    // this will need to change
    let reminders: Vec<(Reminder, String)> = remind_all(event_id, true)
        .await?
        .into_iter()
        .map(|reminder| -> Result<_, AppError> { Ok((reminder.clone(), reminder.render()?)) })
        .collect::<Result<Vec<_>, AppError>>()?;

    if reminders.is_empty() {
        return Ok(());
    }
    let mut email_query = QueryBuilder::new(
        "INSERT INTO email (status, kind, recipient, address, subject, message, event_id) ",
    );
    email_query.push_values(reminders, |mut b, (reminder, body)| {
        b.push("'pending'")
            .push("'html'")
            .push_bind(reminder.worker.id)
            .push_bind(reminder.worker.email)
            .push("'Your shifts at the Cornerstone Cafe'")
            .push_bind(body)
            .push_bind(event_id);
    });

    email_query
        .build()
        .persistent(false)
        .execute(config().pool())
        .await?;
    Ok(())
}
