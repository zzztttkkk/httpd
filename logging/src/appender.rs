use crate::item::Item;

pub trait Renderer: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, item: &Item, buf: &mut Vec<u8>);
}

pub trait Appender: tokio::io::AsyncWrite + Unpin + Send + Sync {
    fn renderer(&self) -> &str; // renderer name
    fn filter(&self, item: &Item) -> bool;
}

#[derive(Default)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub(crate) fn is_block(&self) -> bool {
        self.0 == 0 && self.1 == 0 && self.2 == 1
    }
}

#[derive(Default)]
pub struct ColorScheme {
    pub level: Option<Color>,
    pub time: Option<Color>,

    pub target: Option<Color>,

    pub module: Option<Color>,
    pub file: Option<Color>,
    pub line: Option<Color>,

    pub msg: Option<Color>,

    pub key: Option<Color>,
    pub value: Option<Color>,
}

fn with_color(buf: &mut Vec<u8>, txt: &str, color: &Option<Color>) {
    match color.as_ref() {
        Some(color) => {
            if color.is_block() {
                buf.extend(txt.as_bytes());
                return;
            }
            buf.extend(format!("\x1b[38;2;{};{};{}m", color.0, color.1, color.2).as_bytes());
            buf.extend(txt.as_bytes());
            buf.extend("\x1b[0m".as_bytes());
        }
        None => {
            buf.extend(txt.as_bytes());
        }
    }
}

#[derive(Default)]
pub struct ColorfulLineRenderer {
    scheme: ColorScheme,
    timelayout: String,
}

impl Renderer for ColorfulLineRenderer {
    fn name(&self) -> &str {
        "ColorfulLineRenderer"
    }

    fn render(&self, item: &Item, buf: &mut Vec<u8>) {
        with_color(buf, &format!("{}", item.level.as_str()), &self.scheme.level);
        buf.push(b' ');

        let time: chrono::DateTime<chrono::Utc> = item.time.into();
        let time_in_txt;
        if self.timelayout.is_empty() {
            time_in_txt = time.format("%Y-%m%d %H:%M:%S%.6f %Z");
        } else {
            time_in_txt = time.format(&self.timelayout);
        }
        with_color(buf, &time_in_txt.to_string(), &self.scheme.time);
        buf.push(b' ');

        if item.target.is_empty() {
            with_color(buf, &item.target, &self.scheme.target);
            buf.push(b' ');
        }

        with_color(buf, item.module, &self.scheme.module);
        buf.push(b'(');
        with_color(buf, item.file, &self.scheme.file);
        buf.push(b':');
        with_color(buf, item.line.to_string().as_str(), &self.scheme.line);
        buf.extend(") ".as_bytes());

        with_color(buf, &item.msg, &self.scheme.msg);

        if item.kvs.is_empty() {
            buf.extend("\r\n".as_bytes());
            return;
        }

        buf.extend(" { ".as_bytes());

        let last = item.kvs.len() - 1;
        for (idx, pair) in item.kvs.iter().enumerate() {
            with_color(buf, pair.0.as_str(), &self.scheme.key);
            buf.extend(": ".as_bytes());
            with_color(buf, pair.1.as_str(), &self.scheme.value);
            if idx != last {
                buf.extend(" , ".as_bytes());
            }
        }

        buf.extend(" }\r\n".as_bytes());
    }
}
