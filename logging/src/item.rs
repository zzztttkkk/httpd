type Kvs = smallvec::SmallVec<[(String, String); 16]>;

pub struct Item {
    pub time: std::time::SystemTime,
    pub level: log::Level,
    pub file: &'static str,
    pub line: u32,
    pub msg: String,
    pub kvs: Kvs,
    pub service: String,
}

struct KeyBuf(smallvec::SmallVec<[u8; 32]>);
struct ValBuf(smallvec::SmallVec<[u8; 128]>);

macro_rules! impl_write {
    ($cls:ident) => {
        impl $cls {
            pub fn new() -> Self {
                Self(Default::default())
            }

            fn str(&self) -> &str {
                unsafe { std::str::from_utf8_unchecked(self.0.as_slice()) }
            }

            fn unqoute(&self) -> &str {
                let v = unsafe { std::str::from_utf8_unchecked(self.0.as_slice()) };
                if v.starts_with('"') && v.starts_with('"') {
                    return &(v[1..v.len() - 1]);
                }
                return v;
            }
        }

        impl std::io::Write for $cls {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
    };
}

impl_write!(KeyBuf);
impl_write!(ValBuf);

struct KvsVisitor<'a> {
    kvs: &'a mut Kvs,
    keytemp: KeyBuf,
    valtemp: ValBuf,
    service: KeyBuf,
}

impl<'kvs, 'a> log::kv::VisitSource<'kvs> for KvsVisitor<'a> {
    fn visit_pair(
        &mut self,
        key: log::kv::Key<'kvs>,
        value: log::kv::Value<'kvs>,
    ) -> Result<(), log::kv::Error> {
        self.keytemp.0.clear();
        self.valtemp.0.clear();

        _ = serde_json::to_writer(&mut self.keytemp, &key);
        if self.keytemp.0.is_empty() {
            return Ok(());
        }

        _ = serde_json::to_writer(&mut self.valtemp, &value);

        if self.keytemp.str().eq("\"service\"") {
            self.service.0.clear();
            self.service
                .0
                .extend_from_slice(self.valtemp.unqoute().as_bytes());
            return Ok(());
        }

        self.kvs.push((
            self.keytemp.str().to_string(),
            self.valtemp.str().to_string(),
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
            service: String::new(),
        };
        let mut vi = KvsVisitor {
            kvs: &mut item.kvs,
            keytemp: KeyBuf::new(),
            valtemp: ValBuf::new(),
            service: KeyBuf::new(),
        };
        _ = value.key_values().visit(&mut vi);
        item.service = vi.service.str().to_string();
        item
    }
}
