mod db;
mod handler;
mod mapper;
mod model;
mod optimize;

use crate::optimize::*;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 载入.env文件中的环境变量
    dotenv().ok();

    // 从环境变量获取数据库的连接字符串
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 创建一个连接管理器
    let manager = ConnectionManager::<PgConnection>::new(&database_url);

    // 创建连接池
    let pool: DbPool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    //redis
    let client =
        redis::Client::open(env::var("REDIS_URL").expect("REDIS_URL must be set")).unwrap();
    //jwt
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());

    //开启服务器
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            .service(handler::data::upload_voice)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
