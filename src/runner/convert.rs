use std::collections::HashMap;

use serde::{de::{Visitor, SeqAccess, MapAccess}, Deserializer, Deserialize, Serialize, Serializer, ser::{SerializeMap, SerializeSeq}};

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

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(ScriptValue::List(values))
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut values = HashMap::new();
        while let Some((key, value)) = map.next_entry()? {
            values.insert(key, value);
        }
        Ok(ScriptValue::Dict(values))
    }
}