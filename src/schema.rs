// @generated automatically by Diesel CLI.

diesel::table! {
    day (id) {
        id -> Int4,
        date -> Date,
        entertainment -> Nullable<Varchar>,
    }
}

diesel::table! {
    shift (id) {
        id -> Int4,
        day_id -> Int4,
        start_time -> Time,
        end_time -> Time,
        title -> Varchar,
        description -> Nullable<Text>,
        worker_id -> Nullable<Int4>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        password -> Varchar,
        name -> Varchar,
    }
}

diesel::table! {
    worker (id) {
        id -> Int4,
        email -> Varchar,
        phone -> Nullable<Varchar>,
        name_first -> Varchar,
        name_last -> Varchar,
    }
}

diesel::joinable!(shift -> day (day_id));
diesel::joinable!(shift -> worker (worker_id));

diesel::allow_tables_to_appear_in_same_query!(
    day,
    shift,
    users,
    worker,
);
