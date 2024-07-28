use crate::db::a::a::dsl::*;
use crate::{model::a_model::A, redis_fn, DbPool};
use actix_web::{get, web, HttpResponse, Responder};
use diesel::prelude::*;

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
