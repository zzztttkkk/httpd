use std::collections::HashMap;

use crate::compress::CompressType;
use crate::multi_values_map::MultiValuesMap;

#[derive(Debug)]
pub struct Headers {
    map: Option<MultiValuesMap>,

    _content_length: Option<i64>,
    _is_chunked: Option<bool>,
    _compress_type: Option<Option<CompressType>>,
}

impl Headers {
    pub fn new() -> Self {
        Self {
            map: None,
            _content_length: None,
            _is_chunked: None,
            _compress_type: None,
        }
    }

    pub fn map(&self) -> Option<&HashMap<String, Vec<String>>> {
        match &self.map {
            None => None,
            Some(map) => map.map(),
        }
    }

    pub fn append(&mut self, k: &str, v: &str) {
        match &mut self.map {
            None => {
                let mut map = MultiValuesMap {
                    _map: None,
                    case_sensitive: false,
                };
                map.append(k, v);
                self.map = Some(map);
            }
            Some(map) => {
                map.append(k, v);
            }
        }
    }

    pub fn set(&mut self, k: &str, v: &str) {
        match &mut self.map {
            None => {
                let mut map = MultiValuesMap {
                    _map: None,
                    case_sensitive: false,
                };
                map.append(k, v);
                self.map = Some(map);
            }
            Some(map) => {
                map.set(k, v);
            }
        }
    }

    pub fn remove(&mut self, k: &str) {
        match &mut self.map {
            None => {}
            Some(map) => {
                map.remove(k);
            }
        }
    }

    pub fn contains(&self, k: &str) -> bool {
        match &self.map {
            None => false,
            Some(map) => map.contains(k),
        }
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        match &self.map {
            None => None,
            Some(map) => map.get(k),
        }
    }

    pub fn get_all(&self, k: &str) -> Option<&Vec<String>> {
        match &self.map {
            None => None,
            Some(map) => map.get_all(k),
        }
    }

    pub fn get_all_mut(&mut self, k: &str) -> Option<&mut Vec<String>> {
        match &mut self.map {
            None => None,
            Some(map) => map.get_all_mut(k),
        }
    }

    pub fn content_length(&mut self) -> Option<i64> {
        match &mut self._content_length {
            None => {
                let l: i64;
                match self.get("content-length") {
                    None => {
                        l = -1;
                    }
                    Some(s) => match s.parse::<usize>() {
                        Ok(v) => {
                            l = v as i64;
                        }
                        Err(_) => {
                            l = -1;
                        }
                    },
                }
                self._content_length = Some(l);
                Some(l)
            }
            Some(v) => Some(*v),
        }
    }

    pub fn set_content_length(&mut self, len: usize) {
        self.set("content-length", len.to_string().as_str())
    }

    pub fn content_type(&self) -> Option<&String> {
        self.get("content-type")
    }

    pub fn set_content_type(&mut self, val: &str) {
        self.set("content-type", val)
    }

    pub fn compress_type(&mut self, key: &str) -> Option<CompressType> {
        return match self._compress_type {
            None => match self.get_all(key) {
                None => {
                    self._compress_type = Some(None);
                    None
                }
                Some(vec) => {
                    let mut ct: Option<CompressType> = None;
                    for item in vec.iter() {
                        let item = item.to_ascii_lowercase();
                        if item == "deflate" {
                            ct = Some(CompressType::Deflate);
                            break;
                        }
                        if item == "gzip" {
                            ct = Some(CompressType::Gzip);
                            break;
                        }

                        let fo = item
                            .split(',')
                            .map(|v| v.trim())
                            .find(|&x| x.starts_with("deflate") || x.starts_with("gzip"));
                        if let Some(v) = fo {
                            if v.starts_with('d') {
                                ct = Some(CompressType::Deflate);
                            } else {
                                ct = Some(CompressType::Gzip);
                            }
                            break;
                        }
                    }
                    self._compress_type = Some(ct);
                    ct
                }
            },
            Some(opt) => opt,
        };
    }

    pub fn set_content_encoding(&mut self, compress_type: CompressType) {
        match compress_type {
            CompressType::Gzip => self.set("content-encoding", "gzip"),
            CompressType::Deflate => self.set("content-encoding", "deflate"),
        }
    }

    pub fn ischunked(&mut self) -> bool {
        match self._is_chunked {
            None => {
                let mut v: bool = false;

                match self.get_all("transfer-encoding") {
                    None => {}
                    Some(vec) => {
                        for item in vec.iter() {
                            if item == "chunked" {
                                v = true;
                                break;
                            }

                            if !item.contains(',') {
                                continue;
                            }

                            let fo = item
                                .split(',')
                                .map(|v| v.trim())
                                .find(|&x| x.starts_with("chunked"));
                            if fo.is_some() {
                                v = true;
                                break;
                            }
                        }
                    }
                }

                self._is_chunked = Some(v);
                v
            }
            Some(v) => v,
        }
    }
}
