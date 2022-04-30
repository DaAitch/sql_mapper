# SQL-Mapper

> *back to the roots*.

From the idea:
- ⏱ compile-time,
- 🚨 schema-safe
- 🗺 ORM
- 🌈 with plain SQL.

Why?

- we think in "data", not in abstractions or indirections
- query builder obfuscate code

Example:

```rust
// main.rs

sql_mapper::sql! {
    -- "schema.rs" Pupil
    SELECT
        *
    FROM
        pupil
}

fn main() {
    let mut conn = db::establish_connection().expect("db connection");
    let pupils = Pupil::query(&mut conn);
}
```

*This is an easy example, which already compiles. More complicated use cases with JOIN, GROUP BY, casts and other functions are currently not (🙏 yet) supported.*

## Details

Check out example code [Simple Select](./examples/simple_select/).

1. use a `diesel`-cli setup with migrations to setup and migrate DB states
1. `diesel` creates a `schema.rs` meta-file that `sql_mapper` uses for SQL-validation.
1. `sql_mapper` currently brings a handcrafted SQL-parser which might be replaced by another parsing library.
