use crate::db::upload::upload::dsl::*;
use crate::model::upload_model::Upload;
use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse};
use chrono;
use futures::TryStreamExt;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[post("/upload_voice")]
pub async fn upload_voice(
    mut payload: Multipart,
    pool: web::Data<DbPool>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let mut filedata = Vec::new();
    while let Some(mut field) = payload.try_next().await? {
        while let Some(chunk) = field.try_next().await? {
            filedata.extend_from_slice(&chunk);
        }
    }
    //以时间戳+userid命名文件+后缀
    let filename = format!(
        "{}_{}.ogg",
        chrono::Utc::now().timestamp(),
        user_id.as_str()
    );
    let new_upload = Upload {
        file_name: filename,
        file_data: filedata,
        userid: user_id.into_inner(),
        time: chrono::Utc::now().to_string(),
    };
    let result = web::block(move || {
        diesel::insert_into(upload)
            .values(&new_upload)
            .execute(&mut conn)
    })
    .await;
    match result {
        Ok(_) => Ok(HttpResponse::Ok().body("ok")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("error")),
    }
}
