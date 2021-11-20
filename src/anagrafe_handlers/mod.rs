use actix_web::{web, HttpResponse, Responder};
use bson::{doc, oid::ObjectId};
use futures::stream::StreamExt;
use mongodb::{options::FindOptions, Client};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;


#[derive(Serialize, Deserialize, )]
#[derive(Copy, Clone)]
enum STATO {
    PROVVISORIO,
    INCOMPLETO,
    VALIDATO,
    UNITO,
    ELIMINATO,
}


#[derive(Serialize, Deserialize, )]
pub struct SoggettoDTO {
    id: Option<String>,
    nome: Option<String>,
    cognome: Option<String>,
    cf: Option<String>,
    stato: STATO,
}
impl From<Soggetto> for SoggettoDTO {
    fn from(entity: Soggetto) -> Self {
        SoggettoDTO {
            id: Option::from(entity.id.to_hex()),
            nome: entity.nome,
            cognome: entity.cognome,
            cf: entity.cf,
            stato: entity.stato,
        }
    }
}


#[derive(Serialize, Deserialize, )]
pub struct Soggetto {
    #[serde(rename = "_id")]
    id: ObjectId,
    nome: Option<String>,
    cognome: Option<String>,
    cf: Option<String>,
    stato: STATO,
}


impl From<SoggettoDTO> for Soggetto {
    fn from(dto: SoggettoDTO) -> Self {
        Soggetto {
            id: ObjectId::new(),
            nome: Option::from(String::from(&dto.nome.unwrap())),
            cognome: Option::from(String::from(&dto.cognome.unwrap())),
            cf: Option::from(convert_option_string(dto.cf)),
            stato: dto.stato
        }
    }
}

const MONGO_DB: &'static str = "anagrafe";
const MONGO_COLL_LOGS: &'static str = "soggetti";

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/anagrafe/soggetti")
            .route(web::get().to(get_soggetti))
            .route(web::post().to(add_soggetto)),
    );
    cfg.service(web::resource("/anagrafe/soggetti/{id}").route(web::get().to(get_soggetto_by_id)));
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
                let entity = bson::from_bson::<Soggetto>(bson::Bson::Document(document)).unwrap();
                results.push(SoggettoDTO::from(entity));
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
                let soggetto = bson::from_bson::<Soggetto>(bson::Bson::Document(document)).unwrap();
                HttpResponse::Ok().json(SoggettoDTO::from(soggetto))
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

async fn add_soggetto(data: web::Data<Mutex<Client>>, request: web::Json<SoggettoDTO>) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    let soggetto = Soggetto::from(request.into_inner());

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