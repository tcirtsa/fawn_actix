use crate::db::a::a::dsl::*;
use crate::{model::a_model::A, redis_fn, DbPool};
use actix_multipart::Multipart;
use actix_web::{get, web, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use futures::{TryStreamExt, StreamExt};

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

#[get("/test")]
pub async fn test(pool: web::Data<DbPool>, client: web::Data<redis::Client>) -> impl Responder {
    let mut redis_conn = client.get_connection().unwrap();
    let results: Option<A> = redis_fn::get_struct(&mut redis_conn, "a").unwrap();
    match results {
        Some(data) => HttpResponse::Ok().json(data),
        None => {
            let result = web::block(move || {
                let mut conn = pool.get().expect("couldn't get db connection from pool");
                a.load::<A>(&mut conn)
            })
            .await
            .unwrap();
            match result {
                Ok(data) => {
                    redis_fn::set_struct(&mut redis_conn, "a", &data, 3600).unwrap();
                    HttpResponse::Ok().json(data)
                }
                Err(_) => HttpResponse::InternalServerError().body("error"),
            }
        }
    }
}

#[get("/test2")]
pub async fn test2(pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let results = a.filter(account.eq("123")).load::<A>(&mut conn).unwrap();
    if verify("1234", results[0].psd.as_str()).unwrap() {
        HttpResponse::Ok().body("ok")
    } else {
        HttpResponse::Ok().body("error")
    }
}

#[get("/test3")]
async fn save_file(pool: web::Data<DbPool>, mut payload: Multipart) -> impl Responder {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut bytes = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            bytes.extend_from_slice(&data);
        }

        // 将文件数据插入数据库
        let new_file = A {
            account: "123".to_string(),
            psd: hash_password("123"),
            file: bytes,
        };

        let mut conn = pool.get().expect("couldn't get db connection from pool");

        diesel::insert_into(a)
            .values(&new_file)
            .execute(&mut conn)
            .expect("Error saving new file");
    }

    HttpResponse::Ok().body("File uploaded successfully")
}
