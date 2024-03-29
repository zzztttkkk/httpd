use pest::Parser;
use serde::de::Visitor;
use utils::anyhow;

#[derive(PartialEq, Eq, Debug)]
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

#[derive(PartialEq, Eq, Debug)]
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

#[derive(Debug)]
pub enum MultiValuePolicy {
    All,
    Any,
    First,
    Last,
    Nth(u16),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum LogicKind {
    And,
    Or,
}

#[derive(Debug)]
pub struct Match {
    pub name: String,
    pub logic: LogicKind,
    pub conds: Vec<Condation>,
}

impl Default for Match {
    fn default() -> Self {
        Self {
            name: Default::default(),
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

fn update_condition_left(ins: &mut Condation, pair: pest::iterators::Pair<'_, Rule>) {
    for ele in pair.into_inner() {
        match ele.as_rule() {
            Rule::ConditionKinds => match ele.as_str().to_lowercase().as_str() {
                "code" => {
                    ins.kind = CondationKind::Code;
                }
                "method" => {
                    ins.kind = CondationKind::Method;
                }
                "path" => {
                    ins.kind = CondationKind::Path;
                }
                "query" => {
                    ins.kind = CondationKind::Query;
                }
                "header" => {
                    ins.kind = CondationKind::Header;
                }
                "cookie" => {
                    ins.kind = CondationKind::Cookie;
                }
                "useragent" | "ua" => {
                    ins.kind = CondationKind::UserAgent;
                }
                "form" => {
                    ins.kind = CondationKind::FormBody;
                }
                "json" => {
                    ins.kind = CondationKind::JsonBody;
                }
                "ref" => {
                    ins.kind = CondationKind::MatchRef;
                }
                _ => {}
            },
            Rule::ConditionLeftParam => {
                for ev in ele.clone().into_inner() {
                    match ev.as_rule() {
                        Rule::ident => {
                            ins.left = ev.as_str().to_string();
                        }
                        Rule::MultiValuesOpts => {
                            let mvp = ev.as_str().to_lowercase();
                            match mvp.as_str() {
                                "all" => {
                                    ins.mvp = Some(MultiValuePolicy::All);
                                }
                                "any" => {
                                    ins.mvp = Some(MultiValuePolicy::Any);
                                }
                                "first" => {
                                    ins.mvp = Some(MultiValuePolicy::First);
                                }
                                "last" => {
                                    ins.mvp = Some(MultiValuePolicy::Last);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn update_condition_op(ins: &mut Condation, pair: pest::iterators::Pair<'_, Rule>) {}

fn make_condation(pair: pest::iterators::Pair<'_, Rule>) -> Condation {
    let mut ins = Condation::default();
    for ele in pair.into_inner() {
        match ele.as_rule() {
            Rule::ConditionLeft => {
                update_condition_left(&mut ins, ele.clone());
            }
            Rule::Options => match ele.as_str().to_lowercase().as_str() {
                "not" => {
                    ins.not = Some(true);
                }
                "trim" => {
                    ins.trim_space = Some(true);
                }
                "ignorecase" => {
                    ins.ignore_case = Some(true);
                }
                _ => {}
            },
            Rule::Op => {
                update_condition_op(&mut ins, ele.clone());
            }
            _ => {}
        }
    }
    return ins;
}

fn parse(txt: &str) -> anyhow::Result<Vec<Match>> {
    let pv = anyhow::result(MatchParser::parse(Rule::File, txt))?;
    let file = anyhow::option(
        pv.filter(|v| match v.as_rule() {
            Rule::File => true,
            _ => false,
        })
        .last(),
        "empty file",
    )?;

    let mut matchs = vec![];

    for item in file.into_inner() {
        match item.as_rule() {
            Rule::Match => {}
            _ => {
                continue;
            }
        }

        let mut ins = Match::default();

        for ele in item.clone().into_inner() {
            match ele.as_rule() {
                Rule::ident => {
                    ins.name = ele.as_str().to_string();
                }
                Rule::Logics => match ele.as_str().to_lowercase().as_str() {
                    "and" => {
                        ins.logic = LogicKind::And;
                    }
                    "or" => {
                        ins.logic = LogicKind::Or;
                    }
                    _ => {}
                },
                Rule::Condition => ins.conds.push(make_condation(ele.clone())),
                _ => {}
            }
        }

        matchs.push(ins)
    }

    Ok(matchs)
}

#[cfg(test)]
mod tests {
    use crate::config::r#match::parse;

    #[test]
    fn test_parse_match() {
        for ele in parse(
            r#"
IsWindows:and {
ua<platform> contains "windows";
}

IsAndroid:and {
ua<platform> contains "android";
}

IsWindowsOrAndroid:or {
ref<IsWindows>;

ref<IsAndroid>;
}

CanAcceptGzip:and {
header<accept-encoding:all> contains "gzip";
}

and {
path match "^/account/(?<name>\\w+)/index\\.html$";

ref<IsWindowsOrAndroid>;

ref<CanAcceptGzip>;

query<servers:all> in [
    "10", "11", "12"
];
}
"#,
        )
        .unwrap()
        {
            println!(">>>>>>>>>\r\n{:?}\r\n", ele);
        }
    }
}
