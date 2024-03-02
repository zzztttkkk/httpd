#[derive(Debug, Clone)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[inline]
pub fn result<T>(r: std::result::Result<T, impl std::fmt::Debug>) -> Result<T> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => Err(Error(format!("{:?}", e))),
    }
}

#[inline]
pub fn option<T>(o: Option<T>, msg: &str) -> Result<T> {
    match o {
        Some(v) => Ok(v),
        None => Err(Error(msg.to_string())),
    }
}
