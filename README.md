# SQLX-QB

A simple query builder for [SQLx](https://github.com/launchbadge/sqlx).

If you rust SQLx, you most likely realize how great the library is. However, you have to write (and rewrite) raw queries
for every single task not matter how simple. This is where [SqlxQB](#sqlx-qb) comes in. It aims to simplify the process
of writing simple CRUD queries by providing APIs that write the queries for you and map the results to your models.

> ⚠️ Status: Early-stage project
This library is still in its early development phase and is not yet production-ready. Expect breaking changes and incomplete features.

That said, feedback, testing, and contributions are highly encouraged. If this project interests you, feel free to get involved!
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
async fn qb_demo(){
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

    let qb = QB::<UserModel>::new().insert(map); // INSERT INTO users (name, age) VALUES ("Demo User", 34)
    qb.execute(&pool).await?; // run the query
    
    // RETRIEVE users. You can use existing QB or create a new one.
    let mut qb = qb.select(); // SELECT * FROM users;
    let users = qb.fetch_all(&pool).await?; // This returns all users in Vec<UserModel>
    
    // Add query modifiers to the query
    let modifiers = QueryModifiers::new()
        .with_filter(("id", 4)) // WHERE clause with equal
        .and(eq("age", 32))
        .or(eq("public_id", "some-uuid"))
        .with_limit(1); // query LIMIT (always add this if you want to call the 'fetch' method to retrieve a single model);
    
    let qb = qb.with_modifiers(modifiers); // SELECT * FROM users WHERE id = 4 AND age = 32 OR public_id = some-uuid LIMIT 1;
    let user = qb.fetch(&pool).await?; // This returns a single UserModel
    
    // You can clear the modifiers at any time
    let qb = qb.reset_modifiers();
    
    // What if you only need to get specific fields of the model
    let qb = qb.select_fields(vec!["id", "name"]);
    let (id, name) = qb.fetch_fields(&pool).await?;
    
    // Time to UPDATE a user
    let map = query_map! {
      "name": "Updated User Name",
      "age": 52
    };
    
    let qb = qb.update(map);
    qb.execute(&pool).await;
    
    // DELETE user
    let qb = qb.delete();
    qb.execute(&pool).await;
}
```