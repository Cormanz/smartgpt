use std::collections::HashMap;

use serde::{de::{Visitor, SeqAccess, MapAccess, DeserializeSeed}, Deserializer, Deserialize, Serialize, Serializer, ser::{SerializeMap, SerializeSeq}};
use serde_json::Value;

use crate::ScriptValue;

use mlua::{ToLua, Lua, Result as LuaResult, FromLua};

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

impl ToLua<'_> for ScriptValue {
    fn to_lua(self, lua: &'_ Lua) -> LuaResult<mlua::Value> {
        match self {
            ScriptValue::String(string) => Ok(string.to_lua(lua)?),
            ScriptValue::Int(int) => Ok(int.to_lua(lua)?),
            ScriptValue::Float(float) => Ok(float.to_lua(lua)?),
            ScriptValue::Bool(bool) => Ok(bool.to_lua(lua)?),
            ScriptValue::List(list) => {
                let array = lua.create_table()?;
                for (i, value) in list.into_iter().enumerate() {
                    array.set(i + 1, value.to_lua(lua)?)?;
                }
                Ok(array.to_lua(lua)?)
            }
            ScriptValue::Dict(dict) => {
                let table = lua.create_table()?;
                for (key, value) in dict.into_iter() {
                    table.set(key, value.to_lua(lua)?)?;
                }
                Ok(table.to_lua(lua)?)
            }
            ScriptValue::None => Ok(mlua::Value::Nil),
        }
    }
}

impl<'lua> FromLua<'lua> for ScriptValue {
    fn from_lua(lua_value: mlua::Value<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            mlua::Value::String(s) => Ok(ScriptValue::String(s.to_str()?.to_owned())),
            mlua::Value::Integer(i) => Ok(ScriptValue::Int(i)),
            mlua::Value::Number(n) => Ok(ScriptValue::Float(n)),
            mlua::Value::Boolean(b) => Ok(ScriptValue::Bool(b)),
            mlua::Value::Table(table) => {
                let mut dict = HashMap::new();
                let mut array = Vec::new();
                for pair in table.pairs::<mlua::Value, mlua::Value>() {
                    let (key, value) = pair?;
                    match key {
                        mlua::Value::Integer(i) => {
                            array.resize_with(i as usize + 1, || ScriptValue::None);
                            array[i as usize] = ScriptValue::from_lua(value, lua)?;
                        }
                        mlua::Value::String(s) => {
                            let key_str = s.to_str()?.to_owned();
                            dict.insert(key_str, ScriptValue::from_lua(value, lua)?);
                        }
                        _ => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: key.type_name(),
                                to: "String or Integer",
                                message: Some("unsupported key type".to_owned()),
                            })
                        }
                    }
                }
                if !dict.is_empty() {
                    Ok(ScriptValue::Dict(dict))
                } else {
                    Ok(ScriptValue::List(array))
                }
            }
            mlua::Value::Nil => Ok(ScriptValue::None),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "ScriptValue",
                message: Some("unsupported value type".to_owned()),
            }),
        }
    }
}