#[macro_use] extern crate diesel;

use diesel::{RunQueryDsl, query_dsl::methods::{LimitDsl, FindDsl}};
use rocket::{catch, catchers, get, post, put, delete, routes, serde::json::{Value, serde_json::json}, Request};
use rocket::request::{FromRequest, Outcome};
use rocket::http::Status;
use rocket_sync_db_pools::database;
use rocket::serde::json::Json;
use diesel::ExpressionMethods;

mod schema;
mod models;

use schema::products;
use models::{Product, NewProduct};


#[get("/")]
async fn get_products(conn:DbConn) -> Value {
    conn.run({
        |c| {
            let results = products::table.limit(100)
            .load::<Product>(c)
            .expect("Error listing all products");
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

pub struct BasicAuthStruct {
    pub username: String,
    pub password: String,
}

// Basic username:password
impl BasicAuthStruct {
    fn from_header(header: &str) -> Option<BasicAuthStruct>{
        let header_vec = header.split_whitespace().collect::<Vec<&str>>();
        if header_vec.len() != 2 {
            return None;
        }
        if header_vec[0] != "Basic" {
            return None;
        }

        // process header_vec[1]
        Self::from_base64(header_vec[1])
    }

    fn from_base64(base64_string: &str) -> Option<BasicAuthStruct> {
        let decoded = base64::decode(base64_string).ok()?;
        // convert utf8 to string
        let decoded_string = String::from_utf8(decoded).ok()?;
        // split decoded_string by :
        let decoded_vec = decoded_string.split(":").collect::<Vec<&str>>();
        if decoded_vec.len() != 2 {
            return None;
        }
        // assgin decode value to BasicAuthStruct
        Some(BasicAuthStruct {
            username: decoded_vec[0].to_string(),
            password: decoded_vec[1].to_string(),
        })
    }
}


#[rocket::async_trait]
impl <'r>FromRequest<'r> for BasicAuthStruct {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();
        let auth_header = headers.get_one("Authorization");
        if let Some(auth_header) = auth_header {
            if let Some(auth) = BasicAuthStruct::from_header(auth_header) {
                return Outcome::Success(auth);
            }
        }
        return Outcome::Failure((Status::Unauthorized, ()));
    }
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
