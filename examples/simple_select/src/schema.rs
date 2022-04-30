table! {
    pupil (id) {
        id -> Int4,
        name -> Text,
    }
}

table! {
    teacher (id) {
        id -> Int4,
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    pupil,
    teacher,
);
