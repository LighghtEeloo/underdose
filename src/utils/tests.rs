pub fn remove_tutorial(content: &str) -> String {
    let mut content: Vec<_> = content.split("\n").collect();
    loop {
        if let Some(last) = content.last() {
            if last.is_empty() {
                content.pop();
            } else if last.starts_with("[tutorial]") {
                content.pop();
                break;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    content.join("\n")
}