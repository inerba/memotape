//! Controllo aggiornamenti (ticket 11): l'ultima release di GitHub contro la versione in esecuzione.
//! Senza Tauri. Ogni errore (rete, rate limit, JSON) vale "nessun aggiornamento".

use std::time::Duration;

use semver::Version;
use serde::{Deserialize, Serialize};
use specta::Type;

const LATEST_URL: &str = "https://api.github.com/repos/inerba/memotape/releases/latest";
/// Le pagine che `open_update` accetta di aprire.
pub const RELEASES_PREFIX: &str = "https://github.com/inerba/memotape/";

/// Una versione più recente di quella in esecuzione.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct UpdateInfo {
    pub version: String,
    /// La pagina della release.
    pub url: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
}

/// La versione di `tag` (con o senza `v`) se è una release stabile più recente di `current`.
fn newer(current: &str, tag: &str) -> Option<Version> {
    let candidate = Version::parse(tag.strip_prefix('v').unwrap_or(tag)).ok()?;
    let current = Version::parse(current).ok()?;
    (candidate.pre.is_empty() && candidate.cmp_precedence(&current).is_gt()).then_some(candidate)
}

fn parse(json: &str, current: &str) -> Option<UpdateInfo> {
    let release: Release = serde_json::from_str(json).ok()?;
    if release.prerelease || release.draft || !release.html_url.starts_with(RELEASES_PREFIX) {
        return None;
    }
    Some(UpdateInfo {
        version: newer(current, &release.tag_name)?.to_string(),
        url: release.html_url,
    })
}

/// Chiede a GitHub l'ultima release; `None` se non ce n'è una più nuova o non si può sapere.
pub async fn check(current: &str) -> Option<UpdateInfo> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("memotape/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let response = client
        .get(LATEST_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    parse(&response.text().await.ok()?, current)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, extra: &str) -> String {
        format!(r#"{{"tag_name":"{tag}","html_url":"{RELEASES_PREFIX}releases/tag/{tag}"{extra}}}"#)
    }

    #[test]
    fn il_prefisso_v_si_toglie() {
        assert_eq!(newer("0.1.0", "v0.2.0"), Version::parse("0.2.0").ok());
        assert_eq!(newer("0.1.0", "0.2.0"), Version::parse("0.2.0").ok());
    }

    #[test]
    fn versione_uguale_o_vecchia_non_e_un_aggiornamento() {
        assert_eq!(newer("0.2.0", "v0.2.0"), None);
        assert_eq!(newer("0.2.0", "v0.1.9"), None);
    }

    #[test]
    fn le_prerelease_si_ignorano() {
        assert_eq!(newer("0.1.0", "v0.2.0-beta.1"), None);
        assert_eq!(
            parse(&release("v0.2.0", r#","prerelease":true"#), "0.1.0"),
            None
        );
        assert_eq!(parse(&release("v0.2.0", r#","draft":true"#), "0.1.0"), None);
    }

    #[test]
    fn dalla_prerelease_alla_stabile_si_aggiorna() {
        assert!(newer("0.2.0-beta.1", "v0.2.0").is_some());
    }

    #[test]
    fn una_release_piu_nuova_porta_versione_e_link() {
        let info = parse(&release("v0.2.0", ""), "0.1.0").unwrap();
        assert_eq!(info.version, "0.2.0");
        assert_eq!(info.url, format!("{RELEASES_PREFIX}releases/tag/v0.2.0"));
    }

    #[test]
    fn risposte_inutilizzabili_sono_silenziose() {
        assert_eq!(parse("not json", "0.1.0"), None);
        assert_eq!(parse(r#"{"message":"rate limit exceeded"}"#, "0.1.0"), None);
        assert_eq!(parse(&release("latest", ""), "0.1.0"), None);
    }

    #[test]
    fn il_link_deve_stare_nel_repo() {
        let json = r#"{"tag_name":"v9.0.0","html_url":"https://evil.example/x"}"#;
        assert_eq!(parse(json, "0.1.0"), None);
    }
}
