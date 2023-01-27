use crate::compress::CompressType;
use crate::multi_value_map::MultiValuesMap;

#[derive(Debug)]
pub struct Headers {
    map: Option<MultiValuesMap>,
    _content_length: Option<i64>,
    _is_chunked: Option<bool>,
    _compress_type: Option<Result<CompressType, String>>,
}

impl Headers {
    pub fn new() -> Self {
        Self { map: None, _content_length: None, _is_chunked: None, _compress_type: None }
    }

    pub fn append(&mut self, k: &str, v: &str) {
        match &mut self.map {
            None => {
                let mut map = MultiValuesMap::new();
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
                let mut map = MultiValuesMap::new();
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
            Some(map) => { map.remove(k); }
        }
    }

    pub fn contains(&self, k: &str) -> bool {
        match &self.map {
            None => { false }
            Some(map) => { map.contains(k) }
        }
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        match &self.map {
            None => { None }
            Some(map) => { map.get(k) }
        }
    }

    pub fn getall(&self, k: &str) -> Option<&Vec<String>> {
        match &self.map {
            None => { None }
            Some(map) => { map.getall(k) }
        }
    }

    pub fn getallmut(&mut self, k: &str) -> Option<&mut Vec<String>> {
        match &mut self.map {
            None => { None }
            Some(map) => { map.getallmut(k) }
        }
    }

    pub fn contentlength(&mut self) -> Option<i64> {
        match &mut self._content_length {
            None => {
                let l: i64;
                match self.get("content-length") {
                    None => {
                        l = -1;
                    }
                    Some(s) => {
                        match s.parse::<usize>() {
                            Ok(v) => {
                                l = v as i64;
                            }
                            Err(_) => {
                                l = -1;
                            }
                        }
                    }
                }
                self._content_length = Some(l);
                Some(l)
            }
            Some(v) => {
                Some(*v)
            }
        }
    }

    pub fn contenttype(&mut self) -> Option<String> {
        None
    }

    pub fn compresstype(&mut self) -> Result<CompressType, String> {
        match self._compress_type {
            None => {}
            Some(_) => {}
        }
        Err("".to_string())
    }

    pub fn ischunked(&mut self) -> bool {
        match self._is_chunked {
            None => {
                let mut v: bool = false;

                match self.getall("transfer-encoding") {
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

                            let fo = item.split(',').map(|v| { v.trim() }).find(|&x| x == "chunked");
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
            Some(v) => { v }
        }
    }
}


