use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::Connection;
use sqlx::Row;
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

fn value_to_entity(value: Value) -> Option<Entity> {
    let obj = value.as_object()?;
    let id = obj.get("id")?.as_str()?.to_string();
    let attrs = obj.get("attrs")?.as_object()?.clone();
    Some(Entity { id, attrs })
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

/// load_entity() loads an Entity from the vaults table by its entity ID.
/// If the entity is not found, it returns an empty Entity with the given ID.
/// # Arguments
/// * `conn` - A mutable reference to a SqliteConnection.
/// * `entity_id` - A string slice representing the entity ID.
/// # Errors
/// Returns an error if the SQL query fails or if decoding fails.
async fn load_entity(conn: &mut SqliteConnection, entity_id: &str) -> Result<Entity, sqlx::Error> {
    let addr = addr_for_entity_id(entity_id);
    let row = sqlx::query("SELECT content FROM vaults WHERE addr = ?;")
        .bind(addr)
        .fetch_optional(conn)
        .await?;

    if let Some(row) = row {
        let content: String = row.get("content");
        if let Ok(value) = transit::decode_value(&content) {
            if let Some(entity) = value_to_entity(value) {
                return Ok(entity);
            }
        }
    }

    Ok(Entity {
        id: entity_id.to_string(),
        attrs: Map::new(),
    })
}

async fn scan_entities() -> Result<Vec<Entity>, sqlx::Error> {
    // TODO@chico: implement scanning all entities from the vaults table.
    unimplemented!()
}

//// apply_datoms() applies a list of Datoms to the database.
/// It groups the datoms by entity ID, loads each entity, applies the datoms,
/// and writes the updated entity back to the database.
/// Returns a Result containing a vector of updated Entities or an error.
///
/// # Arguments
/// * `datoms` - A slice of Datoms to apply.
///     - Each Datom specifies an operation (Add or Retract), an entity ID (e),
///     an attribute (a), and a value (v).
///     - For Add operations, the attribute-value pair is added to the entity.
///     - For Retract operations, the attribute is removed from the entity.
///     - If an entity does not exist, it is created with the given ID.
///     - After applying all datoms, the updated entities are written back to the database.
///     - The function returns a vector of the updated entities.
///     - Datom examples:
///     - Add datom: Datom { op: DatomOp::Add, e: "block:page-id".to_string(), a: "block/title".to_string(), v: Value::String("Journal".to_string()) }
/// # Errors
/// Returns an error if any SQL query fails or if loading/writing entities fails.
pub async fn apply_datoms(datoms: &[Datom]) -> Result<Vec<Entity>, sqlx::Error> {
    // we are assuming the db was create and schema ensured at startup.
    // TODO@chico: can improve the apply_datoms performance by batching the writes in a transaction.
    // TODO@chico: this functions can receive a &mut SqliteConnection to avoid opening a new connection each time.
    let mut conn = crate::db::conn().await;

    let mut grouped: HashMap<String, Vec<&Datom>> = HashMap::new();
    for datom in datoms {
        grouped.entry(datom.e.clone()).or_default().push(datom);
    }
    let mut updated = Vec::with_capacity(grouped.len());
    for (entity_id, entity_datoms) in grouped {
        let mut entity = load_entity(&mut conn, &entity_id).await?;
        for datom in entity_datoms {
            match datom.op {
                DatomOp::Add => {
                    entity.attrs.insert(datom.a.clone(), datom.v.clone());
                }
                DatomOp::Retract => {
                    entity.attrs.remove(&datom.a);
                }
            }
        }
        write_entity(&mut conn, &entity).await?;
        updated.push(entity);
    }

    let _ = conn.close().await;
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use crate::db::init_db;

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

    //TODO@chico: reuse this function
    fn unique_test_db_path(test_name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let pid = std::process::id();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("jottty_{}_{}_{}.db", test_name, pid, ts));
        path
    }

    #[tokio::test]
    async fn test_apply_datoms() {
        let db_path = unique_test_db_path("apply_datoms");
        unsafe {
            std::env::set_var("JOTTTY_DB_PATH", db_path.to_string_lossy().to_string());
        }
        init_db().await;

        let datoms = vec![
            Datom {
                op: DatomOp::Add,
                e: "block:page-id".to_string(),
                a: "block/title".to_string(),
                v: Value::String("Journal".to_string()),
            },
            Datom {
                op: DatomOp::Add,
                e: "block:page-id".to_string(),
                a: "block/content".to_string(),
                v: Value::String("This is a bullet in the journal".into()),
            },
        ];
        let result = apply_datoms(&datoms).await;

        for r in result.as_ref().unwrap() {
            if r.id == "block:page-id" {
                assert_eq!(
                    r.attrs.get("block/title").unwrap(),
                    &Value::String("Journal".to_string())
                );
                assert_eq!(
                    r.attrs.get("block/content").unwrap(),
                    &Value::String("This is a bullet in the journal".into())
                );
            }
        }
        assert!(result.is_ok());
    }
}
