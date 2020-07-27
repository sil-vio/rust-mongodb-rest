use actix_web::{web, HttpResponse, Responder};
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use mongodb::{options::FindOptions, Client};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;


#[derive(Serialize, Deserialize,)]
#[derive(Copy, Clone)]
enum STATO {
    PROVVISORIO,
    INCOMPLETO,
    VALIDATO,
    UNITO,
    ELIMINATO
}



#[derive(Serialize, Deserialize,)]
pub struct SoggettoDTO {
    id: Option<String>,
    nome: String,
    cognome: String,
    cf: Option<String>,
    stato: STATO,
}


#[derive(Serialize, Deserialize,)]
pub struct Soggetto {
    #[serde(rename = "_id")]
    id: ObjectId,
    nome: Option<String>,
    cognome: Option<String>,
    cf: Option<String>,
    stato: STATO,
}

const MONGO_DB: &'static str = "anagrafe";
const MONGO_COLL_LOGS: &'static str = "soggetti";

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/soggetti")
            .route(web::get().to(get_soggetti))
            .route(web::post().to(add_soggetto)),
    );
    cfg.service(web::resource("/soggetti/{id}").route(web::get().to(get_soggetto_by_id)));
}

async fn get_soggetti(data: web::Data<Mutex<Client>>) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    let filter = doc! {};
    //   let find_options = FindOptions::builder().sort(doc! { "createdOn": 1 }).build();
    let find_options = FindOptions::builder().sort(doc! { "_id": -1}).build();
    let mut cursor = logs_collection.find(filter, find_options).await.unwrap();

    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                results.push(entity_to_dto(bson::from_bson(bson::Bson::Document(document)).unwrap()));
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
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    let id = log_id.as_str();

    match logs_collection
        .find_one(doc! { "_id":ObjectId::with_string(id).unwrap()}, None)
        .await
    {
        Ok(result) => match result {
            Some(document) => {
                return HttpResponse::Ok().json(entity_to_dto(bson::from_bson(bson::Bson::Document(document)).unwrap()));
            },
            None => {
                return HttpResponse::NotFound().body(format!("No log found with id: {}", log_id))
            }
        },
        Err(err) => {
            println!("Failed! {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

async fn add_soggetto(data: web::Data<Mutex<Client>>, request: web::Json<SoggettoDTO>) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);
    
        let soggetto = dto_to_entity(request.into_inner());

        match bson::to_bson(&soggetto) {
            Ok(model_bson) => {
                match model_bson{
                    bson::Bson::Document(model_doc) => {
                        match logs_collection.insert_one( model_doc, None).await {
                            Ok(db_result) => {
                                if let Some(new_id) = db_result.inserted_id.as_object_id() {
                                    println!("New document inserted with id {}", new_id);   
                                }
                                return HttpResponse::Created().json(db_result.inserted_id)
                            }
                            Err(err) =>
                            {
                                println!("Failed! {}", err);
                                return HttpResponse::InternalServerError().finish()
                            }
                        }
                    },
                    _ => {
                        println!("Failed to create document from new model bson");
                        return HttpResponse::InternalServerError().finish()
                    }
                }
            },
            Err(err) => {
                println!("Failed to create bson from new model:\n{}",err);
                return HttpResponse::InternalServerError().finish()
            }
        }

   /*  match logs_collection.insert_one(doc! {"deviceId": &soggetto.id, "message": &new_log.message, "createdOn": Bson::DateTime(Utc::now())}, None).await {
        Ok(db_result) => {
            if let Some(new_id) = db_result.inserted_id.as_object_id() {
                println!("New document inserted with id {}", new_id);   
            }
            return HttpResponse::Created().json(db_result.inserted_id)
        }
        Err(err) =>
        {
            println!("Failed! {}", err);
            return HttpResponse::InternalServerError().finish()
        }
    } */
}

fn dto_to_entity(dto: SoggettoDTO) -> Soggetto {

   
    return Soggetto{
        id: ObjectId::new(),
        nome: Option::from(String::from(&dto.nome)),
        cognome:  Option::from(String::from(&dto.cognome)),
        cf: Option::from(convert_option_string(dto.cf)),
        stato: dto.stato,
    };
}

fn entity_to_dto(entity: Soggetto) -> SoggettoDTO {
    return SoggettoDTO{
        id: Option::from(entity.id.to_hex()),
        nome: convert_option_string(entity.nome),
        cognome:  convert_option_string(entity.cognome),
        cf: Option::from(convert_option_string(entity.cf)),
        stato: entity.stato,
    };
}

fn convert_option_string(input: Option<String>) -> String {
    match input {
        None => {
            String::from("")
        },
        Some(value) => {
            String::from(value)
        }
    }
}