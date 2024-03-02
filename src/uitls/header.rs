pub fn contains(ovs: Option<&Vec<String>>, target: &str) -> bool {
    match ovs {
        Some(vs) => {
            for v in vs.iter() {
                if v == target || v.contains(target) {
                    return true;
                }
            }
            false
        }
        None => false,
    }
}
