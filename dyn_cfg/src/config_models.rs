use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MappingConfigWithDefault<K, T>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    mapping: HashMap<K, T>,
    #[serde(rename = "default")]
    default_value: T,
}

impl<K, T> MappingConfigWithDefault<K, T>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    pub fn new(mapping: HashMap<K, T>, default_value: T) -> Self {
        Self {
            mapping,
            default_value,
        }
    }

    pub fn default(default_value: T) -> Self {
        Self {
            mapping: HashMap::new(),
            default_value,
        }
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> &T
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        self.mapping.get(k).unwrap_or(&self.default_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_config_with_default() {
        let json = r#"{"mapping":{"1": true, "2": false}, "default": true}"#;
        let res: Result<MappingConfigWithDefault<u32, bool>, _> = serde_json::from_str(json);
        match res {
            Ok(v) => {
                println!("Success: {:?}", v);
                assert_eq!(v.default_value, true);
                assert_eq!(v.mapping.get(&1), Some(&true));
                assert_eq!(v.mapping.get(&2), Some(&false));
            }
            Err(e) => {
                println!("Error: {}", e);
                panic!("failed to parse: {}", e);
            }
        }
    }
}
