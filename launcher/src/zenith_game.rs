//! Zenith's release feed and independently signed game and source downloads.

use crate::backend::{self, Release};
use crate::i18n::{tr, trf};
use horizon_core::release::ZENITH_TAG_PREFIX;
use horizon_release_sign::{PublicKey, SIGNATURE, SUMS};
use serde_json::Value;
use std::io::Read;
use std::path::Path;

pub const REPO: &str = "joy-arz/zenith-rust-edition";
pub const TOKEN_ENV: &str = "ZENITH_ENGINE_TOKEN";
const TOKEN_ENTRY: &str = "zenith-engine-github-access";
const API: &str = "https://api.github.com";
const CHANNEL_SIGNATURE: &str = "SHA256SUMS.zenith.sig";
const FIRST_PREVIEW: &str = "zenith-game@0.1.0-preview.1";

pub fn version(tag: &str) -> Option<&str> {
    let text = tag.strip_prefix(ZENITH_TAG_PREFIX)?;
    let parsed = semver::Version::parse(text).ok()?;
    if !parsed.build.is_empty() {
        return None;
    }
    if let Some(number) = parsed.pre.as_str().strip_prefix("preview.") {
        (number.parse::<u64>().ok()? > 0).then_some(text)
    } else {
        parsed.pre.is_empty().then_some(text)
    }
}

pub fn is_tag(tag: &str) -> bool {
    version(tag).is_some()
}

pub fn label(tag: &str) -> Option<String> {
    version(tag).map(|version| trf("Zenith Rust Edition {version}", &[("version", version)]))
}

pub fn save_access_token(token: &str) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty()
        || token.len() > 1024
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(
            tr("Enter a GitHub token with read access to the Zenith game repository.").into(),
        );
    }
    let store = crate::account::store();
    if !store.persistent() {
        return Err(tr("The launcher cannot save Zenith repository access on this device.").into());
    }
    store.set(TOKEN_ENTRY, token)
}

pub fn forget_access_token() -> Result<(), String> {
    crate::account::store().delete(TOKEN_ENTRY)
}

fn request(path: &str, accept: &str) -> Result<ureq::Response, String> {
    let url = format!("{API}/repos/{REPO}/{path}");
    let agent = backend::agent(&url);
    let mut request = agent.get(&url).set("Accept", accept);
    let token = std::env::var(TOKEN_ENV)
        .ok()
        .filter(|token| !token.trim().is_empty())
        .or_else(|| crate::account::store().get(TOKEN_ENTRY).ok().flatten());
    if let Some(token) = token {
        request = request.set("Authorization", &format!("Bearer {}", token.trim()));
    }
    request.call().map_err(|error| match error {
        ureq::Error::Status(401 | 403 | 404, _) => {
            tr("Zenith previews require access to the Zenith game repository.").to_string()
        }
        ureq::Error::Status(code, _) => trf(
            "GitHub returned HTTP {status} for the Zenith game repository.",
            &[("status", &code.to_string())],
        ),
        ureq::Error::Transport(_) => tr("Could not connect to the Zenith game repository.").into(),
    })
}

fn json(path: &str) -> Result<Value, String> {
    request(path, "application/vnd.github+json")?
        .into_json()
        .map_err(|error| error.to_string())
}

pub(crate) fn asset_name(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("linux", "x86_64") => Some("zenith-linux-x86_64.tar.gz"),
        ("macos", "aarch64") => Some("zenith-macos-arm64.tar.gz"),
        ("macos", "x86_64") => Some("zenith-macos-x64.tar.gz"),
        ("windows", "aarch64") => Some("zenith-windows-arm64.tar.gz"),
        ("windows", "x86_64") => Some("zenith-windows-x64.tar.gz"),
        _ => None,
    }
}

fn has_asset(release: &Value, name: &str) -> bool {
    asset_id(release, name).is_some()
}

fn asset_id(release: &Value, name: &str) -> Option<u64> {
    release["assets"]
        .as_array()?
        .iter()
        .find(|asset| asset["name"].as_str() == Some(name))?["id"]
        .as_u64()
}

fn signature(release: &Value, tag: &str) -> Option<(&'static str, &'static str)> {
    if has_asset(release, CHANNEL_SIGNATURE) {
        Some((
            CHANNEL_SIGNATURE,
            include_str!("../zenith-engine-release-keys.txt"),
        ))
    } else if tag == FIRST_PREVIEW && has_asset(release, SIGNATURE) {
        Some((SIGNATURE, include_str!("../zenith-first-preview-key.txt")))
    } else {
        None
    }
}

pub fn releases() -> Result<Vec<Release>, String> {
    let list = json("releases?per_page=100")?;
    let assets = backend::zenith_game_assets();
    Ok(releases_from(&list, assets.0, assets.1))
}

fn releases_from(list: &Value, asset: Option<&str>, fallback: Option<&str>) -> Vec<Release> {
    let Some(list) = list.as_array() else {
        return Vec::new();
    };
    let trusted: Vec<_> = list
        .iter()
        .filter(|item| {
            item["tag_name"]
                .as_str()
                .filter(|tag| is_tag(tag))
                .is_some_and(|tag| signature(item, tag).is_some() && has_asset(item, SUMS))
        })
        .collect();
    let list = serde_json::to_value(trusted).expect("GitHub JSON serializes as JSON");
    let mut releases = backend::parse_releases(&list, asset, is_tag);
    for (release, other) in releases
        .iter_mut()
        .zip(backend::parse_releases(&list, fallback, is_tag))
    {
        if release.asset.is_none() && release.tag == other.tag {
            release.asset = other.asset;
        }
    }
    for release in &mut releases {
        // A Zenith preview stays on the Zenith channel once selected.
        release.name = label(&release.tag).unwrap_or_else(|| release.tag.clone());
    }
    releases
}

fn asset(release: &Value, name: &str) -> Result<ureq::Response, String> {
    let id = asset_id(release, name).ok_or_else(|| {
        trf(
            "The Zenith release has no {file} attachment.",
            &[("file", name)],
        )
    })?;
    // ureq strips Authorization on redirects, including redirects to GitHub's download storage.
    request(&format!("releases/assets/{id}"), "application/octet-stream")
}

fn small_asset(release: &Value, name: &str) -> Result<Vec<u8>, String> {
    const LIMIT: u64 = 1 << 20;
    let mut bytes = Vec::new();
    asset(release, name)?
        .into_reader()
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err(tr("Zenith release metadata is too large.").into());
    }
    Ok(bytes)
}

pub(crate) struct VerifiedRelease {
    tag: String,
    release: Value,
    sums: Vec<u8>,
    signature: String,
    keys: Vec<PublicKey>,
}

impl VerifiedRelease {
    pub(crate) fn load(tag: &str) -> Result<Self, String> {
        if !is_tag(tag) {
            return Err(tr("Invalid Zenith game version.").into());
        }
        let release = json(&format!("releases/tags/{tag}"))?;
        if release["tag_name"].as_str() != Some(tag) || release["draft"].as_bool() != Some(false) {
            return Err(tr("GitHub did not return the requested Zenith release.").into());
        }
        let (name, keys) = signature(&release, tag)
            .ok_or_else(|| tr("This Zenith release has no trusted signature.").to_string())?;
        let keys = horizon_release_sign::parse_keys(keys)?;
        let sums = small_asset(&release, SUMS)?;
        let signature =
            String::from_utf8(small_asset(&release, name)?).map_err(|error| error.to_string())?;
        Ok(Self {
            tag: tag.to_owned(),
            release,
            sums,
            signature,
            keys,
        })
    }

    fn digest(&self, name: &str) -> Result<String, String> {
        horizon_release_sign::signed_digest(
            &self.tag,
            name,
            Some(&self.sums),
            Some(&self.signature),
            &self.keys,
        )
    }

    pub(crate) fn source_revision(&self) -> Result<String, String> {
        let expected = self.digest("preview.json")?;
        let bytes = small_asset(&self.release, "preview.json")?;
        horizon_release_sign::check(
            "preview.json",
            &expected,
            &horizon_release_sign::sha256_hex(&bytes),
        )?;
        let manifest: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if manifest["tag"].as_str() != Some(&self.tag) {
            return Err(tr("Zenith release metadata names a different game version.").into());
        }
        manifest["source_commit"]
            .as_str()
            .filter(|sha| {
                sha.len() == 40
                    && sha
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
            .map(str::to_owned)
            .ok_or_else(|| tr("The Zenith release has no source revision.").into())
    }

    pub(crate) fn download(
        &self,
        name: &str,
        to: &Path,
        progress: &impl Fn(f32),
    ) -> Result<(), String> {
        let sha256 = self.digest(name)?;
        let response = asset(&self.release, name)?;
        let size = response
            .header("Content-Length")
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);
        backend::save_checked(response.into_reader(), size, name, &sha256, to, progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_pinned_legacy_preview_accepts_its_old_signature() {
        let unsigned = serde_json::json!([
            {
                "tag_name": FIRST_PREVIEW,
                "assets": [{"name": SIGNATURE, "id": 1}, {"name": SUMS, "id": 2}]
            },
            {
                "tag_name": "zenith-game@0.2.0-preview.1",
                "assets": [{"name": SIGNATURE, "id": 3}, {"name": SUMS, "id": 4}]
            }
        ]);
        let releases = releases_from(&unsigned, None, None);
        assert_eq!(
            releases.iter().map(|r| r.tag.as_str()).collect::<Vec<_>>(),
            [FIRST_PREVIEW]
        );
        assert!(!is_tag("zenith-game@0.2.0-preview.1/../escape"));
    }
}
