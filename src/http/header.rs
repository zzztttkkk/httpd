use std::{cell::RefCell, collections::HashMap};
use std::fmt::format;
use crate::http::body::CompressionType;

pub struct Header {
    val: Option<HashMap<String, Vec<String>>>,
    content_length: RefCell<Option<Result<usize, String>>>,
    content_encoding: RefCell<Option<Result<Option<CompressionType>, String>>>,
}

impl Header {
    pub fn new() -> Self {
        return Self {
            val: None,
            content_length: RefCell::new(None),
            content_encoding: RefCell::new(None),
        };
    }

    pub fn contains(&self, key: &str) -> bool {
        if let Some(map) = self.val.as_ref() {
            let key = key.to_lowercase();
            if let Some(vs) = map.get(key.as_str()) {
                return !vs.is_empty();
            }
        }
        return false;
    }

    pub fn add(&mut self, key: &str, val: &str) {
        if self.val.is_none() {
            self.val = Some(HashMap::new())
        }
        let map = self.val.as_mut().unwrap();
        let key = key.to_lowercase();
        match map.get_mut(key.as_str()) {
            Some(vs) => {
                vs.push(val.to_string());
            }
            None => {
                map.insert(key, vec![val.to_string()]);
            }
        }
    }

    pub fn set(&mut self, key: &str, val: &str) {
        if self.val.is_none() {
            self.val = Some(HashMap::new())
        }
        let map = self.val.as_mut().unwrap();
        let key = key.to_lowercase();
        match map.get_mut(key.as_str()) {
            Some(vs) => {
                vs.clear();
                vs.push(val.to_string());
            }
            None => {
                map.insert(key, vec![val.to_string()]);
            }
        }
    }

    pub fn del(&mut self, key: &str, val: Option<&str>) {
        if let Some(map) = self.val.as_mut() {
            let key = key.to_lowercase();

            match val {
                Some(dv) => {
                    if let Some(vs) = map.get_mut(key.as_str()) {
                        vs.retain(|v| v != dv);
                    }
                }
                None => {
                    map.remove(key.as_str());
                }
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        return match self.getall(key) {
            Some(vs) => {
                vs.first()
            }
            None => {
                None
            }
        };
    }

    pub fn getall(&self, key: &str) -> Option<&Vec<String>> {
        if let Some(map) = self.val.as_ref() {
            let key = key.to_lowercase();
            return map.get(key.as_str());
        }
        return None;
    }

    pub fn clear(&mut self) {
        if let Some(map) = self.val.as_mut() {
            map.clear();
        }
        let mut cl_ref = self.content_length.borrow_mut();
        *cl_ref = None;
    }

    pub fn get_content_length(&self) -> Result<usize, String> {
        let mut cl_ref = self.content_length.borrow_mut();
        if cl_ref.is_none() {
            let result: Result<usize, String>;
            if let Some(txt) = self.get("content-length") {
                match txt.parse::<usize>() {
                    Ok(v) => {
                        result = Ok(v);
                    }
                    Err(e) => {
                        result = Err(e.to_string());
                    }
                }
            } else {
                result = Ok(0);
            }

            *cl_ref = Some(result);
        }
        return cl_ref.as_ref().unwrap().clone();
    }

    pub fn set_content_length(&mut self, size: usize) {
        let mut cl_ref = self.content_length.borrow_mut();
        *cl_ref = Some(Ok(size));
        drop(cl_ref);
        self.set("content-length", size.to_string().as_str());
    }

    pub fn get_content_encoding(&self) -> Result<Option<CompressionType>, String> {
        let mut ce_ref = self.content_encoding.borrow_mut();
        if ce_ref.is_none() {
            match self.getall("content-encoding").as_ref() {
                None => {
                    *ce_ref = Some(Ok(None));
                }
                Some(vs) => {
                    for v in *vs {
                        for ele in v.split(',') {
                            let ele = ele.trim();
                            if ele.is_empty() {
                                continue;
                            }
                            return match ele {
                                "br" => {
                                    *ce_ref = Some(Ok(Some(CompressionType::Br)));
                                    ce_ref.as_ref().unwrap().clone()
                                }
                                "deflate" => {
                                    *ce_ref = Some(Ok(Some(CompressionType::Deflate)));
                                    ce_ref.as_ref().unwrap().clone()
                                }
                                "gzip" => {
                                    *ce_ref = Some(Ok(Some(CompressionType::Gzip)));
                                    ce_ref.as_ref().unwrap().clone()
                                }
                                _ => {
                                    *ce_ref = Some(
                                        Err(format!("unsupported content encoding value, {}", ele).to_string())
                                    );
                                    ce_ref.as_ref().unwrap().clone()
                                }
                            };
                        }
                    }
                    *ce_ref = Some(Ok(None));
                }
            }
        }
        return ce_ref.as_ref().unwrap().clone();
    }

    pub fn each<F: FnMut(&String, &Vec<String>) -> ()>(&self, mut visitor: F) {
        if let Some(map) = self.val.as_ref() {
            for (k, v) in map {
                visitor(k, v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Header;

    fn read(header: &Header) {
        println!("{:?}", header.get("xxx"));
        println!("{:?}", header.get_content_length());
    }

    #[test]
    fn test_header() {
        let mut header = Header::new();
        header.set("xxx", "yyy");
        header.set("Content-Length", "1203");
        read(&header);
    }
}
