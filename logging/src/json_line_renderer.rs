use utils::luxon;

use crate::Renderer;

#[derive(Debug, Default)]
pub struct JsonLineRenderer {
    name: String,
    timelayout: String,
}

impl Renderer for JsonLineRenderer {
    fn name(&self) -> &str {
        if self.name.is_empty() {
            return "JsonLineRenderer";
        }
        &self.name
    }

    fn render(&self, item: &crate::item::Item, buf: &mut Vec<u8>) {
        buf.push(b'{');

        macro_rules! push_with_quote {
            ($key:expr) => {
                buf.push(b'"');
                buf.extend($key.as_bytes());
                buf.push(b'"');
            };
        }

        push_with_quote!("level");
        buf.push(b':');
        push_with_quote!(item.level.as_str());

        buf.push(b',');

        push_with_quote!("time");
        buf.push(b':');
        let time_in_txt;
        if self.timelayout.is_empty() {
            time_in_txt = luxon::fmtlocal(item.time, "%Y-%m-%d %H:%M:%S%.6f %Z");
        } else {
            time_in_txt = luxon::fmtlocal(item.time, &self.timelayout);
        }
        push_with_quote!(&time_in_txt.to_string());

        buf.push(b',');

        if !item.target.is_empty() {
            push_with_quote!("target");
            buf.push(b':');
            push_with_quote!(item.target);
        }

        buf.push(b',');

        push_with_quote!("lineno");
        buf.push(b':');
        push_with_quote!(format!("{}:{}", item.file, item.line));

        buf.push(b',');

        push_with_quote!("message");
        buf.push(b':');
        push_with_quote!(item.msg);

        if item.kvs.is_empty() {
            buf.extend("}\r\n".as_bytes());
            return;
        }

        buf.push(b',');
        push_with_quote!("kvs");
        buf.push(b':');

        buf.push(b'{');
        let last = item.kvs.len() - 1;
        for (idx, pair) in item.kvs.iter().enumerate() {
            buf.extend(pair.0.as_bytes());
            buf.push(b':');
            buf.extend(pair.1.as_bytes());
            if idx != last {
                buf.push(b',');
            }
        }

        buf.extend("}}\r\n".as_bytes());
    }
}
