pub fn highlight_matches(text: &str, query: &str) -> String {
    let query_lower = query.to_lowercase();
    let mut result = String::with_capacity(text.len());
    for line in text.lines() {
        let line_lower = line.to_lowercase();
        if let Some(pos) = line_lower.find(&query_lower) {
            let end = pos + query.len();
            if end <= line.len() {
                result.push_str(&line[..pos]);
                result.push_str("\x1b[1;33m");
                result.push_str(&line[pos..end]);
                result.push_str("\x1b[0m");
                result.push_str(&line[end..]);
            } else {
                result.push_str(line);
            }
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    result
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn マッチ箇所がANSIカラーでハイライトされる() {
        let result = highlight_matches("Hello World", "world");
        assert!(result.contains("\x1b[1;33mWorld\x1b[0m"));
    }

    #[test]
    fn 大文字小文字を区別せずにマッチする() {
        let result = highlight_matches("Testing テスト", "testing");
        assert!(result.contains("\x1b[1;33mTesting\x1b[0m"));
    }

    #[test]
    fn マッチしない行はそのまま返す() {
        let result = highlight_matches("no match here", "xyz");
        assert_eq!(result, "no match here\n");
    }

    #[test]
    fn 複数行でそれぞれマッチする() {
        let text = "first test\nsecond test\nno match";
        let result = highlight_matches(text, "test");
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[0].contains("\x1b[1;33m"));
        assert!(lines[1].contains("\x1b[1;33m"));
        assert!(!lines[2].contains("\x1b[1;33m"));
    }

    #[test]
    fn 空のクエリでは入力がそのまま返る() {
        let result = highlight_matches("Hello", "");
        assert!(result.contains("Hello"));
    }

    #[test]
    fn 各行の最初のマッチのみハイライトされる() {
        let result = highlight_matches("test test test", "test");
        let highlight_count = result.matches("\x1b[1;33m").count();
        assert_eq!(highlight_count, 1);
    }
}
