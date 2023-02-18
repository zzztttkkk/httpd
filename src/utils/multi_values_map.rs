use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct MultiValuesMap {
    pub(crate) _map: Option<HashMap<String, Vec<String>>>,
    pub(crate) case_sensitive: bool,
}

impl MultiValuesMap {
    pub fn map(&self) -> Option<&HashMap<String, Vec<String>>> {
        self._map.as_ref()
    }

    pub fn clear(&mut self) {
        if let Some(map) = &mut self._map {
            map.clear();
        }
    }

    pub fn len(&self) -> usize {
        match &self._map {
            Some(map) => map.len(),
            None => 0,
        }
    }

    fn _do_append(&mut self, key: &str, val: &str) {
        match &mut self._map {
            None => {
                let mut map = HashMap::new();
                map.insert(key.to_string(), vec![val.to_string()]);
                self._map = Some(map);
            }
            Some(map) => match map.get_mut(key) {
                None => {
                    map.insert(key.to_string(), vec![val.to_string()]);
                }
                Some(vec) => {
                    vec.push(val.to_string());
                }
            },
        }
    }

    pub fn append(&mut self, key: &str, val: &str) {
        if self.case_sensitive {
            self._do_append(key, val);
            return;
        }
        self._do_append(key.to_ascii_lowercase().as_str(), val);
    }

    pub fn set(&mut self, key: &str, val: &str) {
        let k: String;
        if self.case_sensitive {
            k = key.to_string();
        } else {
            k = key.to_ascii_lowercase();
        }
        let key = k.as_str();

        match &mut self._map {
            None => {
                self._do_append(key, val);
            }
            Some(map) => match map.get_mut(key) {
                None => {
                    map.insert(k, vec![val.to_string()]);
                }
                Some(vec) => {
                    vec.clear();
                    vec.push(val.to_string());
                }
            },
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        return match &self._map {
            None => None,
            Some(map) => match map.get(key) {
                None => None,
                Some(vec) => vec.first(),
            },
        };
    }

    pub fn get_all(&self, key: &str) -> Option<&Vec<String>> {
        return match &self._map {
            None => None,
            Some(map) => match map.get(key) {
                None => None,
                Some(vec) => Some(vec),
            },
        };
    }

    pub fn get_all_mut(&mut self, key: &str) -> Option<&mut Vec<String>> {
        return match &mut self._map {
            None => None,
            Some(map) => match map.get_mut(key) {
                None => None,
                Some(vec) => Some(vec),
            },
        };
    }

    pub fn remove(&mut self, key: &str) {
        match &mut self._map {
            None => {}
            Some(map) => {
                map.remove(key);
            }
        }
    }

    pub fn count(&self, key: &str) -> usize {
        match self.get_all(key) {
            None => 0,
            Some(vec) => vec.len(),
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        match &self._map {
            None => false,
            Some(map) => map.contains_key(key),
        }
    }
}
