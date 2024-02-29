use std::collections::HashMap;

/// multi value map
/// used to save http headers/forms
/// !!! it is the user's responsibility to ensure that the `key` must not be empty.
#[derive(Debug)]
pub struct MultiMap {
    ismap: bool,
    vec: Vec<(String, Vec<String>)>,
    map: Option<HashMap<String, Vec<String>>>,
}

impl Default for MultiMap {
    fn default() -> Self {
        Self {
            ismap: false,
            vec: vec![],
            map: None,
        }
    }
}

impl MultiMap {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    fn vec_find(pairs: &Vec<(String, Vec<String>)>, k: &str) -> Option<usize> {
        pairs.iter().position(|e| e.0 == k)
    }

    #[inline]
    fn vec_find_or_empty(pairs: &Vec<(String, Vec<String>)>, k: &str) -> Option<usize> {
        pairs.iter().position(|e| e.0 == k || e.0.is_empty())
    }

    fn map_append(&mut self, k: &str, v: &str) {
        let map = self.map.as_mut().unwrap();
        match map.get_mut(k) {
            Some(vs) => {
                vs.push(v.to_string());
            }
            None => {
                map.insert(k.to_string(), vec![v.to_string()]);
            }
        }
    }

    pub fn append(&mut self, k: &str, v: &str) {
        if self.ismap {
            self.map_append(k, v);
            return;
        }

        if self.vec.len() >= 36 {
            self.tomap();
            self.map_append(k, v);
            return;
        }

        let pairs = &mut self.vec;

        match Self::vec_find_or_empty(pairs, k) {
            Some(pos) => {
                let vs = unsafe { pairs.get_unchecked_mut(pos) };
                vs.1.push(v.to_string());
                if vs.0.is_empty() {
                    vs.0 = k.to_string();
                }
            }
            None => {
                pairs.push((k.to_string(), vec![v.to_string()]));
            }
        }
    }

    #[inline]
    fn vec_remove_by_value(vs: &mut Vec<String>, v: &str) {
        let mut skip: usize = 0;
        loop {
            match vs.iter().skip(skip).position(|e| e == v) {
                Some(pos) => {
                    vs.remove(pos + skip);
                    skip = pos;
                }
                None => {
                    break;
                }
            }
        }
    }

    pub fn remove(&mut self, k: &str, v: &str) {
        if self.ismap {
            let map = self.map.as_mut().unwrap();
            match map.get_mut(k) {
                Some(vs) => {
                    Self::vec_remove_by_value(vs, v);
                }
                None => {}
            }
            return;
        }

        let pairs = &mut self.vec;
        match Self::vec_find(pairs, k) {
            Some(pos) => {
                let vs = unsafe { pairs.get_unchecked_mut(pos) };
                Self::vec_remove_by_value(&mut vs.1, v);
            }
            None => {}
        }
    }

    pub fn delete(&mut self, k: &str) {
        if self.ismap {
            let map = self.map.as_mut().unwrap();
            map.remove(k);
            return;
        }

        let pairs = &mut self.vec;
        match Self::vec_find(pairs, k) {
            Some(pos) => {
                let vs = unsafe { pairs.get_unchecked_mut(pos) };
                vs.0.clear();
                vs.1.clear();
            }
            None => {}
        }
    }

    pub fn clear(&mut self) {
        self.vec.clear();
        match self.map.as_mut() {
            Some(map) => {
                map.clear();
            }
            None => {}
        }
        self.ismap = false;
    }

    fn tomap(&mut self) {
        if self.map.is_none() {
            self.map = Some(Default::default());
        }
        let map = self.map.as_mut().unwrap();
        for (k, vs) in self.vec.iter() {
            map.insert(k.clone(), vs.clone());
        }
        self.vec.clear();
        self.ismap = true;
    }

    pub fn getall(&self, k: &str) -> Option<&Vec<String>> {
        if self.ismap {
            return self.map.as_ref().unwrap().get(k);
        }
        for (key, vs) in self.vec.iter() {
            if key == k {
                return Some(vs);
            }
        }
        None
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        match self.getall(k) {
            Some(vs) => vs.first(),
            None => None,
        }
    }

    pub fn each<V: Fn(&str, &Vec<String>) -> bool>(&self, visitor: V) {
        if self.ismap {
            for (k, vs) in self.map.as_ref().unwrap().iter() {
                if !visitor(k, vs) {
                    break;
                }
            }
            return;
        }

        for (k, vs) in self.vec.iter() {
            if !visitor(k, vs) {
                break;
            }
        }
    }

    pub fn each_mut<V: Fn(&str, &mut Vec<String>) -> bool>(&mut self, visitor: V) {
        if self.ismap {
            for (k, vs) in self.map.as_mut().unwrap().iter_mut() {
                if !visitor(k, vs) {
                    break;
                }
            }
            return;
        }

        for (k, vs) in self.vec.iter_mut() {
            if !visitor(k, vs) {
                break;
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.ismap {
            return self.map.as_ref().unwrap().len();
        }
        self.vec.len()
    }
}

#[cfg(test)]
mod tests {
    use super::MultiMap;

    #[test]
    fn test_multi_map() {
        let mut map = MultiMap::new();

        let mut i = 0;
        loop {
            map.append(format!("k{}", i).as_str(), i.to_string().as_str());
            i += 1;
            if i > 40 {
                break;
            }
        }

        println!("{:?}", map);

        map.each_mut(|k, vs| {
            vs.push("0.0".to_string());
            true
        });
        println!("{:?}", map);
    }
}
