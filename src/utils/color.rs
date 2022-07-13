pub fn blue(msg: String) -> String {
    return format!("\u{001b}[34m{}\u{001b}[0m", msg);
}

pub fn magenta(msg: String) -> String {
    return format!("\u{001b}[31m{}\u{001b}[0m", msg);
}
