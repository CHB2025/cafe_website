use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use axum_sessions::extractors::ReadableSession;
use chrono::{Duration, NaiveTime, Timelike};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{Shift, User},
};

#[derive(Template)]
#[template(path = "schedule/block_view.html")]
pub struct ScheduleTemplate {
    shift_columns: Vec<Vec<ScheduleItemTemplate>>,
    start_time: NaiveTime,
    end_time: NaiveTime,
}

#[derive(Template)]
#[template(path = "schedule/block_view_item.html")]
pub struct ScheduleItemTemplate {
    shifts: Vec<(Uuid, String)>,
    start_time: NaiveTime,
    end_time: NaiveTime,
}

pub async fn schedule(
    State(app_state): State<AppState>,
    session: ReadableSession,
    Path(day_id): Path<Uuid>,
) -> Result<ScheduleTemplate, StatusCode> {
    let logged_in =
        !session.is_destroyed() && !session.is_expired() && session.get::<User>("user").is_some();

    let shifts = if logged_in {
        sqlx::query_as!(
            Shift,
            "SELECT * FROM shift WHERE day_id = $1 ORDER BY start_time, title ASC",
            day_id
        )
        .fetch_all(app_state.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as!(
            Shift,
            "SELECT * FROM shift WHERE day_id = $1 AND public_signup ORDER BY start_time, title ASC",
            day_id
        )
        .fetch_all(app_state.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let start_time = shifts
        .first()
        .map(|sh| sh.start_time - Duration::minutes(sh.start_time.minute().into()))
        .unwrap_or(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
    let end_time = shifts
        .last()
        .map(|sh| sh.end_time + Duration::minutes(60i64 - sh.end_time.minute() as i64))
        .unwrap_or(NaiveTime::from_hms_opt(22, 00, 0).unwrap());

    let mut shift_columns: Vec<Vec<ScheduleItemTemplate>> = vec![];
    for shift in shifts {
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
            prev.shifts.push((shift.id, shift.title));
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
                shifts: vec![(shift.id, shift.title)],
                start_time: shift.start_time,
                end_time: shift.end_time,
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

    Ok(ScheduleTemplate {
        shift_columns,
        start_time,
        end_time,
    })
}
