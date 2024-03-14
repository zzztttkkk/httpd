type Kvs = smallvec::SmallVec<[(String, String); 16]>;

pub struct Item {
    pub time: std::time::SystemTime,
    pub level: log::Level,
    pub file: &'static str,
    pub line: u32,
    pub msg: String,
    pub kvs: Kvs,
}

impl<'kvs> log::kv::VisitSource<'kvs> for Item {
    fn visit_pair(
        &mut self,
        key: log::kv::Key<'kvs>,
        value: log::kv::Value<'kvs>,
    ) -> Result<(), log::kv::Error> {
        self.kvs.push((
            serde_json::to_string(&key).map_or(String::default(), |v| v),
            serde_json::to_string(&value).map_or(String::default(), |v| v),
        ));
        Ok(())
    }
}

impl std::convert::From<&log::Record<'_>> for Item {
    fn from(value: &log::Record) -> Self {
        let mut item = Item {
            time: std::time::SystemTime::now(),
            level: value.level(),
            file: value.file_static().map_or("", |v| v),
            line: value.line().map_or(0, |v| v),
            msg: format!("{}", value.args()),
            kvs: smallvec::smallvec![],
        };
        _ = value.key_values().visit(&mut item);
        item
    }
}
