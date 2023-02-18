use std::cell::UnsafeCell;
use std::collections::HashMap;

use crate::http::compress::CompressType;
use crate::utils::MultiValuesMap;

#[derive(Default)]
pub struct Headers {
    map: Option<MultiValuesMap>,
}

impl Headers {
    pub fn clear(&mut self) {
        if let Some(map) = &mut self.map {
            map.clear();
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

    pub fn content_length(&self) -> usize {
        let l: usize;
        match self.get("content-length") {
            None => {
                l = 0;
            }
            Some(s) => match s.parse::<usize>() {
                Ok(v) => {
                    l = v;
                }
                Err(_) => {
                    l = 0;
                }
            },
        }
        l
    }

    pub fn set_content_length(&mut self, len: usize) {
        self.set("content-length", len.to_string().as_str());
    }

    pub fn content_type(&self) -> Option<&String> {
        self.get("content-type")
    }

    pub fn set_content_type(&mut self, val: &str) {
        self.set("content-type", val)
    }

    pub fn content_encoding(&self) -> Option<CompressType> {
        self.compress_type("content-encoding")
    }

    pub fn accept_encoding(&self) -> Option<CompressType> {
        self.compress_type("accept-encoding")
    }

    fn compress_type(&self, key: &str) -> Option<CompressType> {
        match self.get_all(key) {
            None => None,
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
                ct
            }
        }
    }

    pub fn set_content_encoding(&mut self, compress_type: CompressType) {
        match compress_type {
            CompressType::Gzip => self.set("content-encoding", "gzip"),
            CompressType::Deflate => self.set("content-encoding", "deflate"),
        }
    }

    pub fn ischunked(&self) -> bool {
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

        v
    }
}
