# SQLX-QB

A simple query builder for [SQLx](https://github.com/launchbadge/sqlx).

If you use rust SQLx, you most likely realize how great the library is. However, you have to write (and rewrite) raw
queries for every single task no matter how simple. This is where [SqlxQB](#sqlx-qb) comes in. It aims to simplify the
process
of writing simple CRUD queries by providing APIs that write the queries for you and map the results to your models.

> **⚠️ Status: Early-stage project**<br>
> This library is still in its early development phase and is not yet production-ready. Expect breaking changes and
> incomplete features.

That said, feedback, testing, and contributions are highly encouraged. If this project interests you, feel free to get
involved!

## Installation

Add it to your Cargo.toml

```toml
sqlx_qb = { git = "https://github.com/dududaa/sqlx-qb" } 
```

## Usage

You can start using Sqlx-QB in two simple steps:

#### 1. Derive `Model` and `FromRow` for your model.

```rust
use sqlx_qb::prelude::*;

/// (Optionally) provide the table name, otherwise the model's identifier will be used in snake_case, 
/// e.g, "user_model" here.
///
/// (Optionally) provide the table's primary column. This is especially useful to call `get` method which retrieves 
/// just one row of the table. The default primary_column is "id".
#[derive(Model, FromRow)]
#[model(table_name = "users")]
#[model(primary_column = "id")]
struct UserModel {
    id: i32,
    name: String,
    age: i16,
    public_id: Uuid,
    created_at: Datetime<Utc>
}
```

#### 2. Create `QB` instance and starting using the interfaces.

```rust
/// This is optional depending on your query. See below.
const TABLE_NAME: &'static str = "users";

async fn main() -> anyhow::Result<()> {
    // create SQLx pool by yourself.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    let mut qb = QB::<UserModel>::new(&pool).with_table_name(TABLE_NAME); // Create mutable QB instance and optionally set the table name.
}
```

###### INSERT models

```rust
async fn main() -> anyhow::Result<()> {
    // ...
    // Define the input map.
    let map = query_map! { TABLE_NAME, // optionally pass table_name
      "name": "Demo User",
      "age": 34
    };

    qb.insert(&map).await?; // INSERT INTO users (name, age) VALUES ("Demo User", 34)
    let id = qb.insert_returns(&map, "id").await?; // Insert and return a field IF YOUR DATABASE SUPPORTS RETURNING.
```
If you enable  `serde` feature, you can also pass any struct that implements `serde::Serialize` and `ModelInsert`:
```rust
#[derive(ModelInsert, Serialize)]
// #[model(table_name = "users")]
// #[model(insert_returns = "i32")] forces type of field to return on insertion.
struct InsertArg {
    name: String,
    age: i32,
}

async fn main() -> anyhow::Result<()> {
    // ...
        
    let map = InsertArg {
        name: "Demo User".to_string(),
        age: 34,
    };

    qb.insert(&map).await?;
    let id = qb.insert_returns(&map, "id").await?;
}
```

###### RETRIEVE models
To retrieve a model, make sure the struct implements `sqlx::From` and `Model`.
```rust
#[derive(Model, FromRow)]
#[model(table_name = "users")]
struct UserModel {
    name: String,
    age: i32
}

async fn main() -> anyhow::Result<()> {
    // ...
    let users: Vec<UserModel> = qb.select_all().await?; // SELECT * FROM users (This returns all users in Vec<UserModel>);

    // Add query modifiers to the query
    let modifiers = Modifiers::new()
        .with_filter(("id", 4)) // WHERE clause with equal
        .and(eq("age", 32))
        .or(eq("public_id", "some-uuid"))
        .with_limit(1); // query LIMIT (always add this if you want to call the 'select' method to retrieve a single model);

    qb.set_modifiers(&modifiers); // SELECT * FROM users WHERE id = 4 AND age = 32 OR public_id = some-uuid LIMIT 1;
    let user: UserModel = qb.select().await?; // This returns a single UserModel

    // What if you only need to get specific fields of the model?
    let total_rows = qb.select_scalar("Count(*)").await?; // select one field
    let (id, name) = qb.select_fields(["id", "name"]).await?; // select multiple fields

    // There's a simple get method that simply retrieves one row using the specified primary column.
    let user = qb.get().await?;
}
```

###### More on modifiers

Be careful with Modifiers! The same modifiers will be used across multiple operations of **the same** `QB` instance. If
you're unsure whether current modifiers match your current query, you can either `reset_modifiers` to remove them or
update
them with `set_modifiers`.

```rust
async fn main() -> anyhow::Result<()> {
    // ...
    // You can clear the modifiers at any time
    qb.reset_modifiers();

    // Or set new modifiers
    qb.set_modifiers(&modifiers);
}
```

###### UPDATE models

```rust
async fn main() -> anyhow::Result<()> {
    // ...
    // Time to UPDATE a user
    let map = query_map! {
      "name": "Updated User Name",
      "age": 52
    };
    
    qb.update(&map).await?;
}
```

###### DELETE models

```rust
async fn main() -> anyhow::Result<()> {
    // ...
    // DELETE user
    qb.delete().await?;
}
```
