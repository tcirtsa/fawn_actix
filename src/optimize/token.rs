// src/jwt.rs

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: i64,
}

pub fn create_token(user_id: &str, jwt_secret: &str) -> String {
    let exp = Utc::now() + Duration::days(1); // 令牌有效期为1天
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: exp.timestamp(),
    };

    let header = Header::default();
    encode(&header, &claims, &EncodingKey::from_secret(jwt_secret.as_ref())).unwrap()
}

pub fn validate_token(token: &str, jwt_secret: &str) -> Option<Claims> {
    let validation = Validation::default();
    decode::<Claims>(token, &DecodingKey::from_secret(jwt_secret.as_ref()), &validation).ok().map(|data| data.claims)
}
