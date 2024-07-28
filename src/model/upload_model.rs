use crate::db::upload::upload;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Insertable, Debug)]
#[diesel(table_name = upload)]
pub struct Upload {
    pub file_name: String,
    pub file_data: Vec<u8>,
    pub userid: String,
    pub time: String,
}