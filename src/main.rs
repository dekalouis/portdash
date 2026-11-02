fn main() {
    println!("{}", greeting("reader"));
}

fn greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_by_name() {
        assert_eq!(greeting("Deka"), "Hello, Deka!");
    }
}
