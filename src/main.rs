#[macro_use] extern crate diesel;

use diesel::{RunQueryDsl, query_dsl::methods::{LimitDsl, FindDsl}};
use rocket::{catch, catchers, get, post, put, delete, routes, serde::json::{Value, serde_json::json}, Request};
use rocket_sync_db_pools::database;
use rocket::serde::json::Json;
use diesel::ExpressionMethods;

mod schema;
mod models;
mod basic_auth;
mod repositories;

use schema::products;
use models::{Product, NewProduct};
use basic_auth::BasicAuthStruct;
use repositories::ProductRepository;

#[get("/")]
async fn get_products(conn:DbConn) -> Value {
    conn.run({
        |c| {
            let results = ProductRepository::find_all(c).expect("Error listing all products");
            json!(results)
        }
    }).await
}

#[get("/<id>")]
async fn view_product(id: i32, conn: DbConn) -> Value {
    conn.run(move |c| {
            let result = products::table
            .find(id)
            .get_result::<Product>(c)
            .expect("Error finding single product");
            json!(result)
        }).await
}

#[post("/", format = "json", data = "<new_product>")]
async fn create_product(auth: BasicAuthStruct, conn:DbConn, new_product:Json<NewProduct>) -> Value {
    // test
    // print!("{} {}", auth.username, auth.password);
    conn.run({
        |c| {
            let result = diesel::insert_into(products::table)
            .values(new_product.into_inner())
            .execute(c)
            .expect("Error creating new product");
            json!(result)
        }
    }).await
}

#[put("/<id>", format = "json", data = "<product>")]
async fn update_product(id: i32, auth: BasicAuthStruct, conn: DbConn, product: Json<Product>) -> Value {
    conn.run(move |c| {
            let result = diesel::update(products::table.find(id))
            .set((
                products::name.eq(product.name.to_owned()),
                products::description.eq(product.description.to_owned()),
            ))
            .execute(c)
            .expect("Error updating product");
            json!(result)
        }).await
}

#[delete("/<id>")]
async fn delete_product(id: i32, auth: BasicAuthStruct, conn: DbConn) -> Value {
    conn.run(move |c| {
            let result = diesel::delete(products::table.find(id))
            .execute(c)
            .expect("Error deleting product");
            json!(result)
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
