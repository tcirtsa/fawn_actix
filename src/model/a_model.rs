use crate::db::a::a;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Insertable, Deserialize, Debug)]
#[diesel(table_name = a)]
pub struct A {
    pub account: String,
    pub psd: String,
}
