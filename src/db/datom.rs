use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::SqliteConnection;

use crate::db::transit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub attrs: Map<String, Value>,
}

#[derive(Debug, Clone, Copy)]
pub enum DatomOp {
    Add,
    Retract,
}

pub struct Datom {
    pub op: DatomOp,
    pub e: String,
    pub a: String,
    pub v: Value,
}

/// FNV-1a 64-bit hash function
/// See: https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
/// That block computes a stable address (addr) for each entity ID so it can be stored in the vaults table:
fn fnv1a_hash64(input: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// addr_for_entity_id() converts an entity ID string (like block:...) into an i64 address.
fn addr_for_entity_id(entity_id: &str) -> i64 {
    fnv1a_hash64(entity_id) as i64
}

fn entity_to_value(entity: &Entity) -> Value {
    let mut map = Map::new();
    map.insert("id".to_string(), Value::String(entity.id.clone()));
    map.insert("attrs".to_string(), Value::Object(entity.attrs.clone()));
    Value::Object(map)
}

/// write_entity() writes an Entity into the vaults table.
/// It encodes the entity as a transit value and stores it in the content column.
/// The addresses column is set to an empty array for now.
///
/// Returns Result<(), sqlx::Error>
/// # Arguments
/// * `conn` - A mutable reference to a SqliteConnection.
/// * `entity` - A reference to the Entity to write.
/// # Errors
/// Returns an error if the SQL query fails or if encoding fails.
async fn write_entity(conn: &mut SqliteConnection, entity: &Entity) -> Result<(), sqlx::Error> {
    let addr = addr_for_entity_id(&entity.id);
    let content = match transit::encode_value(&entity_to_value(entity)) {
        Ok(value) => value,
        Err(err) => return Err(sqlx::Error::Protocol(err.to_string())),
    };
    let addresses = "[]";

    sqlx::query(
        "INSERT INTO vaults (addr, content, addresses)
        VALUES (?, ?, ?)
        ON CONFLICT(addr) DO UPDATE SET content = excluded.content, addresses = excluded.addresses;",
    )
    .bind(addr)
    .bind(content)
    .bind(addresses)
    .execute(conn)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a_hash64() {
        let input = "block:0000000000000000000a7b3c4d5e6f7g8h9i0jklmnopqrstuvwx";
        let hash = fnv1a_hash64(input);
        assert_eq!(hash, 16887854668524219895); // Example expected hash value
    }

    #[test]
    fn test_addr_for_entity_id() {
        let entity_id = "block:0000000000000000000a7b3c4d5e6f7g8h9i0jklmnopqrstuvwx";
        let addr = addr_for_entity_id(entity_id);
        assert_eq!(addr, -1558889405185331721); // Example expected address value
    }

    #[test]
    fn test_entity_to_value() {
        let mut attrs = Map::new();
        attrs.insert(
            "block/title".to_string(),
            Value::String("Journal".to_string()),
        );
        attrs.insert(
            "block/content".to_string(),
            Value::String("This is a bullet in the journal".into()),
        );

        let entity = Entity {
            id: "block:page-id".to_string(),
            attrs,
        };

        let value = entity_to_value(&entity);

        //TODO@chico: improve this test to cover more edge cases.
        assert!(value.is_object());
        let obj = value.as_object().unwrap();
        assert_eq!(
            obj.get("id").unwrap(),
            &Value::String("block:page-id".to_string())
        );
        let attrs_value = obj.get("attrs").unwrap();
        assert!(attrs_value.is_object());
        let attrs_obj = attrs_value.as_object().unwrap();
        assert_eq!(
            attrs_obj.get("block/title").unwrap(),
            &Value::String("Journal".to_string())
        );
        assert_eq!(
            attrs_obj.get("block/content").unwrap(),
            &Value::String("This is a bullet in the journal".into())
        );
    }
}
