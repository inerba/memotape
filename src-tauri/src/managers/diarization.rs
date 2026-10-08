//! Diarizzazione di un Tape già trascritto: nessuna ASR, un solo salvataggio a successo.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};
use transcribe_cpp::CancelToken;

use crate::engine::diarize::{self, Turn};
use crate::error::AppError;
use crate::managers::activity::Activity;
use crate::managers::models::Models;
use crate::managers::settings::{Settings, SettingsStore};
use crate::tape::{self, Document};
use crate::transcript::{
    Diarizzazione, DiarizzazioneIngresso, EsitoDiarizzazione, Ingresso, Phrase,
};

pub async fn diarize(app: AppHandle, activity: &Activity, source: PathBuf) -> Result<(), AppError> {
    let cancel = CancelToken::new();
    let _activity = activity.begin(Some(source.clone()), {
        let cancel = cancel.clone();
        move || cancel.cancel()
    })?;
    let settings = app.state::<SettingsStore>().get();
    tauri::async_runtime::spawn_blocking(move || {
        // Prenotazione e verifica del modello fuori dall'esecutore async, prima dell'analisi.
        check_cancel(&cancel)?;
        let mut diarizer = None;
        diarize_tape(&source, &settings, &cancel, |ingresso| {
            // Il core rifiuta prima un Tape illeggibile o senza testo, senza caricare modelli.
            if diarizer.is_none() {
                diarizer = Some(
                    app.state::<Models>()
                        .reserve_configured_diarizer(&app, &settings)?,
                );
            }
            diarizer
                .as_ref()
                .expect("modello prenotato")
                .diarize_ingresso(&source, ingresso, &cancel)
        })
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Il core non emette Frasi provvisorie: errore o Annulla in qualunque Ingresso conserva il Tape.
fn diarize_tape(
    source: &Path,
    settings: &Settings,
    cancel: &CancelToken,
    mut analyze: impl FnMut(Ingresso) -> Result<Vec<Turn>, AppError>,
) -> Result<(), AppError> {
    check_cancel(cancel)?;
    let mut document = tape::read(source)?;
    if document.frasi.is_empty() {
        return Err(AppError::Internal(
            "il Tape non ha testo da diarizzare".into(),
        ));
    }
    let separate = tape::has_ingressi(source)?;
    // L'azione esplicita non dipende dalla casella Riconosci i parlanti di Trascrivi.
    let ingressi = selected_ingressi(separate, settings.parlanti_microfono);
    let mut turns = Vec::new();
    for &ingresso in &ingressi {
        check_cancel(cancel)?;
        turns.push((ingresso, analyze(ingresso)?));
    }
    check_cancel(cancel)?;
    update_document(&mut document, settings, separate, &turns);
    tape::rewrite_cancellable(source, &document, cancel)
}

fn selected_ingressi(separate: bool, microphone: bool) -> Vec<Ingresso> {
    if !separate {
        vec![Ingresso::Mix]
    } else if microphone {
        vec![Ingresso::Microfono, Ingresso::Sistema]
    } else {
        vec![Ingresso::Sistema]
    }
}

fn check_cancel(cancel: &CancelToken) -> Result<(), AppError> {
    if cancel.is_cancelled() {
        Err(AppError::Cancelled)
    } else {
        Ok(())
    }
}

fn update_document(
    document: &mut Document,
    settings: &Settings,
    separate: bool,
    turns: &[(Ingresso, Vec<Turn>)],
) {
    let mut phrases: Vec<_> = document
        .frasi
        .iter()
        .map(|frase| Phrase {
            inizio_ms: frase.inizio_ms,
            fine_ms: frase.fine_ms,
            text: frase.testo.clone(),
            ingresso: frase.ingresso,
            parlante: frase.parlante,
            parlante_non_determinato: frase.parlante_non_determinato,
            parlante_provvisorio: frase.parlante_provvisorio,
            tempi: frase.tempi.clone(),
        })
        .collect();
    for (ingresso, found) in turns {
        if settings.diarizer == crate::managers::settings::Diarizer::Nemotron3 {
            diarize::assign_ingresso_unambiguous(&mut phrases, *ingresso, found);
        } else {
            diarize::assign_ingresso(&mut phrases, *ingresso, found);
        }
        for phrase in phrases.iter_mut().filter(|p| p.ingresso == *ingresso) {
            phrase.parlante_provvisorio = false;
        }
    }
    if separate && !settings.parlanti_microfono {
        for phrase in phrases
            .iter_mut()
            .filter(|p| p.ingresso == Ingresso::Microfono)
        {
            phrase.parlante = None;
            phrase.parlante_non_determinato = false;
            phrase.parlante_provvisorio = false;
        }
    }
    // Mantiene la provenienza di ogni parte: le identità vecchie non sono i numeri del modello.
    let mut next_id = document.frasi.iter().map(|f| f.id).max().unwrap_or(0);
    let mut projected = Vec::new();
    for (index, (original, phrase)) in document.frasi.iter().zip(&phrases).enumerate() {
        if original.parlante_corretto {
            let mut confirmed = original.clone();
            confirmed.parlante_provvisorio = false;
            projected.push((index, confirmed));
            continue;
        }
        let found = turns
            .iter()
            .find(|(ingresso, _)| *ingresso == original.ingresso);
        let parts = if settings.diarizer == crate::managers::settings::Diarizer::Nemotron3 {
            found
                .map(|(_, turns)| diarize::divide(phrase.clone(), turns))
                .unwrap_or_else(|| vec![phrase.clone()])
        } else {
            vec![phrase.clone()]
        };
        for (part, phrase) in parts.into_iter().enumerate() {
            let id = if part == 0 {
                original.id
            } else {
                next_id += 1;
                next_id
            };
            projected.push((
                index,
                tape::Frase {
                    id,
                    inizio_ms: phrase.inizio_ms,
                    fine_ms: phrase.fine_ms,
                    testo: phrase.text,
                    parlante: phrase.parlante,
                    parlante_non_determinato: phrase.parlante_non_determinato,
                    parlante_provvisorio: false,
                    tempi: phrase.tempi,
                    ..original.clone()
                },
            ));
        }
    }
    reconcile_identities(document, &mut projected, turns, &phrases);
    projected.sort_by_key(|(_, f)| f.inizio_ms);
    document.frasi = projected.into_iter().map(|(_, f)| f).collect();
    document.diarizzazione = Some(Diarizzazione {
        modello: settings.diarizer,
        esito: EsitoDiarizzazione::Completata,
        ingressi: turns
            .iter()
            .map(|(ingresso, _)| DiarizzazioneIngresso {
                ingresso: *ingresso,
                esito: EsitoDiarizzazione::Completata,
            })
            .collect(),
    });
}

/// Corrispondenze temporali uno-a-uno conservano l'identità personalizzata. In caso di ambiguità
/// il nome resta nel Tape, ma non viene dato a una voce nuova. I suoi numeri restano riservati.
fn reconcile_identities(
    document: &Document,
    projected: &mut [(usize, tape::Frase)],
    turns: &[(Ingresso, Vec<Turn>)],
    analyzed: &[Phrase],
) {
    use std::collections::{BTreeMap, BTreeSet};
    for (ingresso, _) in turns {
        let mut candidates: BTreeMap<u32, BTreeSet<Option<u32>>> = BTreeMap::new();
        let mut inverse: BTreeMap<Option<u32>, BTreeSet<u32>> = BTreeMap::new();
        let mut ambiguous = BTreeSet::new();
        let mut protected_unknown = BTreeSet::new();
        let mut reserved = BTreeSet::new();
        for key in document.parlanti.keys() {
            if let Some(number) = key
                .strip_prefix(&format!("{}:", ingresso.key()))
                .and_then(|s| s.parse::<u32>().ok())
            {
                reserved.insert(number);
            }
        }
        for (index, current) in projected.iter().filter(|(_, f)| f.ingresso == *ingresso) {
            let old = &document.frasi[*index];
            if old.parlante_corretto {
                // Anche il tratto protetto contribuisce alla corrispondenza delle voci,
                // senza cambiare la sua attribuzione. Ignorarlo nominerebbe una voce diversa.
                if let Some(number) = analyzed[*index]
                    .parlante
                    .filter(|_| !analyzed[*index].parlante_non_determinato)
                {
                    candidates.entry(number).or_default().insert(old.parlante);
                    inverse.entry(old.parlante).or_default().insert(number);
                } else {
                    protected_unknown.insert(old.parlante);
                }
                if let Some(number) = old.parlante {
                    reserved.insert(number);
                }
                continue;
            }
            if let Some(number) = current
                .parlante
                .filter(|_| !current.parlante_non_determinato)
            {
                // Una vecchia voce sconosciuta non è una prova dell'identità della nuova.
                if old.parlante_non_determinato {
                    ambiguous.insert(number);
                    candidates.entry(number).or_default();
                    continue;
                }
                candidates.entry(number).or_default().insert(old.parlante);
                inverse.entry(old.parlante).or_default().insert(number);
            }
        }
        let mut next = reserved
            .iter()
            .copied()
            .chain(candidates.keys().copied())
            .max()
            .unwrap_or(0);
        let mut mapping = BTreeMap::new();
        for (&number, old) in &candidates {
            let matched = old
                .iter()
                .next()
                .copied()
                .filter(|_| old.len() == 1)
                .filter(|_| !ambiguous.contains(&number))
                .filter(|identity| !protected_unknown.contains(identity))
                .filter(|identity| inverse.get(identity).is_some_and(|found| found.len() == 1))
                .filter(|identity| {
                    document
                        .parlanti
                        .contains_key(&ingresso.parlante_key(*identity))
                        || identity.is_some_and(|id| reserved.contains(&id))
                });
            let identity = match matched {
                Some(identity) => identity,
                None if reserved.contains(&number) => {
                    next += 1;
                    Some(next)
                }
                None => Some(number),
            };
            mapping.insert(number, identity);
        }
        for (_, current) in projected
            .iter_mut()
            .filter(|(_, f)| f.ingresso == *ingresso && !f.parlante_corretto)
        {
            if let Some(identity) = current.parlante.and_then(|number| mapping.get(&number)) {
                current.parlante = *identity;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::Read;

    use crate::audio_toolkit::ogg_opus::OggOpusWriter;
    use crate::audio_toolkit::ogg_opus::tests::{sine, temp_dir};
    use crate::engine::asr::AsrResult;
    use crate::managers::settings::{Diarizer, SpeechLanguage};
    use crate::tape::Modalita;

    fn fixture(name: &str, separate: bool) -> (PathBuf, Document) {
        let folder = temp_dir(name);
        let ogg = folder.join("audio.ogg");
        let mut writer = OggOpusWriter::new(File::create(&ogg).unwrap(), 16_000, 1, 32).unwrap();
        writer.write(&sine(16_000, 1, 3.0)).unwrap();
        writer.finish().unwrap();
        let timed = AsrResult::timed(
            "Ciao. È già qui!".into(),
            [(0, 900, "Ciao.".into()), (1000, 2000, "È già qui!".into())],
        );
        let ingressi = if separate {
            vec![Ingresso::Microfono, Ingresso::Sistema]
        } else {
            vec![Ingresso::Mix]
        };
        let mut phrases: Vec<_> = ingressi
            .iter()
            .map(|&ingresso| Phrase {
                inizio_ms: 0,
                fine_ms: 2100,
                text: timed.text.clone(),
                ingresso,
                parlante: Some(7),
                parlante_non_determinato: false,
                parlante_provvisorio: true,
                tempi: timed.tempi.clone(),
            })
            .collect();
        phrases.push(Phrase {
            inizio_ms: 2200,
            fine_ms: 2900,
            text: "Questa è la correzione manuale.".into(),
            tempi: Vec::new(),
            ..phrases.last().unwrap().clone()
        });
        let mut document = Document::new(
            "2026-10-03T17:05:00+02:00".into(),
            3000,
            if separate {
                Modalita::IngressiSeparati
            } else {
                Modalita::Mix
            },
            Some("nemotron".into()),
            SpeechLanguage::from("it"),
            false,
            &phrases,
        );
        document.origine = Some("lezione.mp4".into());
        document.parlanti = BTreeMap::from([
            ("mix:7".into(), "Mario".into()),
            ("microfono".into(), "Il mio nome".into()),
            ("microfono:7".into(), "Mario al microfono".into()),
            ("sistema:7".into(), "Lucia".into()),
        ]);
        let path = folder.join("Call.tape");
        let mut audio = vec![(Ingresso::Mix, ogg.as_path())];
        if separate {
            audio.extend([
                (Ingresso::Microfono, ogg.as_path()),
                (Ingresso::Sistema, ogg.as_path()),
            ]);
        }
        tape::write(&path, &audio, &document, Some(&[0.2, 0.4, 0.1])).unwrap();
        (path, document)
    }

    fn found() -> Vec<Turn> {
        vec![
            Turn {
                inizio_ms: 0,
                fine_ms: 1000,
                parlante: 11,
            },
            Turn {
                inizio_ms: 1000,
                fine_ms: 3000,
                parlante: 24,
            },
        ]
    }

    fn text(document: &Document, ingresso: Ingresso) -> String {
        document
            .frasi
            .iter()
            .filter(|f| f.ingresso == ingresso)
            .map(|f| f.testo.as_str())
            .collect()
    }

    fn entry(path: &Path, name: &str) -> Vec<u8> {
        let mut zip = zip::ZipArchive::new(File::open(path).unwrap()).unwrap();
        let mut bytes = Vec::new();
        zip.by_name(name).unwrap().read_to_end(&mut bytes).unwrap();
        bytes
    }

    #[test]
    fn ridiarizzazione_del_mix_conserva_testo_correzioni_audio_e_metadati() {
        let (path, before) = fixture("rediari-mix", false);
        let audio = entry(&path, "mix.ogg");
        let onda = entry(&path, "forma-onda.json");
        let settings = Settings {
            diarizer: Diarizer::Nemotron3,
            parlanti_file: false,
            ..Default::default()
        };
        let mut analyzed = Vec::new();
        diarize_tape(&path, &settings, &CancelToken::new(), |ingresso| {
            analyzed.push(ingresso);
            Ok(found())
        })
        .unwrap();
        assert_eq!(analyzed, [Ingresso::Mix]);
        let after = tape::read(&path).unwrap();
        assert_eq!(text(&before, Ingresso::Mix), text(&after, Ingresso::Mix));
        assert_eq!(after.frasi.len(), 3);
        assert_eq!(after.frasi[0].parlante, Some(1));
        assert_eq!(after.frasi[1].parlante, Some(2));
        assert_eq!(after.frasi[2].testo, "Questa è la correzione manuale.");
        assert!(after.frasi[2].tempi.is_empty());
        assert!(after.frasi.iter().all(|f| !f.parlante_provvisorio));
        assert_eq!(after.parlanti.get("mix:7"), before.parlanti.get("mix:7"));
        assert_eq!(audio, entry(&path, "mix.ogg"));
        assert_eq!(onda, entry(&path, "forma-onda.json"));
        // Normalizzando solo i campi che devono cambiare, il documento è identico.
        let mut unchanged = after.clone();
        unchanged.frasi = before.frasi.clone();
        unchanged.parlanti = before.parlanti.clone();
        unchanged.diarizzazione = before.diarizzazione.clone();
        assert_eq!(unchanged, before);
        let reopened = crate::managers::transcription::open_tape(&path).unwrap();
        assert_eq!(reopened.phrases.len(), after.frasi.len());
        assert_eq!(
            reopened.info.diarizzazione.unwrap().esito,
            EsitoDiarizzazione::Completata
        );
    }

    #[test]
    fn ridiarizzazione_separata_usa_solo_sistema_e_conserva_nome_del_microfono_singolo() {
        let (path, before) = fixture("rediari-sistema", true);
        let settings = Settings {
            diarizer: Diarizer::Sortformer,
            parlanti_microfono: false,
            parlanti_sistema: false,
            ..Default::default()
        };
        let mut analyzed = Vec::new();
        diarize_tape(&path, &settings, &CancelToken::new(), |ingresso| {
            analyzed.push(ingresso);
            Ok(found())
        })
        .unwrap();
        assert_eq!(analyzed, [Ingresso::Sistema]);
        let after = tape::read(&path).unwrap();
        assert_eq!(
            text(&after, Ingresso::Microfono),
            text(&before, Ingresso::Microfono)
        );
        assert_eq!(
            text(&after, Ingresso::Sistema),
            text(&before, Ingresso::Sistema)
        );
        assert!(
            after
                .frasi
                .iter()
                .filter(|f| f.ingresso == Ingresso::Microfono)
                .all(|f| f.parlante.is_none()
                    && !f.parlante_non_determinato
                    && !f.parlante_provvisorio)
        );
        assert_eq!(
            after.parlanti.get("microfono").map(String::as_str),
            Some("Il mio nome")
        );
        assert_eq!(after.parlanti, before.parlanti);
        assert!(tape::has_ingressi(&path).unwrap());
        assert_eq!(
            after.diarizzazione.unwrap().ingressi[0].ingresso,
            Ingresso::Sistema
        );
    }

    #[test]
    fn ridiarizzazione_separata_analizza_entrambi_senza_confondere_le_identita() {
        let (path, before) = fixture("rediari-entrambi", true);
        let audio: Vec<_> = ["mix.ogg", "microfono.ogg", "sistema.ogg"]
            .into_iter()
            .map(|name| (name, entry(&path, name)))
            .collect();
        let settings = Settings {
            diarizer: Diarizer::Nemotron3,
            parlanti_microfono: true,
            ..Default::default()
        };
        let mut analyzed = Vec::new();
        diarize_tape(&path, &settings, &CancelToken::new(), |ingresso| {
            analyzed.push(ingresso);
            Ok(found())
        })
        .unwrap();
        assert_eq!(analyzed, [Ingresso::Microfono, Ingresso::Sistema]);
        let after = tape::read(&path).unwrap();
        for ingresso in analyzed {
            assert_eq!(text(&after, ingresso), text(&before, ingresso));
            assert_eq!(
                after
                    .frasi
                    .iter()
                    .find(|f| f.ingresso == ingresso)
                    .unwrap()
                    .parlante,
                Some(1)
            );
        }
        assert_eq!(after.parlanti, before.parlanti);
        for (name, bytes) in audio {
            assert_eq!(entry(&path, name), bytes);
        }
        assert_eq!(after.diarizzazione.unwrap().ingressi.len(), 2);
    }

    #[test]
    fn ridiarizzazione_fallita_o_annullata_nel_secondo_ingresso_non_salva_nemmeno_il_primo() {
        for cancelled in [false, true] {
            let (path, _) = fixture(
                if cancelled {
                    "rediari-annullata"
                } else {
                    "rediari-fallita"
                },
                true,
            );
            let before = std::fs::read(&path).unwrap();
            let cancel = CancelToken::new();
            let settings = Settings {
                parlanti_microfono: true,
                ..Default::default()
            };
            let result = diarize_tape(&path, &settings, &cancel, |ingresso| {
                if ingresso == Ingresso::Sistema {
                    if cancelled {
                        cancel.cancel();
                    } else {
                        return Err(AppError::Internal("guasto del secondo Ingresso".into()));
                    }
                }
                Ok(found())
            });
            assert!(result.is_err());
            if cancelled {
                assert_eq!(result, Err(AppError::Cancelled));
            }
            assert_eq!(std::fs::read(&path).unwrap(), before);
        }
    }

    #[test]
    fn ridiarizzazione_rifiuta_testo_assente_e_conserva_il_tape_se_il_salvataggio_fallisce() {
        let (path, mut document) = fixture("rediari-vuota", false);
        document.frasi.clear();
        tape::rewrite(&path, &document).unwrap();
        let before = std::fs::read(&path).unwrap();
        assert!(
            diarize_tape(
                &path,
                &Settings::default(),
                &CancelToken::new(),
                |_| panic!("non deve analizzare senza testo")
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let (path, _) = fixture("rediari-scrittura-fallita", false);
        let before = std::fs::read(&path).unwrap();
        std::fs::create_dir(path.with_extension("tape.tmp")).unwrap();
        assert!(matches!(
            diarize_tape(&path, &Settings::default(), &CancelToken::new(), |_| Ok(
                found()
            )),
            Err(AppError::UnwritableFolder(_))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn ridiarizzazione_non_riunisce_frasi_gia_divise_e_non_inventa_tempi_dopo_correzione() {
        let (_, mut document) = fixture("rediari-confini", false);
        document.frasi[0].tempi.clear();
        let before = document.frasi.clone();
        update_document(
            &mut document,
            &Settings {
                diarizer: Diarizer::Nemotron3,
                ..Default::default()
            },
            false,
            &[(Ingresso::Mix, found())],
        );
        assert_eq!(document.frasi.len(), before.len());
        assert!(document.frasi[0].parlante_non_determinato);
        for (after, before) in document.frasi.iter().zip(before) {
            assert_eq!(after.testo, before.testo);
            assert_eq!(
                (after.inizio_ms, after.fine_ms),
                (before.inizio_ms, before.fine_ms)
            );
            assert!(after.tempi.is_empty());
        }
    }

    #[test]
    fn attribuzione_manuale_non_si_divide_ma_il_solo_testo_corretto_si_riattribuisce() {
        let (path, mut document) = fixture("rediari-manuale", false);
        document.frasi[0].parlante_corretto = true;
        document.frasi[1].testo_corretto = true;
        document.frasi[1].parlante = Some(8);
        tape::rewrite(&path, &document).unwrap();
        diarize_tape(
            &path,
            &Settings {
                diarizer: Diarizer::Nemotron3,
                ..Default::default()
            },
            &CancelToken::new(),
            |_| Ok(found()),
        )
        .unwrap();
        let after = tape::read(&path).unwrap();
        assert_eq!(after.frasi.len(), 2);
        assert_eq!(after.frasi[0].parlante, Some(7));
        assert_eq!(after.frasi[0].tempi, document.frasi[0].tempi);
        assert!(after.frasi[0].parlante_corretto);
        assert_eq!(after.frasi[1].parlante, Some(2));
        assert!(after.frasi[1].testo_corretto);
        assert_eq!(after.frasi[1].testo, document.frasi[1].testo);
        assert_eq!(after.frasi[1].id, document.frasi[1].id);
        assert!(
            crate::managers::transcription::open_tape(&path)
                .unwrap()
                .info
                .corretto_a_mano
        );
    }

    #[test]
    fn editor_turno_ridiarizzazione_conserva_correzioni_senza_inventare_allineamenti() {
        for (suffix, corretto, protected) in [
            ("unito", "Testo unito.", false),
            ("multilinea", "Testo.\n\nParagrafo.", true),
            ("vuoto", "", false),
            ("protetto-ambiguo", "Correzione protetta.", true),
        ] {
            let (path, mut original) = fixture(&format!("editor-rediari-{suffix}"), false);
            if suffix != "protetto-ambiguo" {
                original.frasi[0].fine_ms = 900;
            }
            if protected {
                original.frasi[0].parlante_corretto = true;
            }
            tape::rewrite(&path, &original).unwrap();
            let text = original
                .frasi
                .iter()
                .map(|f| f.testo.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            tape::edit_turno(&path, Ingresso::Mix, &[0, 1], &text, corretto).unwrap();
            let audio = entry(&path, "mix.ogg");
            let before = tape::read(&path).unwrap();
            diarize_tape(
                &path,
                &Settings {
                    diarizer: Diarizer::Nemotron3,
                    ..Default::default()
                },
                &CancelToken::new(),
                |_| Ok(found()),
            )
            .unwrap();
            let after = tape::read(&path).unwrap();
            assert_eq!(after.correzioni_testo, before.correzioni_testo);
            assert_eq!(after.frasi.len(), 2);
            assert!(
                after
                    .frasi
                    .iter()
                    .all(|f| f.tempi.is_empty() && f.testo_corretto)
            );
            if protected {
                assert_eq!(after.frasi[0].parlante, Some(7));
                assert!(after.frasi[0].parlante_corretto);
            }
            assert_ne!(after.frasi[1].parlante, original.frasi[1].parlante);
            let opened = crate::managers::transcription::open_tape(&path).unwrap();
            assert_eq!(opened.phrases[0].text, corretto);
            assert_eq!(opened.phrases[1].text, "");
            assert!(opened.phrases.iter().all(|f| f.parlante_non_determinato));
            assert_eq!(entry(&path, "mix.ogg"), audio);
            assert_eq!(tape::forma_onda(&path), Some(vec![0.2, 0.4, 0.1]));
        }
        // Un Turno non corretto pu\u00F2 ancora dividersi in parti con tempi ASR reali.
        let (path, mut original) = fixture("editor-rediari-nuovi-confini", false);
        original.frasi[1].parlante = Some(8);
        tape::rewrite(&path, &original).unwrap();
        tape::edit_turno(&path, Ingresso::Mix, &[1], &original.frasi[1].testo, "").unwrap();
        diarize_tape(
            &path,
            &Settings {
                diarizer: Diarizer::Nemotron3,
                ..Default::default()
            },
            &CancelToken::new(),
            |_| Ok(found()),
        )
        .unwrap();
        let opened = crate::managers::transcription::open_tape(&path).unwrap();
        assert_eq!(opened.phrases.len(), 3);
        assert!(
            opened
                .phrases
                .iter()
                .find(|f| f.phrase_id == 1)
                .unwrap()
                .text
                .is_empty()
        );
        assert!(opened.phrases.iter().any(|f| f.phrase_id > 1));
    }

    #[test]
    fn nomi_seguono_la_voce_temporale_e_non_la_coincidenza_del_numero() {
        let (_, mut document) = fixture("rediari-identita", false);
        // L'analisi nuova chiama 1 la vecchia voce 2 e 2 la vecchia voce 1.
        document.frasi[0].fine_ms = 900;
        document.frasi[0].tempi.clear();
        document.frasi[0].parlante = Some(2);
        document.frasi[1].parlante = Some(1);
        document.parlanti = BTreeMap::from([
            ("mix:1".into(), "Mario".into()),
            ("mix:2".into(), "Anna".into()),
        ]);
        update_document(
            &mut document,
            &Settings::default(),
            false,
            &[(Ingresso::Mix, found())],
        );
        assert_eq!(document.frasi[0].parlante, Some(2));
        assert_eq!(document.frasi[1].parlante, Some(1));
        assert_eq!(document.parlanti["mix:1"], "Mario");
        assert_eq!(document.parlanti["mix:2"], "Anna");

        // Se un vecchio Parlante si divide in due voci, il suo nome non va ad entrambe.
        document.frasi[1].parlante = Some(2);
        update_document(
            &mut document,
            &Settings::default(),
            false,
            &[(Ingresso::Mix, found())],
        );
        assert!(
            document
                .frasi
                .iter()
                .all(|f| ![Some(1), Some(2)].contains(&f.parlante))
        );
        assert_eq!(document.parlanti["mix:2"], "Anna");
    }

    #[test]
    #[ignore = "richiede MEMOTAPE_NEMOTRON3_MODEL, eseguire senza altri test nativi in parallelo"]
    fn ridiarizzazione_nativa_salva_e_riapre_il_tape_senza_asr() {
        use crate::audio_toolkit::decode::Decoder;
        use crate::engine::transcribe_cpp::OfflineDiarizer;

        let model = PathBuf::from(std::env::var("MEMOTAPE_NEMOTRON3_MODEL").unwrap());
        let folder = temp_dir("rediari-nativa");
        let wav = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parlato-due-voci.wav");
        let ogg = folder.join("audio.ogg");
        let mut writer = OggOpusWriter::new(File::create(&ogg).unwrap(), 16_000, 1, 32).unwrap();
        let mut decoder = Decoder::open(&wav).unwrap();
        let mut samples = 0;
        while let Some(block) = decoder.next_block().unwrap() {
            assert_eq!(block.rate, 16_000);
            let mono = block.mono();
            samples += mono.len();
            writer.write(&mono).unwrap();
        }
        let onda = writer.forma_onda();
        writer.finish().unwrap();
        let duration = u32::try_from(samples * 1000 / 16_000).unwrap();
        let cancel = CancelToken::new();
        let settings = Settings {
            diarizer: Diarizer::Nemotron3,
            parlanti_microfono: true,
            ..Default::default()
        };
        // Anche testo corretto senza tempi può essere analizzato: resta intero e ambiguo se
        // più voci attraversano la Frase. Nessun modello ASR viene caricato dal test.
        for separate in [false, true] {
            let ingressi = if separate {
                vec![Ingresso::Microfono, Ingresso::Sistema]
            } else {
                vec![Ingresso::Mix]
            };
            let phrases: Vec<_> = ingressi
                .iter()
                .map(|&ingresso| Phrase {
                    inizio_ms: 0,
                    fine_ms: duration,
                    text: "Il mio testo corretto: accenti, spazi e punteggiatura!".into(),
                    ingresso,
                    parlante: Some(1),
                    parlante_non_determinato: false,
                    parlante_provvisorio: false,
                    tempi: Vec::new(),
                })
                .collect();
            let mut document = Document::new(
                "2026-10-03T17:05:00+02:00".into(),
                duration,
                if separate {
                    Modalita::IngressiSeparati
                } else {
                    Modalita::Mix
                },
                Some("nemotron".into()),
                SpeechLanguage::from("it"),
                true,
                &phrases,
            );
            document
                .parlanti
                .insert(ingressi[0].parlante_key(Some(1)), "Mario".into());
            let path = folder.join(if separate {
                "Separato.tape"
            } else {
                "Mix.tape"
            });
            let mut audio = vec![(Ingresso::Mix, ogg.as_path())];
            if separate {
                audio.extend([
                    (Ingresso::Microfono, ogg.as_path()),
                    (Ingresso::Sistema, ogg.as_path()),
                ]);
            }
            tape::write(&path, &audio, &document, Some(&onda)).unwrap();
            let mix = entry(&path, "mix.ogg");
            diarize_tape(&path, &settings, &cancel, |ingresso| {
                let found = OfflineDiarizer::load_nemotron3(&model)?
                    .diarize_saved(Decoder::open_ingresso(&path, ingresso)?, &cancel)?;
                let speakers: std::collections::BTreeSet<_> =
                    found.iter().map(|turn| turn.parlante).collect();
                assert_eq!(speakers.len(), 2, "{ingresso:?}: {found:?}");
                Ok(found)
            })
            .unwrap();
            let after = tape::read(&path).unwrap();
            assert!(after.parlanti.is_empty());
            assert_eq!(after.frasi.len(), document.frasi.len());
            for ingresso in ingressi {
                assert_eq!(text(&after, ingresso), text(&document, ingresso));
            }
            assert_eq!(entry(&path, "mix.ogg"), mix);
            let opened = crate::managers::transcription::open_tape(&path).unwrap();
            assert!(opened.phrases.iter().all(|p| p.parlante_non_determinato));
            assert_eq!(
                opened.info.diarizzazione.unwrap().esito,
                EsitoDiarizzazione::Completata
            );
        }
    }
}
