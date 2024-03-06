fn main() {
    let gitver = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("get git version failed");

    println!(
        ">>>>>>>>>>>>>>>>>>>>>>>>>>>>> git version: {}",
        std::str::from_utf8(gitver.stdout.as_slice()).unwrap()
    );
}
