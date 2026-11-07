//! Portable invitations for directly reachable game addresses, including Tailscale.
//! Links are pasted into the launcher's Multiplayer address field or `play --server`.
use std::fs;
use std::net::{IpAddr, Ipv6Addr};
use std::path::Path;

const PREFIX: &str = "zenith://server/";
const LEGACY_PREFIX: &str = "horizon://server/";
const FILE: &str = "horizon-share-address.txt";

/// Validate without DNS, network access, URL decoding, or credentials.
pub fn address(input: &str) -> Result<String, String> {
    let input = input.trim();
    let invalid = || "Use a public IP or DNS name with a TCP port from 1 to 65535.".to_string();
    let (host, port) = if let Some(rest) = input.strip_prefix('[') {
        let (host, port) = rest.split_once("]:").ok_or_else(invalid)?;
        host.parse::<Ipv6Addr>().map_err(|_| invalid())?;
        (format!("[{host}]"), port)
    } else {
        let (host, port) = input.rsplit_once(':').ok_or_else(invalid)?;
        let host = host.trim_end_matches('.');
        if host.is_empty() || host.len() > 253 || host.contains(':') {
            return Err(invalid());
        }
        if host.parse::<IpAddr>().is_err()
            && (host.chars().all(|c| c.is_ascii_digit() || c == '.')
                || !host.split('.').all(|label| {
                    !label.is_empty()
                        && label.len() <= 63
                        && !label.starts_with('-')
                        && !label.ends_with('-')
                        && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                }))
        {
            return Err(invalid());
        }
        (host.to_ascii_lowercase(), port)
    };
    if port.is_empty() || !port.bytes().all(|c| c.is_ascii_digit()) {
        return Err(invalid());
    }
    let port = port.parse::<u16>().map_err(|_| invalid())?;
    if port == 0 {
        return Err(invalid());
    }
    Ok(format!("{host}:{port}"))
}

pub fn link(input: &str) -> Result<String, String> {
    Ok(format!("{PREFIX}{}", address(input)?))
}

/// Ordinary game addresses (including invite codes and SRV names) retain their existing behavior.
pub fn resolve(input: &str) -> Result<String, String> {
    let input = input.trim();
    match input
        .strip_prefix(PREFIX)
        .or_else(|| input.strip_prefix(LEGACY_PREFIX))
    {
        Some(endpoint) => address(endpoint),
        None if input.contains("://") => {
            Err("Use a zenith://server/HOST:PORT link or a game server address.".into())
        }
        None => Ok(input.to_string()),
    }
}

pub fn save(dir: &Path, input: &str) -> Result<String, String> {
    let endpoint = address(input)?;
    let path = dir.join(FILE);
    let temporary = dir.join(format!("{FILE}.tmp"));
    let mut file = fs::File::create(&temporary).map_err(|e| e.to_string())?;
    use std::io::Write;
    file.write_all(endpoint.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(temporary, path).map_err(|e| e.to_string())?;
    Ok(endpoint)
}

pub fn load(dir: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(dir.join(FILE)) {
        Ok(text) => address(&text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invitations_keep_external_ports_and_ipv6() {
        for (input, expected) in [
            ("Play.Example.COM:25570", "play.example.com:25570"),
            ("203.0.113.12:25565", "203.0.113.12:25565"),
            ("[2001:db8::1]:25580", "[2001:db8::1]:25580"),
        ] {
            assert_eq!(resolve(&link(input).unwrap()).unwrap(), expected);
        }
        assert_eq!(resolve("play.example.com").unwrap(), "play.example.com");
        assert_eq!(resolve("friend:ABC").unwrap(), "friend:ABC");
        assert_eq!(
            link("play.example.com:25570").unwrap(),
            "zenith://server/play.example.com:25570"
        );
        assert_eq!(
            resolve("horizon://server/play.example.com:25570").unwrap(),
            "play.example.com:25570"
        );
    }

    #[test]
    fn malformed_invitations_are_rejected() {
        for input in [
            "",
            "host",
            "host:0",
            "host:65536",
            "host:-1",
            "host: 12",
            "user@host:25565",
            "host:25565/path",
            "host:25565?x=1",
            "host:25565#fragment",
            "a b:25565",
            "-host:25565",
            "host..com:25565",
            "[bad::ip]:25565",
            "2001:db8::1:25565",
            "999.1.2.3:25565",
        ] {
            assert!(address(input).is_err(), "{input}");
            assert!(resolve(&format!("{PREFIX}{input}")).is_err(), "{input}");
        }
    }

    #[test]
    fn saved_address_survives_reload_and_invalid_edits() {
        let dir = std::env::temp_dir().join(format!("horizon-share-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(
            save(&dir, "play.example.com:25570").unwrap(),
            "play.example.com:25570"
        );
        assert!(save(&dir, "https://example.com").is_err());
        assert_eq!(load(&dir).unwrap().unwrap(), "play.example.com:25570");
        fs::remove_dir_all(dir).unwrap();
    }
}
