// it will work if I use
// fn print<'a>(mut stdout: Box<dyn Write + 'a>) {


#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Write;

    fn print(mut stdout: Box<dyn Write>) {
        writeln!(stdout, "Print").unwrap();
    }

    #[test]
    fn x() {
        let mut stdout = io::stdout();
        print(Box::new(stdout.lock()));
    }
}

