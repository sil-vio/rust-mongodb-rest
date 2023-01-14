use actix_web::{web, HttpResponse, Responder, HttpRequest};
use bson::{doc, oid::ObjectId, Document};
use futures::stream::StreamExt;
use mongodb::{options::FindOptions, Client};
use serde::{Deserialize, Serialize};
use std::{sync::Mutex};


#[derive(Serialize, Deserialize, )]
pub struct UserDTO {
    id: Option<String>,
    name: Option<String>,
    surname: Option<String>,
    cf: Option<String>,
}
impl From<User> for UserDTO {
    fn from(entity: User) -> Self {
        UserDTO {
            id: Option::from(entity.id.to_hex()),
            name: entity.name,
            surname: entity.surname,
            cf: entity.cf,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Params {
    cf: String,
}


#[derive(Serialize, Deserialize, )]
pub struct User {
    #[serde(rename = "_id")]
    id: ObjectId,
    name: Option<String>,
    surname: Option<String>,
    cf: Option<String>,
}


impl From<UserDTO> for User {
    fn from(dto: UserDTO) -> Self {
        User {
            id: ObjectId::new(),
            name: Option::from(String::from(&dto.name.unwrap())),
            surname: Option::from(String::from(&dto.surname.unwrap())),
            cf: Option::from(convert_option_string(dto.cf)),
        }
    }
}

const MONGO_DB: &'static str = "my_database";
const MONGO_COLL_LOGS: &'static str = "users";

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
            .route(web::get().to(get_soggetti))
            .route(web::post().to(add_soggetto)),
    );
    cfg.service(web::resource("/users/{id}").route(web::get().to(get_soggetto_by_id)));
}

async fn get_soggetti(    req: HttpRequest,
    data: web::Data<Mutex<Client>>) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();
    let mut filter: Document = doc! {};
    if  params.cf.capacity() > 0 {
        filter = doc! { "cf": params.cf.to_owned() } 
    } 

    let find_options = FindOptions::builder().sort(doc! { "_id": -1}).build();
    let mut cursor = logs_collection.find(filter, find_options).await.unwrap();

    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                let entity = bson::from_bson::<User>(bson::Bson::Document(document)).unwrap();
                results.push(UserDTO::from(entity));
            }
            _ => {
                return HttpResponse::InternalServerError().finish();
            }
        }
    }
    HttpResponse::Ok().json(results)
}

async fn get_soggetto_by_id(
    data: web::Data<Mutex<Client>>,
    log_id: web::Path<String>,
) -> impl Responder {
    let soggetti_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    let id = log_id.as_str();

    return match soggetti_collection
        .find_one(doc! { "_id":ObjectId::parse_str(id).unwrap()}, None)
        .await
    {
        Ok(result) => match result {
            Some(document) => {
                let soggetto = bson::from_bson::<User>(bson::Bson::Document(document)).unwrap();
                HttpResponse::Ok().json(UserDTO::from(soggetto))
            }
            None => {
                HttpResponse::NotFound().body(format!("No log found with id: {}", log_id))
            }
        },
        Err(err) => {
            println!("Failed! {}", err);
            HttpResponse::InternalServerError().finish()
        }
    };
}

async fn add_soggetto(data: web::Data<Mutex<Client>>, request: web::Json<UserDTO>) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    let soggetto = User::from(request.into_inner());

    match bson::to_bson(&soggetto) {
        Ok(model_bson) => {
            match model_bson {
                bson::Bson::Document(model_doc) => {
                    match logs_collection.insert_one(model_doc, None).await {
                        Ok(db_result) => {
                            if let Some(new_id) = db_result.inserted_id.as_object_id() {
                                println!("New document inserted with id {}", new_id);
                            }
                            return HttpResponse::Created().json(db_result.inserted_id);
                        }
                        Err(err) =>
                            {
                                println!("Failed! {}", err);
                                return HttpResponse::InternalServerError().finish();
                            }
                    }
                }
                _ => {
                    println!("Failed to create document from new model bson");
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }
        Err(err) => {
            println!("Failed to create bson from new model:\n{}", err);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

fn convert_option_string(input: Option<String>) -> String {
    match input {
        None => {
            String::from("")
        }
        Some(value) => {
            value
        }
    }
}