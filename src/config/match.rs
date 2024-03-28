use pest::Parser;
use serde::de::Visitor;
use utils::anyhow;

#[derive(PartialEq, Eq)]
pub enum CondationKind {
    Code,
    Method,
    Path,
    Query,
    Header,
    Cookie,
    UserAgent,
    FormBody,
    JsonBody,
    MatchRef,
}

#[derive(PartialEq, Eq)]
pub enum OpKind {
    Eq,
    Lt,
    Gt,
    Le,
    Ge,

    In,
    Contains,
    Match,

    IntLt,
    IntGt,
    IntLe,
    IntGe,
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
    pub ignore_case: Option<bool>,
    pub trim_space: Option<bool>,
    pub mvp: Option<MultiValuePolicy>,
    pub op: Option<OpKind>,
    pub right: Option<Vec<String>>,
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
            ignore_case: Default::default(),
            trim_space: Default::default(),
        }
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

#[derive(pest_derive::Parser)]
#[grammar = "./src/config/match.pest"]
pub struct MatchParser;

#[derive(Default)]
pub struct MatchVisitor {}

fn unwrap<E: serde::de::Error, T>(v: anyhow::Result<T>) -> Result<T, E> {
    match v {
        Ok(v) => Ok(v),
        Err(e) => Err(serde::de::Error::custom(format!("{}", e))),
    }
}

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
        let mut ins = Match::default();
        let txt;
        if v.starts_with("file://") || v.starts_with("FILE://") {
            txt = unwrap(anyhow::result(std::fs::read_to_string(&v[7..])))?;
        } else {
            txt = v.to_string();
        }

        let pairs = unwrap(anyhow::result(MatchParser::parse(Rule::File, &txt)))?;

        for pair in pairs.into_iter() {
            match pair.as_rule() {
                Rule::Match => todo!(),
                Rule::File => todo!(),
                _ => {}
            }
        }

        Ok(ins)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::MatchParser;

    #[test]
    fn test_parse_match() {
        let pairs = MatchParser::parse(
            super::Rule::File,
            "
and {
    code == 200;
}
",
        )
        .unwrap();

        for pair in pairs.clone().flatten() {
            match pair.as_rule() {
                super::Rule::Match => {}
                _ => {
                    println!("xxx");
                    continue;
                }
            }

            let matchs = pair.into_inner();

            for pair in matchs.clone().flatten() {
                println!("MatchPart: {}", pair.as_str());
            }
        }
    }
}
