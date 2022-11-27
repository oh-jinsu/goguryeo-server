use serde::{Deserialize, de};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct Vector3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vector3 {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Vector3 { x, y, z }
    }
}

impl<'de> Deserialize<'de> for Vector3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_str(Visitor)
    }
}

struct Visitor;

impl Visitor {
    fn parse_int<E>(values: &Vec<&str>, index: usize) -> Result<i32, E>
    where E: serde::de::Error {
        match values.get(index) {
            Some(x) => match x.parse::<i32>() {
                Ok(x) => Ok(x),
                Err(e) => Err(serde::de::Error::custom(e)),
            },
            None => Err(serde::de::Error::custom("not enough arguments")),
        }
    }
}

impl<'de> de::Visitor<'de> for Visitor {
    type Value = Vector3;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("the value should be string like x,y,z")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        let values: Vec<&str> = v.split(",").collect();

        let x = Visitor::parse_int(&values, 0)?;

        let y = Visitor::parse_int(&values, 1)?;

        let z = Visitor::parse_int(&values, 2)?;

        Ok(Vector3 { x, y, z })
    }
}