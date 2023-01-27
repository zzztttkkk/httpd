use std::collections::HashMap;

#[derive(Debug)]
pub struct MultiValuesMap(Option<HashMap<String, Vec<String>>>);

impl MultiValuesMap {
    pub fn new() -> Self { Self(None) }

    pub fn append(&mut self, key: &str, val: &str) {
        match &mut self.0 {
            None => {
                let mut map = HashMap::new();
                map.insert(key.to_ascii_lowercase(), vec![val.to_string()]);
                self.0 = Some(map);
            }
            Some(map) => {
                match map.get_mut(key) {
                    None => {
                        map.insert(key.to_ascii_lowercase(), vec![val.to_string()]);
                    }
                    Some(vec) => {
                        vec.push(val.to_string());
                    }
                }
            }
        }
    }

    pub fn set(&mut self, key: &str, val: &str) {
        match &mut self.0 {
            None => {
                self.append(key, val);
            }
            Some(map) => {
                match map.get_mut(key) {
                    None => {
                        map.insert(key.to_ascii_lowercase(), vec![val.to_string()]);
                    }
                    Some(vec) => {
                        vec.clear();
                        vec.push(val.to_string());
                    }
                }
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        return match &self.0 {
            None => {
                None
            }
            Some(map) => {
                match map.get(key) {
                    None => {
                        None
                    }
                    Some(vec) => {
                        vec.first()
                    }
                }
            }
        }
    }

    pub fn getall(&self, key: &str) -> Option<&Vec<String>> {
        return match &self.0 {
            None => {
                None
            }
            Some(map) => {
                match map.get(key) {
                    None => {
                        None
                    }
                    Some(vec) => {
                        Some(vec)
                    }
                }
            }
        }
    }

    pub fn getallmut(&mut self, key: &str) -> Option<&mut Vec<String>> {
        return match &mut self.0 {
            None => {
                None
            }
            Some(map) => {
                match map.get_mut(key) {
                    None => {
                        None
                    }
                    Some(vec) => {
                        Some(vec)
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, key: &str) {
        match &mut self.0 {
            None => {}
            Some(map) => {
                map.remove(key);
            }
        }
    }

    pub fn count(&self, key: &str) -> usize {
        match self.getall(key) {
            None => {
                0
            }
            Some(vec) => {
                vec.len()
            }
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        match &self.0 {
            None => {
                false
            }
            Some(map) => {
                map.contains_key(key)
            }
        }
    }
}
