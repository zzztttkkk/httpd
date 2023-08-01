use bytes::BufMut;

pub fn read_all(mut r: impl std::io::Read, bufsize: usize) -> std::io::Result<Vec<u8>> {
    let mut tmp = Vec::new();
    let mut buf: Vec<u8> = vec![0; std::cmp::max(bufsize, 1024)];

    loop {
        match r.read(&mut buf[..]) {
            Ok(rl) => {
                if rl == 0 {
                    break;
                }
                tmp.put_slice(&buf[..rl]);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    return Ok(tmp);
}

#[cfg(test)]
mod tests {
    use crate::utils::read_all;

    #[test]
    fn test_read_all() {
        let mut fp = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
        let bytes = read_all(&mut fp, 1024 * 1024).unwrap();
        println!("{} {}", fp.metadata().unwrap().len(), bytes.len());
    }
}
