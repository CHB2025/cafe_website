use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveTime;

use crate::{app_state::AppState, models::Shift};

#[derive(Template)]
#[template(path = "schedule.html")]
pub struct ScheduleTemplate {
    editable: bool,
    day_id: i32,
    shift_columns: Vec<Vec<ScheduleItemTemplate>>,
}

#[derive(Template)]
#[template(path = "schedule-item.html")]
pub struct ScheduleItemTemplate {
    title: Option<String>,
    start_time: NaiveTime,
    end_time: NaiveTime,
}

pub async fn schedule(
    State(app_state): State<AppState>,
    Path(day_id): Path<i32>,
) -> Result<ScheduleTemplate, StatusCode> {
    let shifts = sqlx::query_as!(
        Shift,
        "SELECT * FROM shift WHERE day_id = $1 ORDER BY start_time, title ASC",
        day_id
    )
    .fetch_all(app_state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let start_time = shifts
        .first()
        .map(|sh| sh.start_time)
        .unwrap_or(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
    let end_time = shifts
        .last()
        .map(|sh| sh.end_time)
        .unwrap_or(NaiveTime::from_hms_opt(10, 30, 0).unwrap());

    // Split the shifts into columns first, then turn each column into ScheduleItems with the missing time
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
            if shift.start_time != start_time {
                shift_columns[col_ind].push(ScheduleItemTemplate {
                    title: None,
                    start_time,
                    end_time: shift.start_time,
                })
            }
            shift_columns[col_ind].push(ScheduleItemTemplate {
                title: Some(shift.title),
                start_time: shift.start_time,
                end_time: shift.end_time,
            })
        } else {
            let prev = shift_columns[col_ind]
                .last_mut()
                .expect("Never an empty vec");
            if prev.start_time == shift.start_time {
                prev.title
                    .iter_mut()
                    .for_each(|t| *t = format!("{}\n{}", t, shift.title));
            } else {
                let end_time = prev.end_time;
                if shift.start_time != end_time {
                    shift_columns[col_ind].push(ScheduleItemTemplate {
                        title: None,
                        start_time: end_time,
                        end_time: shift.start_time,
                    })
                }
                shift_columns[col_ind].push(ScheduleItemTemplate {
                    title: Some(shift.title),
                    start_time: shift.start_time,
                    end_time: shift.end_time,
                })
            }
        }
    }

    for col in &mut shift_columns {
        let col_end = col.last().expect("No empty columns").end_time;
        if col_end != end_time {
            col.push(ScheduleItemTemplate {
                title: None,
                start_time: col_end,
                end_time,
            });
        }
    }

    // let mut shift_columns: Vec<Vec<ScheduleItemTemplate>> = vec![];
    // for column in columns {
    //     shift_columns.push(vec![]);
    //     let col_ref = shift_columns.last_mut().unwrap();
    //     let mut prev_end = start_time;

    //     for shift in column {
    //         if shift.start_time > prev_end {
    //             col_ref.push(ScheduleItemTemplate {
    //                 shift: None,
    //                 time: (shift.start_time - prev_end).whole_minutes(),
    //             });
    //         }
    //         prev_end = shift.end_time;
    //         col_ref.push(ScheduleItemTemplate {
    //             time: (shift.end_time - shift.start_time).whole_minutes(),
    //             shift: Some(shift),
    //         })
    //     }

    //     if prev_end < end_time {
    //         col_ref.push(ScheduleItemTemplate {
    //             shift: None,
    //             time: (end_time - prev_end).whole_minutes(),
    //         })
    //     }
    // }

    Ok(ScheduleTemplate {
        day_id,
        editable: true,
        shift_columns,
    })
}
