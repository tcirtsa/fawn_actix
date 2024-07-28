use redis::{Commands, RedisResult};
use serde::{Deserialize, Serialize};
use serde_json;

pub fn set_struct<T: Serialize>(
    client: &mut redis::Connection,
    key: &str,
    my_struct: &T,
    expiration: u64,
) -> RedisResult<()> {
    let serialized = serde_json::to_string(my_struct).unwrap();
    client.set_ex(key, serialized, expiration)?;
    Ok(())
}

pub fn get_struct<T: for<'de> Deserialize<'de>>(
    client: &mut redis::Connection,
    key: &str,
) -> RedisResult<Option<T>> {
    let serialized: Option<String> = client.get(key)?;
    match serialized {
        Some(data) => {
            let deserialized: T = serde_json::from_str(&data).unwrap();
            Ok(Some(deserialized))
        }
        None => Ok(None),
    }
}
