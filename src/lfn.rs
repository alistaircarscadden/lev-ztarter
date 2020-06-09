use serde::{
    de::{Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
use std::{
    ffi::OsStr,
    fmt::{Display, Formatter},
    str::from_utf8,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LevelFileName {
    pub data: [u8; 12],
    pub len: usize,
}
impl LevelFileName {
    fn new() -> Self {
        Self {
            data: [0; 12],
            len: 0,
        }
    }

    pub fn from_str(file_name: &str) -> Self {
        let copy_len;
        Self {
            data: {
                let mut data = [0u8; 12];
                let bytes = file_name.as_bytes();
                copy_len = std::cmp::min(12, bytes.len());
                data[..copy_len].copy_from_slice(&bytes[..copy_len]);
                data
            },
            len: copy_len,
        }
    }

    pub fn as_str(&self) -> &str {
        from_utf8(&self.data[..self.len]).unwrap()
    }

    pub fn from_osstr(file_name: &OsStr) -> Self {
        Self::from_str(file_name.to_str().unwrap())
    }

    pub fn fmt_level_name(pre: &str, pad: usize, num: i32) -> Self {
        Self::from_str(&Self::fmt_level_name_string(pre, pad, num))
    }

    pub fn fmt_level_name_string(pre: &str, pad: usize, num: i32) -> String {
        let mut s = String::new();
        let num = format!("{}", num);
        s.push_str(pre);
        for _ in 0..pad - num.len() {
            s.push('0');
        }
        s.push_str(&num);
        s.push_str(".lev");
        s
    }
}
impl Display for LevelFileName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::as_str(self))
    }
}
impl Serialize for LevelFileName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
impl<'de> Deserialize<'de> for LevelFileName {
    fn deserialize<D>(deserializer: D) -> Result<LevelFileName, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LevelFileNameVisitor;

        impl<'de> Visitor<'de> for LevelFileNameVisitor {
            type Value = LevelFileName;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a string that can be converted to [u8; 12]")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(LevelFileName::from_str(value))
            }
        }

        deserializer.deserialize_bytes(LevelFileNameVisitor)
    }
}
