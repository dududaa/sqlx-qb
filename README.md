# SQLX-QB

A simple query builder for [SQLx](https://github.com/launchbadge/sqlx).

If you use rust SQLx, you most likely realize how great the library is. However, you have to write (and rewrite) raw
queries
for every single task no matter how simple. This is where [SqlxQB](#sqlx-qb) comes in. It aims to simplify the process
of writing simple CRUD queries by providing APIs that write the queries for you and map the results to your models.

> **⚠️ Status: Early-stage project**<br>
>This library is still in its early development phase and is not yet production-ready. Expect breaking changes and
> incomplete features.

That said, feedback, testing, and contributions are highly encouraged. If this project interests you, feel free to get
involved!

### Installation

Add it to your Cargo.toml

```toml
sqlx_qb = { git = "https://github.com/dududaa/sqlx-qb" } 
```

### Usage

Follow the steps in this sample code to start using QB:

```rust
use sqlx::FromRow;
use sqlx_qb::prelude::*;

// 1. Your model must derive FromRow
#[derive(FromRow)]
struct UserModel {
    id: i32,
    name: String,
    age: i16,
    public_id: Uuid,
    created_at: Datetime<Utc>
}

// 2. Implement qb's Model for your model to provide a table name (and other auto-implemented methods). This step won't be necessary in future updates;
impl Model for UserModel {
    const TABLE_NAME: &'static str = "users";
}

// 3. Start using QB. This function demonstrates how to use QB.
async fn qb_demo() -> anyhow::Result<()> {
    // create sqlx pool by yourself (I'm deliberately leaving this up to you)
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    // INSERT new user.
    let map = query_map! {
      "name": "Demo User",
      "age": 34
    };

    let mut qb = QB::<UserModel>::new(); // create QB instance 
    qb.insert(map, &pool).await?; // INSERT INTO users (name, age) VALUES ("Demo User", 34)

    // RETRIEVE users. You can use existing QB or create a new one.
    qb.select_all(); // SELECT * FROM users (This returns all users in Vec<UserModel>);

    // Add query modifiers to the query
    let modifiers = QueryModifiers::new()
        .with_filter(("id", 4)) // WHERE clause with equal
        .and(eq("age", 32))
        .or(eq("public_id", "some-uuid"))
        .with_limit(1); // query LIMIT (always add this if you want to call the 'select' method to retrieve a single model);

    qb.set_modifiers(modifiers); // SELECT * FROM users WHERE id = 4 AND age = 32 OR public_id = some-uuid LIMIT 1;
    qb.select(&pool).await?; // This returns a single UserModel

    // You can clear the modifiers at any time
    qb.reset_modifiers();

    // What if you only need to get specific fields of the model?
    let (id, name) = qb.select_fields(vec!["id", "name"], &pool).await?;

    // Time to UPDATE a user
    let map = query_map! {
      "name": "Updated User Name",
      "age": 52
    };

    qb.update(map, &pool).await?;

    // DELETE user
    qb.delete(&pool).await?;

    Ok(())
}
```