mod sql {
    use sql_mapper::sql;

    sql! {
        -- "schema.rs" Pupil
        SELECT
            *
        FROM
            pupil
    }

    sql! {
        -- "schema.rs" Teacher
        SELECT
            *
        FROM
            teacher
    }
}

fn main() {
    dotenv::dotenv().unwrap();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let mut client = postgres::Client::connect(db_url.as_str(), postgres::NoTls).unwrap();

    let pupils = sql::Pupil::query(&mut client).unwrap();
    // [Pupil { pupil_id: 1, pupil_name: "Robert Redrust" }]
    println!("{:?}", pupils);

    let teachers = sql::Teacher::query(&mut client).unwrap();
    // [Teacher { teacher_id: 1, teacher_name: "Rebecca Rustwood" }]
    println!("{:?}", teachers);
}
