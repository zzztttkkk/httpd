#[derive(Debug, Clone)]
pub struct MsgError(pub String);

impl std::fmt::Display for MsgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub type Result<T> = std::result::Result<T, MsgError>;

#[inline]
pub fn result<T>(r: std::result::Result<T, impl std::fmt::Debug>) -> Result<T> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => Err(MsgError(format!("{:?}", e))),
    }
}

#[inline]
pub fn option<T>(o: Option<T>, msg: &str) -> Result<T> {
    match o {
        Some(v) => Ok(v),
        None => Err(MsgError(msg.to_string())),
    }
}
