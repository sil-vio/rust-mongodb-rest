use std::env;
use std::sync::*;

use actix_web::{App, HttpServer, web};
use mongodb::{Client, options::ClientOptions};

mod users_handlers;
mod time_handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "actix_web=debug");
    }
    let mongo_url = env::var("MONGO_URL").or::<String>(Result::Ok(String::from("mongodb://127.0.0.1:27017"))).unwrap();
    println!("Using connection string: {}", mongo_url);
    let mut client_options = ClientOptions::parse(&mongo_url).await.unwrap();
    client_options.app_name = Some("User".to_string());
    let client = web::Data::new(Mutex::new(Client::with_options(client_options).unwrap()));
    let no_db = web::Data::new(env::var_os("NO_DB").unwrap_or("false".into()));
    HttpServer::new(move || {
        App::new()
        .app_data(no_db.clone())
        .app_data(client.clone())
        .service(
            web::scope("/api")
                .configure(users_handlers::scoped_config)
                .configure(time_handlers::scoped_config),
        )
    })
    .bind("0.0.0.0:8088")?
    .run()
    .await
}