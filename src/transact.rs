use crate::db::{self, Datom, DatomOp, Entity};
use serde_json::Value;

fn normalize_op(op: &str) -> Option<DatomOp> {
    let op = op.trim_start_matches(':');
    match op {
        "db/add" => Some(DatomOp::Add),
        "db/retract" => Some(DatomOp::Retract),
        _ => None,
    }
}

fn datom_from_value(value: Value) -> Result<Datom, String> {
    let list = value
        .as_array()
        .ok_or_else(|| "Datom must be an array".to_string())?;
    if list.len() != 4 {
        return Err("Datom list must have 4 items".to_string());
    }
    let op = list[0]
        .as_str()
        .ok_or_else(|| "Datom op must be a string".to_string())
        .and_then(|op| normalize_op(op).ok_or_else(|| format!("Unsupported op: {}", op)))?;
    let e = list[1]
        .as_str()
        .ok_or_else(|| "Datom e must be a string".to_string())?
        .to_string();
    let a = list[2]
        .as_str()
        .ok_or_else(|| "Datom a must be a string".to_string())?
        .to_string();
    let v = list[3].clone();
    Ok(Datom { op, e, a, v })
}

pub async fn transact_with_fallback(datoms: Vec<Value>) -> Result<Vec<Entity>, String> {
    let mut parsed = Vec::with_capacity(datoms.len());
    for value in datoms {
        parsed.push(datom_from_value(value)?);
    }
    db::apply_datoms(&parsed)
        .await
        .map_err(|err| err.to_string())
}
