use std::collections::HashMap;

use serde::{de::{Visitor, SeqAccess, MapAccess, DeserializeSeed}, Deserializer, Deserialize, Serialize, Serializer, ser::{SerializeMap, SerializeSeq}};
use serde_json::Value;

use crate::ScriptValue;

impl Serialize for ScriptValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ScriptValue::String(string) => serializer.serialize_str(string),
            ScriptValue::Int(int) => serializer.serialize_i64(*int),
            ScriptValue::Float(float) => serializer.serialize_f64(*float),
            ScriptValue::Bool(bool) => serializer.serialize_bool(*bool),
            ScriptValue::List(list) => {
                let mut seq = serializer.serialize_seq(Some(list.len()))?;
                for item in list {
                    seq.serialize_element(item)?;
                }
                seq.end()
            },
            ScriptValue::Dict(dict) => {
                let mut map = serializer.serialize_map(Some(dict.len()))?;
                for (key, value) in dict {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            },
            ScriptValue::None => serializer.serialize_none()
        }
    }
}

impl<'de> Deserialize<'de> for ScriptValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ScriptValueVisitor)
    }
}

struct ScriptValueVisitor;

impl<'de> Visitor<'de> for ScriptValueVisitor {
    type Value = ScriptValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a script value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(ScriptValue::Bool(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(ScriptValue::Int(v as i64))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(ScriptValue::Int(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
        Ok(ScriptValue::Float(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        Ok(ScriptValue::String(v.to_owned()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(ScriptValue::None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(ScriptValue::None)
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<ScriptValue, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut vec = Vec::new();

        while let Ok(Some(elem)) = visitor.next_element() {
            vec.push(elem);
        }

        Ok(ScriptValue::List(vec))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<ScriptValue, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut map = HashMap::new();

        while let Ok(Some((key, value))) = visitor.next_entry() {
            map.insert(key, value);
        }

        Ok(ScriptValue::Dict(map))
    }
}