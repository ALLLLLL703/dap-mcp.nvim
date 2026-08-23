/// Returns whether a Host header targets loopback on the expected port.
#[must_use]
pub fn is_loopback_host(header: Option<&str>, expected_port: u16) -> bool {
    let Some(raw_header) = header else {
        return false;
    };
    let header = raw_header.trim().to_ascii_lowercase();
    let (host, port) = split_host_and_port(&header);
    matches!(host, "localhost" | "127.0.0.1" | "::1")
        && port.is_none_or(|value| value == expected_port.to_string())
}

/// Returns whether an optional Origin header uses an HTTP loopback host.
#[must_use]
pub fn is_loopback_origin(header: Option<&str>) -> bool {
    let Some(origin) = header else {
        return true;
    };
    let Ok(uri) = origin.parse::<axum::http::Uri>() else {
        return false;
    };
    matches!(uri.scheme_str(), Some("http" | "https"))
        && uri
            .host()
            .is_some_and(|host| matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]"))
}

/// Splits IPv4, hostname, and bracketed IPv6 Host header forms.
fn split_host_and_port(header: &str) -> (&str, Option<&str>) {
    if let Some(rest) = header.strip_prefix('[')
        && let Some((host, suffix)) = rest.split_once(']')
    {
        return (host, suffix.strip_prefix(':'));
    }
    match header.split_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (header, None),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_loopback_host, is_loopback_origin};

    /// Accepts supported loopback Host header forms.
    #[test]
    fn accepts_loopback_hosts() {
        assert!(is_loopback_host(Some("localhost:3001"), 3001));
        assert!(is_loopback_host(Some("127.0.0.1"), 3001));
        assert!(is_loopback_host(Some("[::1]:3001"), 3001));
    }

    /// Rejects missing, remote, and wrong-port Host headers.
    #[test]
    fn rejects_unsafe_hosts() {
        assert!(!is_loopback_host(None, 3001));
        assert!(!is_loopback_host(Some("attacker.example:3001"), 3001));
        assert!(!is_loopback_host(Some("localhost:3002"), 3001));
    }

    /// Accepts omitted or loopback HTTP origins and rejects others.
    #[test]
    fn validates_origins() {
        assert!(is_loopback_origin(None));
        assert!(is_loopback_origin(Some("http://localhost:3001")));
        assert!(is_loopback_origin(Some("http://[::1]:3001")));
        assert!(!is_loopback_origin(Some("https://attacker.example")));
        assert!(!is_loopback_origin(Some("not a uri")));
    }
}
