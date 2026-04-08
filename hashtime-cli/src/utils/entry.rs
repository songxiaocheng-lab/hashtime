use hashtime::FileHashTimeResult;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct HashTimeEntry {
    pub path: String,
    pub size: Option<u64>,
    pub birthtime_ns: Option<i64>,
    pub mtime_ns: Option<i64>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
}

impl serde::Serialize for HashTimeEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("HashTimeEntry", 9)?;
        state.serialize_field("path", &self.path)?;
        state.serialize_field("size", &self.size)?;
        if let Some(ref v) = self.birthtime_ns {
            state.serialize_field("birthtime_ns", v)?;
        }
        if let Some(ref v) = self.mtime_ns {
            state.serialize_field("mtime_ns", v)?;
        }
        if let Some(ref v) = self.md5 {
            state.serialize_field("md5", v)?;
        }
        if let Some(ref v) = self.sha1 {
            state.serialize_field("sha1", v)?;
        }
        if let Some(ref v) = self.sha256 {
            state.serialize_field("sha256", v)?;
        }
        if let Some(ref v) = self.sha512 {
            state.serialize_field("sha512", v)?;
        }
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for HashTimeEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;
        struct HashTimeEntryVisitor;

        impl<'de> serde::de::Visitor<'de> for HashTimeEntryVisitor {
            type Value = HashTimeEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a HashTimeEntry struct")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut path = None;
                let mut size = None;
                let mut birthtime_ns = None;
                let mut mtime_ns = None;
                let mut md5 = None;
                let mut sha1 = None;
                let mut sha256 = None;
                let mut sha512 = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "path" => path = Some(map.next_value()?),
                        "size" => size = Some(map.next_value()?),
                        "birthtime_ns" => birthtime_ns = map.next_value()?,
                        "mtime_ns" => mtime_ns = map.next_value()?,
                        "md5" => md5 = map.next_value()?,
                        "sha1" => sha1 = map.next_value()?,
                        "sha256" => sha256 = map.next_value()?,
                        "sha512" => sha512 = map.next_value()?,
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                Ok(HashTimeEntry {
                    path: path.unwrap_or_default(),
                    size,
                    birthtime_ns,
                    mtime_ns,
                    md5,
                    sha1,
                    sha256,
                    sha512,
                })
            }
        }

        deserializer.deserialize_map(HashTimeEntryVisitor)
    }
}

pub fn file_hash_time_result_to_entry(
    r: &FileHashTimeResult,
    hash_fields_set: &HashSet<&String>,
    time_fields_set: &HashSet<&String>,
) -> HashTimeEntry {
    HashTimeEntry {
        path: r.path.to_string_lossy().to_string(),
        size: r.size,
        birthtime_ns: if time_fields_set.contains(&"birthtime".to_string()) {
            r.created_ns
        } else {
            None
        },
        mtime_ns: if time_fields_set.contains(&"mtime".to_string()) {
            r.modified_ns
        } else {
            None
        },
        md5: if hash_fields_set.contains(&"md5".to_string()) {
            r.md5.clone()
        } else {
            None
        },
        sha1: if hash_fields_set.contains(&"sha1".to_string()) {
            r.sha1.clone()
        } else {
            None
        },
        sha256: if hash_fields_set.contains(&"sha256".to_string()) {
            r.sha256.clone()
        } else {
            None
        },
        sha512: if hash_fields_set.contains(&"sha512".to_string()) {
            r.sha512.clone()
        } else {
            None
        },
    }
}
