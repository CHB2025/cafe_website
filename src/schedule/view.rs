use askama::Template;
use axum::extract::Path;
use cafe_website::{filters, AppError};
use chrono::{Duration, NaiveDate, NaiveTime, Timelike};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{config, session::Session};

#[derive(Template)]
#[template(path = "schedule/view.html")]
pub struct ScheduleTemplate {
    /// For Block view
    shift_columns: Vec<Vec<ScheduleItemTemplate>>,
    start_time: NaiveTime,
    end_time: NaiveTime,
    event_id: Uuid,
    date: NaiveDate,
    public: bool,
    /// For list view
    grouped_shifts: Vec<ShiftGroup>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ShiftWorker {
    id: Uuid,
    start_time: NaiveTime,
    end_time: NaiveTime,
    title: String,
    // Worker name
    name_first: Option<String>,
    name_last: Option<String>,
}

#[derive(Debug)]
struct ShiftGroup {
    start_time: NaiveTime,
    shifts: Vec<ShiftWorker>,
}

#[derive(Template)]
#[template(path = "schedule/block_view_item.html")]
pub struct ScheduleItemTemplate {
    shifts: Vec<ShiftWorker>,
    start_time: NaiveTime,
    end_time: NaiveTime,
}

pub async fn schedule(
    session: Session,
    Path((event_id, date)): Path<(Uuid, NaiveDate)>,
) -> Result<ScheduleTemplate, AppError> {
    let mut query = QueryBuilder::new(
        "SELECT s.id, s.title, s.start_time, s.end_time, w.name_first, w.name_last 
        FROM shift as s LEFT OUTER JOIN worker as w ON s.worker_id = w.id ",
    );
    query
        .push("WHERE s.date = ")
        .push_bind(date)
        .push(" AND s.event_id = ")
        .push_bind(event_id);

    if !session.is_authenticated() {
        query.push(" AND s.public_signup = TRUE AND w IS NULL");
    }

    query.push(" ORDER BY s.start_time, s.title ASC");
    let shifts = query
        .build_query_as::<ShiftWorker>()
        .fetch_all(config().pool())
        .await?;

    let start_time = shifts
        .first()
        .map(|sh| sh.start_time - Duration::minutes(sh.start_time.minute().into()))
        .unwrap_or(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
    let end_time = shifts
        .last()
        .map(|sh| sh.end_time + Duration::minutes(60i64 - sh.end_time.minute() as i64))
        .unwrap_or(NaiveTime::from_hms_opt(22, 00, 0).unwrap());

    // Columns for groups
    let mut shift_columns: Vec<Vec<ScheduleItemTemplate>> = vec![];
    for shift in shifts.clone() {
        let mut col_ind = 0;
        while shift_columns
            .get(col_ind)
            .and_then(|col| col.last())
            .is_some_and(|sh| {
                sh.end_time > shift.start_time
                    && !(sh.start_time == shift.start_time && sh.end_time == shift.end_time)
            })
        {
            col_ind += 1;
        }
        if col_ind >= shift_columns.len() {
            shift_columns.push(vec![]);
            let end_time = if shift.start_time != start_time {
                shift.start_time
            } else {
                shift.end_time
            };
            shift_columns[col_ind].push(ScheduleItemTemplate {
                shifts: vec![],
                start_time,
                end_time,
            })
        }
        let prev = shift_columns[col_ind]
            .last_mut()
            .expect("Never an empty vec");
        if prev.start_time == shift.start_time {
            prev.shifts.push(shift);
        } else {
            let end_time = prev.end_time;
            if shift.start_time != end_time {
                shift_columns[col_ind].push(ScheduleItemTemplate {
                    shifts: vec![],
                    start_time: end_time,
                    end_time: shift.start_time,
                })
            }
            shift_columns[col_ind].push(ScheduleItemTemplate {
                start_time: shift.start_time,
                end_time: shift.end_time,
                shifts: vec![shift],
            })
        }
    }

    for col in &mut shift_columns {
        let col_end = col.last().expect("No empty columns").end_time;
        if col_end != end_time {
            col.push(ScheduleItemTemplate {
                shifts: vec![],
                start_time: col_end,
                end_time,
            });
        }
    }

    // Groups for list
    let group_start_time = shifts
        .first()
        .map(|sh| sh.start_time)
        .unwrap_or(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
    let mut current = ShiftGroup {
        shifts: Vec::new(),
        start_time: group_start_time,
    };
    let mut grouped_shifts = vec![];
    for shift in shifts {
        if shift.start_time != current.start_time {
            grouped_shifts.push(current);
            current = ShiftGroup {
                shifts: Vec::new(),
                start_time: shift.start_time,
            }
        }
        current.shifts.push(shift);
    }
    if !current.shifts.is_empty() {
        grouped_shifts.push(current);
    }

    Ok(ScheduleTemplate {
        shift_columns,
        event_id,
        date,
        start_time,
        end_time,
        public: !session.is_authenticated(),

        grouped_shifts,
    })
}
