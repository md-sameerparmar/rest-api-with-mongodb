use actix_web::{get, put, patch, web::{self, Form}, App, HttpResponse, HttpServer, Responder};
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel, results::DeleteResult};
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

#[get("/")]
async fn start() -> HttpResponse {
    HttpResponse::Ok().body("Connected")
}

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

#[get("/get_all_users")]
async fn get_all_users(client: web::Data<Client>) -> HttpResponse {
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    let cursor = collection.find(None, None).await;
    match cursor {
        Ok(cursor) => {
            let users: Vec<User> = cursor.try_collect().await.unwrap();
            println!("print {:?}",users);
            HttpResponse::Ok().json(users)
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[patch("/update/{username}")]
async fn update_user(client: web::Data<Client>,updt: web::Form<Update>, username: web::Path<String>) -> impl Responder {
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    
    let updt_val = doc! { "$set": {
        "first_name": &updt.first_name,
        "last_name": &updt.last_name,
        "email": &updt.email
    }
    };

    match collection.update_one(doc! { "username": &username.to_string()},updt_val, None).await {
        Ok(_) => HttpResponse::Ok().body("user updated"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
    // let result = collection.update_one({"username": &username.to_string()}, None).await;
    // match update_one(doc! {"username": &username.to_string()}, None).await {
    //     Ok(_) => HttpResponse::Ok().body("user updated"),
    //     Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    // }
    
}

#[get("/delete_user/{username}")]
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
            .service(delete_user)
            .service(get_all_users)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}