//! Il protocollo `bino` del player: serve il `mix.ogg` di un Bino al tag `<audio>`, con le richieste
//! `Range` per lo spostamento. L'URL è `http://bino.localhost/<percorso del Bino>` (`convertFileSrc`).
//! E la Forma d'onda che fa da barra di avanzamento.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use tauri::http::{Request, Response, header};

use crate::audio_toolkit::{decode, forma_onda};
use crate::bino::{self, Mix};
use crate::error::AppError;
use crate::managers::activity::Activity;

/// Il massimo di byte di una risposta: wry passa il corpo intero in memoria.
const MAX_LEN: u64 = 1024 * 1024;

/// La risposta a una richiesta su un mix di `total` byte: lo stato, il `Content-Range` e il tratto
/// da leggere.
#[derive(Debug, PartialEq, Eq)]
pub struct Window {
    pub status: u16,
    pub content_range: Option<String>,
    pub start: u64,
    pub len: u64,
}

/// La finestra chiesta dall'intestazione `Range`. Senza `Range`, o con una che non si capisce
/// (più intervalli, unità diversa da `bytes`), il mix intero con 200, come permette RFC 9110; con
/// `Range` al più `MAX_LEN` byte con 206, o 416 se il tratto comincia oltre la fine.
pub fn window(range: Option<&str>, total: u64) -> Window {
    let whole = Window {
        status: 200,
        content_range: None,
        start: 0,
        len: total,
    };
    let Some((first, last)) = range.and_then(parse_range) else {
        return whole;
    };
    let (start, end) = match (first, last) {
        // `bytes=-n`: gli ultimi `n` byte.
        (None, Some(n)) if n > 0 => (total.saturating_sub(n), total),
        (Some(a), last) if a < total => (a, last.map_or(total, |b| b.saturating_add(1).min(total))),
        _ => {
            return Window {
                status: 416,
                content_range: Some(format!("bytes */{total}")),
                start: 0,
                len: 0,
            };
        }
    };
    let len = (end - start).min(MAX_LEN);
    Window {
        status: 206,
        content_range: Some(format!("bytes {start}-{}/{total}", start + len - 1)),
        start,
        len,
    }
}

/// `bytes=a-b`, `bytes=a-` o `bytes=-n` come (inizio, fine compresa) o (`None`, n); `None` se non è
/// un solo intervallo valido.
fn parse_range(range: &str) -> Option<(Option<u64>, Option<u64>)> {
    let (first, last) = range.trim().strip_prefix("bytes=")?.split_once('-')?;
    let number = |s: &str| -> Option<Option<u64>> {
        let s = s.trim();
        if s.is_empty() {
            Some(None)
        } else {
            s.parse().ok().map(Some)
        }
    };
    let (first, last) = (number(first)?, number(last)?);
    match (first, last) {
        (None, None) => None,
        (Some(a), Some(b)) if b < a => None,
        parsed => Some(parsed),
    }
}

/// Apre il Bino `path`, legge la finestra di `mix.ogg` chiesta da `range` e lo chiude: un Bino
/// tenuto aperto non si potrebbe riscrivere, rinominare o mandare nel Cestino.
pub fn read(path: &Path, range: Option<&str>) -> Result<(Window, Vec<u8>), AppError> {
    let mut mix = Mix::open(path)?;
    let window = window(range, mix.len());
    let mut bytes = vec![0; usize::try_from(window.len).unwrap_or(0)];
    mix.seek(SeekFrom::Start(window.start))
        .and_then(|_| mix.read_exact(&mut bytes))
        .map_err(|e| AppError::UnreadableFile(format!("{}: {e}", path.display())))?;
    Ok((window, bytes))
}

/// La Forma d'onda del Bino `path` in `count` valori. Un Bino che non la ha la calcola dal mix e
/// prova una volta a salvarla, se `activity` non lavora su quel Bino; se non riesce, la ricalcolerà
/// alla prossima apertura (ADR-0010).
pub fn forma_onda(path: &Path, count: usize, activity: &Activity) -> Result<Vec<f32>, AppError> {
    if let Some(values) = bino::forma_onda(path) {
        return Ok(forma_onda::regroup(&values, count));
    }
    let values = decode::peaks(path, forma_onda::VALORI)?;
    let saved = activity
        .write(path)
        .and_then(|_writing| bino::save_forma_onda(path, &values));
    if let Err(e) = saved {
        log::warn!("Forma d'onda non salvata in {}: {e}", path.display());
    }
    Ok(forma_onda::regroup(&values, count))
}

/// La risposta del protocollo a `request`: serve solo il mix di un `.bino`.
pub fn respond(request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let path = request.uri().path().trim_start_matches('/');
    let path = PathBuf::from(
        percent_encoding::percent_decode_str(path)
            .decode_utf8_lossy()
            .as_ref(),
    );
    let builder = Response::builder()
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_TYPE, "audio/ogg");
    let range = request
        .headers()
        .get(header::RANGE)
        .and_then(|r| r.to_str().ok());
    let response = if bino::is_bino(&path) {
        match read(&path, range) {
            Ok((window, bytes)) => {
                let builder = builder.status(window.status);
                match window.content_range {
                    Some(content_range) => builder.header(header::CONTENT_RANGE, content_range),
                    None => builder,
                }
                .body(bytes)
            }
            Err(e) => {
                log::warn!("protocollo bino: {e:?}");
                builder.status(404).body(Vec::new())
            }
        }
    } else {
        builder.status(403).body(Vec::new())
    };
    response.unwrap_or_else(|_| Response::new(Vec::new()))
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::{sine, temp_dir};
    use crate::bino::{Document, Modalita};
    use crate::managers::settings::SpeechLanguage;
    use crate::transcript::Ingresso;

    fn win(status: u16, content_range: Option<&str>, start: u64, len: u64) -> Window {
        Window {
            status,
            content_range: content_range.map(Into::into),
            start,
            len,
        }
    }

    #[test]
    fn la_finestra_segue_la_richiesta_range() {
        assert_eq!(window(None, 1000), win(200, None, 0, 1000));
        assert_eq!(
            window(Some("bytes=0-"), 1000),
            win(206, Some("bytes 0-999/1000"), 0, 1000)
        );
        assert_eq!(
            window(Some("bytes=10-19"), 1000),
            win(206, Some("bytes 10-19/1000"), 10, 10)
        );
        assert_eq!(
            window(Some("bytes=990-5000"), 1000),
            win(206, Some("bytes 990-999/1000"), 990, 10)
        );
        assert_eq!(
            window(Some("bytes=-100"), 1000),
            win(206, Some("bytes 900-999/1000"), 900, 100)
        );
        assert_eq!(
            window(Some("bytes=-5000"), 1000),
            win(206, Some("bytes 0-999/1000"), 0, 1000)
        );
        let beyond = win(416, Some("bytes */1000"), 0, 0);
        assert_eq!(window(Some("bytes=1000-"), 1000), beyond);
        assert_eq!(window(Some("bytes=-0"), 1000), beyond);
        assert_eq!(
            window(Some("bytes=0-18446744073709551615"), 1000),
            win(206, Some("bytes 0-999/1000"), 0, 1000)
        );
        assert_eq!(
            window(Some("bytes=0-"), 0),
            win(416, Some("bytes */0"), 0, 0)
        );
        // Non capite: il mix intero.
        for odd in [
            "items=0-1",
            "bytes=0-1,5-6",
            "bytes=5-1",
            "bytes=-",
            "bytes=a-",
        ] {
            assert_eq!(window(Some(odd), 1000), win(200, None, 0, 1000), "{odd}");
        }
    }

    #[test]
    fn ogni_risposta_parziale_si_ferma_a_un_mega() {
        let total = 5 * MAX_LEN;
        assert_eq!(
            window(Some("bytes=100-"), total),
            win(
                206,
                Some(&format!("bytes 100-{}/{total}", 100 + MAX_LEN - 1)),
                100,
                MAX_LEN
            )
        );
    }

    /// Un Bino con 3 s di mix, e i byte del suo `mix.ogg`.
    fn bino_with_mix(name: &str) -> (PathBuf, Vec<u8>) {
        let dir = temp_dir(name);
        let ogg = dir.join("mix.ogg");
        let mut writer = OggOpusWriter::new(File::create(&ogg).unwrap(), 48_000, 2, 64).unwrap();
        writer.write(&sine(48_000, 2, 3.0)).unwrap();
        writer.finish().unwrap();
        let path = dir.join("Riunione dell'unità.bino");
        let document = Document::new(
            "2026-10-04T10:00:00+02:00".into(),
            3000,
            Modalita::Mix,
            None,
            SpeechLanguage::It,
            true,
            &[],
        );
        bino::write(&path, &[(Ingresso::Mix, &ogg)], &document, None).unwrap();
        (path, std::fs::read(&ogg).unwrap())
    }

    #[test]
    fn le_richieste_range_danno_i_byte_del_mix_estratto() {
        let (path, extracted) = bino_with_mix("player-finestra");
        let total = extracted.len();
        let (whole, bytes) = read(&path, None).unwrap();
        assert_eq!(whole.status, 200);
        assert_eq!(bytes, extracted);
        for (range, from, to) in [
            ("bytes=0-99", 0, 100),
            ("bytes=1000-", 1000, total),
            ("bytes=-500", total - 500, total),
            ("bytes=200-299", 200, 300),
        ] {
            let (window, bytes) = read(&path, Some(range)).unwrap();
            assert_eq!(window.status, 206, "{range}");
            assert_eq!(bytes, extracted[from..to], "{range}");
        }
        let (beyond, bytes) = read(&path, Some(&format!("bytes={total}-"))).unwrap();
        assert_eq!(beyond.status, 416);
        assert!(bytes.is_empty());
    }

    #[test]
    fn la_forma_d_onda_si_calcola_una_volta_e_poi_si_legge_dal_bino() {
        let (path, _) = bino_with_mix("player-forma-onda");
        let activity = Activity::default();
        // Durante un'Attività su quel Bino si calcola ma non si salva.
        let guard = activity.begin(Some(path.clone()), || {}).unwrap();
        let computed = forma_onda(&path, 30, &activity).unwrap();
        assert_eq!(computed.len(), 30);
        assert_eq!(bino::forma_onda(&path), None);
        drop(guard);
        assert_eq!(forma_onda(&path, 30, &activity).unwrap(), computed);
        let saved = bino::forma_onda(&path).unwrap();
        // 3 s sono 150 picchi da 20 ms, meno dei valori salvati.
        assert_eq!(saved.len(), 150);
        assert_eq!(forma_onda::regroup(&saved, 30), computed);
        assert_eq!(forma_onda(&path, 3, &activity).unwrap().len(), 3);
    }

    #[test]
    fn il_protocollo_serve_solo_il_mix_di_un_bino() {
        // Come `convertFileSrc`, che usa `encodeURIComponent`.
        let request = |path: &str| {
            let encoded: String =
                percent_encoding::utf8_percent_encode(path, percent_encoding::NON_ALPHANUMERIC)
                    .collect();
            Request::builder()
                .uri(format!("http://bino.localhost/{encoded}"))
                .header(header::RANGE, "bytes=10-")
                .body(Vec::new())
                .unwrap()
        };
        let (path, extracted) = bino_with_mix("player-protocollo");
        let served = respond(&request(&path.display().to_string()));
        assert_eq!(served.status(), 206);
        assert_eq!(served.headers()[header::CONTENT_TYPE], "audio/ogg");
        assert_eq!(served.headers()[header::ACCEPT_RANGES], "bytes");
        assert_eq!(
            served.headers()[header::CONTENT_RANGE],
            format!("bytes 10-{}/{}", extracted.len() - 1, extracted.len()).as_str()
        );
        assert_eq!(served.body(), &extracted[10..]);
        assert_eq!(respond(&request(r"C:\Windows\win.ini")).status(), 403);
        assert_eq!(respond(&request(r"C:\non c'è\x.bino")).status(), 404);
    }
}
