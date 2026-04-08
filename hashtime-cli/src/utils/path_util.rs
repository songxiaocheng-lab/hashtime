pub fn parse_hash_fields(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    s.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn parse_time_fields(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    s.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hash_fields_single() {
        let result = parse_hash_fields("md5");
        assert_eq!(result, vec!["md5"]);
    }

    #[test]
    fn test_parse_hash_fields_multiple() {
        let result = parse_hash_fields("md5,sha256,sha512");
        assert_eq!(result, vec!["md5", "sha256", "sha512"]);
    }

    #[test]
    fn test_parse_hash_fields_with_whitespace() {
        let result = parse_hash_fields("md5 , sha256 , sha512");
        assert_eq!(result, vec!["md5", "sha256", "sha512"]);
    }

    #[test]
    fn test_parse_hash_fields_case_insensitive() {
        let result = parse_hash_fields("MD5,SHA256");
        assert_eq!(result, vec!["md5", "sha256"]);
    }

    #[test]
    fn test_parse_hash_fields_empty() {
        let result = parse_hash_fields("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_time_fields_single() {
        let result = parse_time_fields("birthtime");
        assert_eq!(result, vec!["birthtime"]);
    }

    #[test]
    fn test_parse_time_fields_multiple() {
        let result = parse_time_fields("birthtime,mtime");
        assert_eq!(result, vec!["birthtime", "mtime"]);
    }

    #[test]
    fn test_parse_time_fields_with_whitespace() {
        let result = parse_time_fields("birthtime , mtime");
        assert_eq!(result, vec!["birthtime", "mtime"]);
    }

    #[test]
    fn test_parse_time_fields_empty() {
        let result = parse_time_fields("");
        assert!(result.is_empty());
    }
}
