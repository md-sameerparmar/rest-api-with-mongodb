use actix_web::{get, put, web, App, HttpResponse, HttpServer, Responder};
use mongodb::{bson::{doc}, options::{IndexOptions, UpdateOptions}, Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};
use futures_util::{stream::TryStreamExt};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    first_name: String,
    last_name: String,
    username: String,
    email: String,
}

const DB_NAME: &str = "myApp";
const COLL_NAME: &str = "users";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Update {
    first_name: String,
    last_name: String,
    email: String,
}

// press ctrl+enter hear -> http://127.0.0.1:8080/ to check connection
#[get("/")]
async fn start() -> HttpResponse {
    HttpResponse::Ok().body("Connected")
}

// ----------------Add a new user----------------
// write this in postmen -> http://127.0.0.1:8080/add_user
// method should be "GET"
// then select "Body -> raw -> JSON"
// then type 
// {
//     "first_name": "your first name",
//     "last_name": "your last name",
//     "username": "your username",
//     "email": "your email"
// }

#[get("/add_user")]
async fn add_user(client: web::Data<Client>, valuez: web::Json<User>) -> impl Responder {
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    // println!("{}",valuez.first_name.to_string());
    let result = collection.insert_one(valuez, None).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// ----------------Get user data----------------
// write this in postmen -> http://127.0.0.1:8080/get_user/test "instead of test write your username"
// method should be "GET"
#[get("/get_user/{username}")]
async fn get_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    match collection
        .find_one(doc! { "username": &username.to_string() }, None)
        .await
    {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// ----------------Get all user data----------------
// write this in post men or click hear-> http://127.0.0.1:8080/get_all_users
// method should be "GET"
#[get("/get_all_users")]
async fn get_all_users(client: web::Data<Client>) -> HttpResponse {
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    let cursor = collection.find(None, None).await;
    match cursor {
        Ok(cursor) => {
            let users: Vec<User> = cursor.try_collect().await.unwrap();
            // println!("print {:?}",users);
            HttpResponse::Ok().json(users)
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// ----------------UPDATE user data----------------
// write this in post men -> http://127.0.0.1:8080/update/test "instead of test write your username"
// method should be "PUT"
// then select "Body -> x-www-form-urlencoded"
// then type informationin key value pairs

//     KEY         VALUE    
//     first_name  your first name,
//     last_name   your last name,
//     username    your username,
//     email       your email

#[put("/update/{username}")]
async fn update_user(client: web::Data<Client>, username: web::Path<String>, form: web::Form<User>) -> HttpResponse {
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    let filter = doc!{"username": username.to_string()};
    let update = doc! {"$set": {"first_name": form.first_name.to_string(), "last_name": form.last_name.to_string(), "email": form.email.to_string()}};
    let options = UpdateOptions::builder().upsert(false).build();
    let result = collection.update_one(filter, update, options).await;
    match result {
            Ok(_) => HttpResponse::Ok().body("user updated"),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// ----------------DELETE user data----------------
// write this in post men or click hear -> http://127.0.0.1:8080/delete/test "instead of test write your username"
// method should be "GET"
#[get("/delete/{username}")]
async fn delete_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    match collection
        .delete_one(doc! { "username": &username.to_string() }, None)
        .await
    {
        
        Ok(_) => HttpResponse::Ok().body("User successfully deleted"),
        Err(_) => HttpResponse::InternalServerError().body("No user found for this username"),
    }
}

async fn create_username_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options)
        .build();
    client
        .database(DB_NAME)
        .collection::<User>(COLL_NAME)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    create_username_index(&client).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(start)
            .service(add_user)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .service(get_all_users)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}