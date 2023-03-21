#[macro_use] extern crate diesel;

use rocket::{catch, catchers, get, post, put, delete, routes, serde::json::{Value, serde_json::json}};
use rocket_sync_db_pools::database;
use rocket::serde::json::Json;
use rocket::response::status;
use rocket::http::Status;

mod schema;
mod models;
mod basic_auth;
mod repositories;

use models::{Product, NewProduct};
use basic_auth::BasicAuthStruct;
use repositories::ProductRepository;

#[get("/")]
async fn get_products(conn:DbConn) -> Result<Value, status::Custom<Value>> {
    conn.run({
        |c| {
             ProductRepository::find_all(c).
                map(|products| json!(products)).
                map_err(|e| status::Custom(Status::InternalServerError, json!({"error": e.to_string()})))
        }
    }).await
}

#[get("/<id>")]
async fn view_product(id: i32, conn: DbConn) -> Result<Value, status::Custom<Value>> {
    conn.run(move |c| {
            ProductRepository::find(c, id).
                map(|product| json!(product)).
                map_err(|e| status::Custom(Status::InternalServerError, json!({"error": e.to_string()})))
        }).await
}

#[post("/", format = "json", data = "<new_product>")]
async fn create_product(auth: BasicAuthStruct, conn:DbConn, new_product:Json<NewProduct>) -> Result<Value, status::Custom<Value>> {
    // test
    // print!("{} {}", auth.username, auth.password);
    conn.run({
        |c| {
            ProductRepository::create(c, new_product.into_inner())
                .map(|product| json!(product))
                .map_err(|e| status::Custom(Status::InternalServerError, json!({"error": e.to_string()})))
        }
    }).await
}

#[put("/<id>", format = "json", data = "<product>")]
async fn update_product(id: i32, auth: BasicAuthStruct, conn: DbConn, product: Json<Product>) -> Result<Value, status::Custom<Value>> {
    conn.run(move |c| {
            ProductRepository::save(c, product.into_inner())
                .map(|product| json!(product))
                .map_err(|e| status::Custom(Status::InternalServerError, json!({"error": e.to_string()})))
        }).await
}

#[delete("/<id>")]
async fn delete_product(id: i32, auth: BasicAuthStruct, conn: DbConn) -> Result<Value, status::Custom<Value>> {
    conn.run(move |c| {
            ProductRepository::delete(c, id)
                .map(|count| json!(count))
                .map_err(|e| status::Custom(Status::InternalServerError, json!({"error": e.to_string()})))
        }).await
}

#[catch(404)]
async fn not_found_url() -> Value {
    json!("Not found url")
}

#[database("sqlite_path")]
struct DbConn(diesel::SqliteConnection);

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rocket::build()
    .mount("/product", routes![get_products, 
                               view_product, 
                               create_product, 
                               update_product, 
                               delete_product])
    .register("/", catchers!(not_found_url))
    .attach(DbConn::fairing())
    .launch().await?;
    Ok(())
}
