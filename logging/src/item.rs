type Kvs = smallvec::SmallVec<[(String, String); 16]>;

pub struct Item {
    time: std::time::SystemTime,
    level: log::Level,
    target: String,
    module: &'static str,
    file: &'static str,
    line: u32,
    msg: String,
    kvs: Kvs,
}

impl<'kvs> log::kv::VisitSource<'kvs> for Item {
    fn visit_pair(
        &mut self,
        key: log::kv::Key<'kvs>,
        value: log::kv::Value<'kvs>,
    ) -> Result<(), log::kv::Error> {
        Ok(())
    }
}

impl std::convert::From<&log::Record<'_>> for Item {
    fn from(value: &log::Record) -> Self {
        let mut item = Item {
            time: std::time::SystemTime::now(),
            level: value.level(),
            target: value.target().to_string(),
            module: value.module_path_static().map_or("", |v| v),
            file: value.file_static().map_or("", |v| v),
            line: value.line().map_or(0, |v| v),
            msg: format!("{}", value.args()),
            kvs: smallvec::smallvec![],
        };

        // value.key_values().visit(|| {});
        item
    }
}
