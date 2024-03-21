use std::str::FromStr;

use crate::anyhow;

pub fn namewithoutext(fp: &str) -> anyhow::Result<String> {
    let fp = anyhow::result(std::path::PathBuf::from_str(fp))?;

    let basename = anyhow::option(
        anyhow::option(fp.file_name(), "filename failed")?.to_str(),
        "to str failed",
    )?;

    match fp.extension() {
        Some(ext) => {
            let ext = anyhow::option(ext.to_str(), "ext to str failed")?;
            return Ok(basename[..basename.len() - ext.len() - 1].to_string());
        }
        None => {
            return Ok(basename.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::paths::namewithoutext;

    #[test]
    fn test_namewithoutext() {
        println!("{}", namewithoutext("./a.txt").unwrap());
        println!("{}", namewithoutext("./a..txt").unwrap());
        println!("{}", namewithoutext("./a.txt..").unwrap());
        println!("{}", namewithoutext("./a").unwrap());
    }
}
