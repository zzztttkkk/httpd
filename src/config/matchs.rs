use crate::utils::anyhow;
use pest::Parser;
use serde::{de::Visitor, Deserialize};

#[derive(PartialEq, Eq, Debug, Clone)]
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

#[derive(PartialEq, Eq, Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum MultiValuePolicy {
    All,
    Any,
    First,
    Last,
    Nth(i16),
}

#[derive(Debug, Clone, Default)]
pub struct Condation {
    pub kind: Option<CondationKind>,
    pub left: String,
    pub not: Option<bool>,
    pub ignore_case: Option<bool>,
    pub trim_space: Option<bool>,
    pub mvp: Option<MultiValuePolicy>,
    pub op: Option<OpKind>,
    pub right: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum LogicKind {
    And,
    Or,
}

#[derive(Debug, Clone, Default)]
pub struct Match {
    pub name: String,
    pub logic: Option<LogicKind>,
    pub conds: Vec<Condation>,
}

#[derive(Default)]
pub struct MatchVisitor;

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
        Ok(ins)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}

impl<'de> Deserialize<'de> for Match {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(MatchVisitor)
    }
}
