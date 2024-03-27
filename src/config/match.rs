use serde::de::Visitor;

#[derive(PartialEq, Eq)]
pub enum CondationKind {
    Method,
    Path,
    Query,
    Header,
    Cookie,
    UserAgent, // user agent
    FormBody,  // http form
    JsonBody,  // json path
    MatchRef,
}

#[derive(PartialEq, Eq)]
pub enum OpKind {
    Eq,
    In,
    Contains,
    Match,
}

pub enum MultiValuePolicy {
    All,
    Any,
    First,
    Last,
    Nth(u16),
}

pub struct Condation {
    pub kind: CondationKind,
    pub left: String,
    pub not: Option<bool>,
    pub mvp: Option<MultiValuePolicy>,
    pub op: Option<OpKind>,
    pub right: Option<String>,
}

impl Default for Condation {
    fn default() -> Self {
        Self {
            kind: CondationKind::Method,
            left: Default::default(),
            op: Default::default(),
            not: Default::default(),
            mvp: Default::default(),
            right: Default::default(),
        }
    }
}

impl Condation {
    fn from(buf: &mut bytebuffer::ByteReader) -> Result<Option<Condation>, String> {
        let mut cond = Condation::default();

        let mut word = read_word(buf).trim().to_lowercase();

        match word.as_str() {
            "method" => {
                cond.kind = CondationKind::Method;
            }
            "path" => {
                cond.kind = CondationKind::Path;
            }
            "query" | "q" => {
                cond.kind = CondationKind::Query;
            }
            "header" => {
                cond.kind = CondationKind::Header;
            }
            "cookie" => {
                cond.kind = CondationKind::Cookie;
            }
            "useragent" | "ua" => {
                cond.kind = CondationKind::UserAgent;
            }
            "form" => {
                cond.kind = CondationKind::FormBody;
            }
            "json" => {
                cond.kind = CondationKind::JsonBody;
            }
            "ref" => {
                cond.kind = CondationKind::MatchRef;
            }
            "}" => {
                return Ok(None);
            }
            _ => {}
        }

        if !read_until_ignore_space(buf, b'<') {
            return Err("require a `<`".to_string());
        }
        word = read_word(buf);
        if !read_until_ignore_space(buf, b'>') {
            return Err("require a `>`".to_string());
        }
        cond.left = word;

        loop {
            word = read_word(buf).trim().to_lowercase();
            if word.is_empty() {
                break;
            }

            match word.as_str() {
                "not" => {
                    cond.not = Some(true);
                }
                "all" => {
                    if cond.mvp.is_some() {
                        return Err("multi value policy already set".to_string());
                    }
                    cond.mvp = Some(MultiValuePolicy::All);
                }
                "any" => {
                    if cond.mvp.is_some() {
                        return Err("multi value policy already set".to_string());
                    }
                    cond.mvp = Some(MultiValuePolicy::Any);
                }
                "first" => {
                    if cond.mvp.is_some() {
                        return Err("multi value policy already set".to_string());
                    }
                    cond.mvp = Some(MultiValuePolicy::First);
                }
                "last" => {
                    if cond.mvp.is_some() {
                        return Err("multi value policy already set".to_string());
                    }
                    cond.mvp = Some(MultiValuePolicy::Last);
                }
                _ => {
                    if word.starts_with("nth#") {
                        if cond.mvp.is_some() {
                            return Err("multi value policy already set".to_string());
                        }

                        match (word[4..]).parse::<u16>() {
                            Ok(idx) => cond.mvp = Some(MultiValuePolicy::Nth(idx)),
                            Err(_) => {
                                return Err(format!("parse to u16 failed: `{}`", word));
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        if cond.kind == CondationKind::MatchRef {
            if !read_until_ignore_space(buf, b';') {
                return Err("require a `;`".to_string());
            }
            return Ok(Some(cond));
        }

        match word.as_str() {
            "eq" => {}
            "in" => {}
            "contains" => {}
            "match" => {}
            _ => {}
        }

        Ok(None)
    }
}

pub enum LogicKind {
    And,
    Or,
}

pub struct Match {
    pub logic: LogicKind,
    pub conds: Vec<Condation>,
}

impl Default for Match {
    fn default() -> Self {
        Self {
            logic: LogicKind::And,
            conds: Default::default(),
        }
    }
}

fn read_word(buf: &mut bytebuffer::ByteReader) -> String {
    let mut bytes = vec![];

    loop {
        let c = buf.read_u8().unwrap();
        if bytes.is_empty() && c.is_ascii_whitespace() {
            continue;
        }
        match c {
            b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' | b'0'..=b'9' | b'#' => {
                bytes.push(c);
            }
            _ => {
                break;
            }
        }
    }
    return String::from_utf8(bytes).unwrap();
}

fn read_until_ignore_space(buf: &mut bytebuffer::ByteReader, target: u8) -> bool {
    loop {
        match buf.read_u8() {
            Ok(c) => {
                if c == target {
                    return true;
                }
                if c.is_ascii_whitespace() {
                    continue;
                }
                return false;
            }
            Err(_) => {
                return false;
            }
        }
    }
}

impl Match {
    fn from<E>(&mut self, buf: &mut bytebuffer::ByteReader) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        let mut first_word = read_word(buf).to_lowercase().trim().to_string();
        if first_word.is_empty() {
            first_word = "and".to_string();
        }

        match first_word.as_str() {
            "and" => {
                self.logic = LogicKind::And;
            }
            "or" => {
                self.logic = LogicKind::Or;
            }
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "read a unknown logic kind: ${}",
                    first_word
                )));
            }
        }

        if !read_until_ignore_space(buf, b'{') {
            return Err(serde::de::Error::custom("require a `{`"));
        }

        loop {
            match Condation::from(buf) {
                Ok(cond) => match cond {
                    Some(cond) => self.conds.push(cond),
                    None => {
                        break;
                    }
                },
                Err(e) => {
                    return Err(serde::de::Error::custom(e));
                }
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct MatchVisitor {}

impl<'de> Visitor<'de> for MatchVisitor {
    type Value = Match;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a string value for match, see `https://github.com/zzztttkkk/httpd/match.md`",
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut buf = bytebuffer::ByteReader::from_bytes(v.as_bytes());
        let buf = &mut buf;
        let mut ins = Match::default();
        ins.from(buf)?;
        Ok(ins)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}
