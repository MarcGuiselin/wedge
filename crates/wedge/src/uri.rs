use url::Url;

/// Is this a valid web url?
fn is_http_url(url: &str) -> bool {
    match Url::parse(&url) {
        Ok(url) => {
            let scheme = url.scheme();
            scheme == "http" || scheme == "https"
        }
        Err(_) => false,
    }
}

/// Tries parsing web url from "microsoft-edge:" protocol
pub fn parse_ms_edge_url(url: &str) -> Option<String> {
    // Is valid ms-edge url?
    if !url.starts_with("microsoft-edge:") || url.contains(' ') {
        return None;
    }

    // Parse url from schema
    let url = {
        // microsoft-edge:?a=1&url=parsed_url&b=2
        if url.starts_with("microsoft-edge:?") {
            if let Ok(url) = Url::parse(url) {
                url.query_pairs()
                    .find_map(|pair| {
                        if pair.0 == "url" {
                            Some(pair.1.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            }
        }
        // microsoft-edge:parsed_url
        else {
            // Remove "microsoft-edge:" at start of string. Using string slices is fine for
            // these characters.
            String::from(&url["microsoft-edge:".len()..])
        }
    };

    // Valid url
    if is_http_url(&url) {
        Some(url)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_http_url() {
        assert!(is_http_url("http://example.com"));
        assert!(is_http_url(
            "https://some.example.org/path/sub/file.png?a=abc&x=%3F%3C%3E%3A%2F%5C"
        ));
        assert!(!is_http_url(r""));
        assert!(!is_http_url(r"file:///some.html"));
        assert!(!is_http_url(r"C:\Windows\system32\notepad.exe"));
    }

    #[test]
    fn test_invalid_ms_uri() {
        assert_eq!(None, parse_ms_edge_url(""));
        assert_eq!(None, parse_ms_edge_url("http://example.com"));
        assert_eq!(None, parse_ms_edge_url("abcdefg"));
        assert_eq!(None, parse_ms_edge_url("abcdefghijklmnohttp://example.com"));
        assert_eq!(None, parse_ms_edge_url("microsoft-edge:"));
        assert_eq!(None, parse_ms_edge_url("microsoft-edge:     "));
    }

    #[test]
    fn test_ms_uri_normal_schema() {
        assert_eq!(None, parse_ms_edge_url("microsoft-edge:?invalid_url"));
        assert_eq!(
            None,
            parse_ms_edge_url("microsoft-edge:?a=1&url=invalid_url&b=2")
        );
        assert_eq!(
            None,
            parse_ms_edge_url("microsoft-edge:? a=1&url=http%3A%2F%2Fexample.com&b=2")
        );
        assert_eq!(
            Some(String::from("http://example.com")),
            parse_ms_edge_url("microsoft-edge:http://example.com")
        );
        assert_eq!(
            Some(String::from("https://some.example.org/path/sub/file.png?a=abc&x=%3F%3C%3E%3A%2F%5C")),
            parse_ms_edge_url("microsoft-edge:https://some.example.org/path/sub/file.png?a=abc&x=%3F%3C%3E%3A%2F%5C")
        );
    }

    #[test]
    fn test_ms_uri_alt_schema() {
        assert_eq!(None, parse_ms_edge_url("microsoft-edge:invalid_url"));
        assert_eq!(
            None,
            parse_ms_edge_url("microsoft-edge: http://example.com")
        );
        assert_eq!(
            Some(String::from("http://example.com")),
            parse_ms_edge_url("microsoft-edge:?url=http%3A%2F%2Fexample.com&a=abc&b=2")
        );
        assert_eq!(
            Some(String::from(
                "https://some.example.org/path/sub/file.png?a=abc&x=%3F%3C%3E%3A%2F%5C"
            )),
            parse_ms_edge_url(
                "microsoft-edge:?a=test&url=https%3A%2F%2Fsome.example.org%2Fpath%2Fsub%2Ffile.\
                 png%3Fa%3Dabc%26x%3D%253F%253C%253E%253A%252F%255C&b=2"
            )
        );
    }

    #[test]
    fn test_vulnerabilities() {
        // Protect against arbitrary code execution vulnerabilities
        //   https://www.ctrl.blog/entry/edgedeflector-default-browser.html
        assert_eq!(None, parse_ms_edge_url("microsoft-edge:file:///some.html"));
        assert_eq!(None, parse_ms_edge_url("microsoft-edge:calc.exe"));
        assert_eq!(
            None,
            parse_ms_edge_url(r"microsoft-edge:C:\Windows\system32\notepad.exe")
        );
    }
}
