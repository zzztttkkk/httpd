use log::kv::ToKey;

type Kvs = smallvec::SmallVec<[(String, String); 16]>;

pub struct Item {
    pub time: std::time::SystemTime,
    pub level: log::Level,
    pub file: &'static str,
    pub line: u32,
    pub msg: String,
    pub kvs: Kvs,
    pub service: usize,
}

struct ValBuf(smallvec::SmallVec<[u8; 64]>);

impl ValBuf {
    pub fn new() -> Self {
        Self(Default::default())
    }

    fn str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.0.as_slice()) }
    }
}

impl std::io::Write for ValBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct KvsVisitor<'a> {
    kvs: &'a mut Kvs,
    valtemp: ValBuf,
    service: &'a mut usize,
}

impl<'kvs, 'a> log::kv::VisitSource<'kvs> for KvsVisitor<'a> {
    fn visit_pair(
        &mut self,
        key: log::kv::Key<'kvs>,
        value: log::kv::Value<'kvs>,
    ) -> Result<(), log::kv::Error> {
        let key = key.to_key();
        let key = key.as_str();
        if key.is_empty() {
            return Ok(());
        }

        if key.eq("service") {
            match value.to_u64() {
                Some(idx) => {
                    *self.service = idx as usize;
                }
                None => {}
            }
            return Ok(());
        }

        self.valtemp.0.clear();
        _ = serde_json::to_writer(&mut self.valtemp, &value);

        self.kvs
            .push((key.to_string(), self.valtemp.str().to_string()));
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
            service: 0,
        };
        let mut vi = KvsVisitor {
            kvs: &mut item.kvs,
            valtemp: ValBuf::new(),
            service: &mut item.service,
        };
        _ = value.key_values().visit(&mut vi);
        item
    }
}
