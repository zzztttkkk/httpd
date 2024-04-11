use crate::utils::luxon;

use super::{
    appender::Renderer,
    color::{Color, ColorScheme},
    item::Item,
};

fn with_color(buf: &mut Vec<u8>, txt: &str, color: &Option<Color>) {
    match color.as_ref() {
        Some(color) => {
            if color.is_block() {
                buf.extend(txt.as_bytes());
                return;
            }
            buf.push(b'\x1b');
            buf.extend(format!("[38;2;{};{};{}m", color.0, color.1, color.2).as_bytes());
            buf.extend(txt.as_bytes());
            buf.push(b'\x1b');
            buf.extend("[0m".as_bytes());
        }
        None => {
            buf.extend(txt.as_bytes());
        }
    }
}

#[derive(Default)]
pub struct ColorfulLineRenderer {
    name: String,
    scheme: ColorScheme,
    timelayout: String,
}

impl Renderer for ColorfulLineRenderer {
    fn name(&self) -> &str {
        if self.name.is_empty() {
            return "ColorfulLineRenderer";
        }
        &self.name
    }

    fn render(&self, item: &Item, buf: &mut Vec<u8>) {
        let level = match self.scheme.levels.as_ref() {
            Some(colors) => colors.get(item.level),
            None => None,
        };
        with_color(buf, format!("[{}]", item.level.as_str()).as_str(), &level);
        buf.push(b' ');

        let time_in_txt;
        if self.timelayout.is_empty() {
            time_in_txt = luxon::fmtlocal(item.time, "%Y-%m-%d %H:%M:%S%.3f");
        } else {
            time_in_txt = luxon::fmtlocal(item.time, &self.timelayout);
        }
        with_color(buf, &time_in_txt.to_string(), &self.scheme.time);

        buf.push(b' ');
        buf.push(b'(');
        with_color(buf, item.file, &self.scheme.file);
        buf.push(b':');
        with_color(buf, item.line.to_string().as_str(), &self.scheme.line);
        buf.extend(") ".as_bytes());

        with_color(buf, &item.msg, &level);

        if item.kvs.is_empty() {
            buf.extend("\r\n".as_bytes());
            return;
        }

        buf.extend(" { ".as_bytes());

        let last = item.kvs.len() - 1;
        for (idx, pair) in item.kvs.iter().enumerate() {
            with_color(buf, pair.0.as_str(), &self.scheme.key);
            buf.extend(": ".as_bytes());
            with_color(buf, pair.1.as_str(), &level);
            if idx != last {
                buf.extend(" , ".as_bytes());
            }
        }

        buf.extend(" }\r\n".as_bytes());
    }
}

pub struct ColorfulLineRendererBuilder {
    ins: ColorfulLineRenderer,
}

impl ColorfulLineRendererBuilder {
    pub fn new() -> Self {
        Self {
            ins: Default::default(),
        }
    }

    pub fn with_name(&mut self, name: &str) -> &mut Self {
        self.ins.name = name.to_string();
        self
    }

    pub fn with_shceme(&mut self, scheme: ColorScheme) -> &mut Self {
        self.ins.scheme = scheme;
        self
    }

    pub fn with_timelayout(&mut self, layout: &str) -> &mut Self {
        self.ins.timelayout = layout.to_string();
        self
    }

    pub fn finish(self) -> ColorfulLineRenderer {
        self.ins
    }
}

#[cfg(test)]
mod tests {
    use log::Level;
    use slab::Slab;

    use crate::logging::{appender, ConsoleAppender};

    use super::ColorfulLineRenderer;

    #[test]
    fn test_colors() {
        let mut slab = Slab::new();
        slab.insert(vec![0]);

        let _g = crate::logging::init(
            Level::Trace,
            vec![Box::new(ConsoleAppender::new(
                0,
                "ColorfulLineRenderer",
                appender::filter(|_| true),
            ))],
            vec![Box::new(ColorfulLineRenderer::default())],
            slab,
        )
        .unwrap();

        log::trace!(a= 12, b = "xxx5"; "this is a trace msg, hello {} !", "world");
        log::debug!(a= 13, b = "xxx4"; "this is a debug msg");
        log::info!(a= 14, b = "xxx3"; "this is a info msg");
        log::warn!(a= 15, b = "xxx2"; "this is a warn msg");
        log::error!(a= 16, b = "xxx1"; "this is a error msg");
    }
}
