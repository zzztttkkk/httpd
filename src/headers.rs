use std::collections::HashMap;

#[derive(Debug)]
struct KvItem {
    key: String,
    val: String,
    ok: bool,
}

#[derive(Debug)]
struct KvItems {
    items: Vec<KvItem>,
    bad_idxes: Vec<usize>,
}

impl KvItems {
    fn new() -> Self {
        Self { items: vec![], bad_idxes: vec![] }
    }

    fn add(&mut self, k: &str, v: &str) {
        if let Some(idx) = self.bad_idxes.pop() {
            let item = &mut self.items[idx];
            item.key.clear();
            item.key.push_str(k);
            item.val.clear();
            item.val.push_str(v);
            return;
        }
        self.items.push(KvItem {
            key: k.to_string(),
            val: v.to_string(),
            ok: true,
        });
    }

    fn _find(&self, k: &str) {}
}


#[derive(Debug)]
pub struct Headers {
    items: Option<KvItems>,
    map: Option<HashMap<String, Vec<String>>>,

    _content_length: Option<i64>,
}

impl Headers {
    pub fn new() -> Self {
        Self { items: None, map: Some(HashMap::new()), _content_length: None }
    }

    pub fn add(&mut self, k: &str, v: &str) {
        match &mut self.map {
            None => {
                println!("{} {}", k, v);
            }
            Some(m) => {
                match m.get_mut(k) {
                    None => {
                        m.insert(k.to_string(), vec![v.to_string()]);
                    }
                    Some(lst) => {
                        lst.push(v.to_string());
                    }
                }
            }
        }
    }

    pub fn set(&mut self, k: &str, v: &str) {}

    pub fn del(&mut self, k: &str) {}

    pub fn contains(&self, k: &str) {}

    pub fn get(&self, k: &str) -> Option<&str> {
        match &self.map {
            None => {
                None
            }
            Some(m) => {
                match m.get(k) {
                    None => {
                        None
                    }
                    Some(lst) => {
                        match lst.first() {
                            None => {
                                None
                            }
                            Some(v) => {
                                Some(v)
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn getall(&self, k: &str) -> Option<&Vec<String>> {
        None
    }

    pub fn getallmut(&mut self, k: &str) -> Option<&mut Vec<String>> {
        None
    }

    pub fn contentlength(&mut self) -> Option<i64> {
        match &mut self._content_length {
            None => {
                let l: i64;
                match self.get("Content-Length") {
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

    pub fn ischunked(&mut self) -> bool {
        false
    }
}


