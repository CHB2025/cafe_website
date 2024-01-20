use chrono::NaiveDate;

fn get_date<D: chrono::Datelike>(d: &D) -> NaiveDate {
    NaiveDate::from_ymd_opt(d.year(), d.month(), d.day()).expect("Converting from another date")
}

pub fn date_short<D: chrono::Datelike>(d: &D) -> askama::Result<String> {
    Ok(format!("{}/{}/{}", d.month(), d.day(), d.year()))
}

pub fn date_long<D: chrono::Datelike>(d: &D) -> askama::Result<String> {
    let d = get_date(d);
    Ok(d.format("%a, %b %e").to_string())
}

pub fn time_short<T: chrono::Timelike>(t: &T) -> askama::Result<String> {
    let ap = if t.hour12().0 { "PM" } else { "AM" };
    Ok(format!("{}:{:02} {}", t.hour12().1, t.minute(), ap))
}
