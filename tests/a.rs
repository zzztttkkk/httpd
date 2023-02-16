#[cfg(test)]
mod tests {

    #[test]
    fn x() {
        let tmp = "/";
        match tmp.find('/') {
            None => {}
            Some(idx) => {
                println!("{}", &tmp[0..idx + 1])
            }
        }
    }
}
