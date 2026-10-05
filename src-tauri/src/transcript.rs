//! Il documento di una Trascrizione (metadati e Frasi con i tempi) e il suo rendering in Markdown o
//! testo semplice: il `.md` accanto alla Sorgente e Copia testo. Senza Tauri.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::audio_toolkit::segmenter::Params;
use crate::managers::settings::{CopiaCome, Language, SpeechLanguage};

/// Oltre questo silenzio tra due Frasi si apre un paragrafo nuovo.
const PARAGRAPH_PAUSE_MS: u32 = 2000;

#[derive(Debug, Clone, PartialEq)]
pub struct Transcript {
    /// Il nome della Sorgente senza estensione.
    pub title: String,
    /// Data e ora locali della Trascrizione, `2026-10-03 17:05`.
    pub date: String,
    /// L'audio letto dalla pipeline; `None` finché non è finita (o se è stata annullata).
    pub durata_ms: Option<u32>,
    /// Il nome del modello.
    pub model: String,
    pub speech_language: SpeechLanguage,
    /// In ordine di inizio.
    pub phrases: Vec<Phrase>,
    /// I nomi dati ai Parlanti, per chiave `<ingresso>:<n>` (`Ingresso::parlante_key`).
    pub parlanti: BTreeMap<String, String>,
}

impl Transcript {
    /// Aggiunge una Frase in ordine di inizio, dopo quelle che iniziano insieme a lei: con gli
    /// Ingressi separati una Frase può arrivare dopo un'altra iniziata più tardi.
    pub fn insert(&mut self, phrase: Phrase) {
        let at = self
            .phrases
            .partition_point(|p| p.inizio_ms <= phrase.inizio_ms);
        self.phrases.insert(at, phrase);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Phrase {
    pub inizio_ms: u32,
    pub fine_ms: u32,
    pub text: String,
    pub ingresso: Ingresso,
    /// Il Parlante (da 1, per ordine di comparsa), con la Diarizzazione.
    pub parlante: Option<u32>,
}

/// Da dove viene una Frase: dal mix, o con gli Ingressi separati dal microfono o dall'audio di
/// sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Ingresso {
    Mix,
    Microfono,
    Sistema,
}

impl Ingresso {
    /// La chiave del Parlante `n` di questo Ingresso tra i nomi dei Parlanti: `sistema:2`.
    pub fn parlante_key(self, n: u32) -> String {
        format!("{}:{n}", self.key())
    }

    /// Il nome nel Bino e nelle chiavi: `mix`, `microfono`, `sistema`.
    pub fn key(self) -> &'static str {
        match self {
            Self::Mix => "mix",
            Self::Microfono => "microfono",
            Self::Sistema => "sistema",
        }
    }

    /// L'Ingresso di `key`.
    pub fn from_key(key: &str) -> Option<Self> {
        [Self::Mix, Self::Microfono, Self::Sistema]
            .into_iter()
            .find(|i| i.key() == key)
    }
}

impl Phrase {
    /// `Microfono · Parlante 2`, `Microfono`, `Parlante 2` o nessuna etichetta (il mix). Un Parlante
    /// rinominato ha il suo nome al posto di `Parlante 2`.
    fn label(&self, labels: &Labels, parlanti: &BTreeMap<String, String>) -> Option<String> {
        let ingresso = match self.ingresso {
            Ingresso::Mix => None,
            Ingresso::Microfono => Some(labels.get("/settings/recording/inputs/mic")),
            Ingresso::Sistema => Some(labels.get("/settings/recording/inputs/system")),
        };
        let parlante = self
            .parlante
            .map(|n| match parlanti.get(&self.ingresso.parlante_key(n)) {
                Some(nome) => nome.clone(),
                None => labels.parlante(n),
            });
        match (ingresso, parlante) {
            (Some(ingresso), Some(parlante)) => Some(format!("{ingresso} · {parlante}")),
            (Some(ingresso), None) => Some(ingresso.to_string()),
            (None, parlante) => parlante,
        }
    }
}

/// I testi dell'intestazione nella Lingua dell'interfaccia, presi dalle traduzioni del frontend.
pub struct Labels(serde_json::Value);

impl Labels {
    pub fn of(language: Language) -> Self {
        let json = match language {
            Language::It => include_str!("../../src/locales/it.json"),
            Language::En => include_str!("../../src/locales/en.json"),
            Language::Fr => include_str!("../../src/locales/fr.json"),
            Language::Es => include_str!("../../src/locales/es.json"),
            Language::De => include_str!("../../src/locales/de.json"),
            Language::Pl => include_str!("../../src/locales/pl.json"),
        };
        Self(serde_json::from_str(json).expect("le traduzioni sono JSON valido"))
    }

    /// Il testo in `pointer` (`/transcript/date`); vuoto se manca, ma il test lo esclude.
    /// L'etichetta del Parlante `n` non rinominato: `Parlante 2`.
    pub fn parlante(&self, n: u32) -> String {
        self.get("/transcript/parlante")
            .replace("{{n}}", &n.to_string())
    }

    fn get(&self, pointer: &str) -> &str {
        self.0
            .pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
    }

    fn speech_language(&self, language: SpeechLanguage) -> &str {
        match language.code() {
            None => self.get("/speechLanguage/auto"),
            Some(code) => self.get(&format!("/speechLanguage/languages/{code}")),
        }
    }
}

/// Il documento: titolo, intestazione con data, durata, modello e Lingua del parlato, poi i
/// paragrafi. In Markdown il titolo è un `#` e le etichette sono in grassetto.
pub fn render(transcript: &Transcript, labels: &Labels, format: CopiaCome) -> String {
    let markdown = format == CopiaCome::Markdown;
    let mut fields = vec![(labels.get("/transcript/date"), transcript.date.clone())];
    if let Some(durata_ms) = transcript.durata_ms {
        fields.push((labels.get("/transcript/duration"), duration(durata_ms)));
    }
    fields.push((labels.get("/transcript/model"), transcript.model.clone()));
    fields.push((
        labels.get("/speechLanguage/label"),
        labels
            .speech_language(transcript.speech_language)
            .to_string(),
    ));
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{}{}\n",
        if markdown { "# " } else { "" },
        transcript.title
    );
    for (label, value) in fields {
        let _ = if markdown {
            writeln!(out, "- **{label}:** {value}")
        } else {
            writeln!(out, "{label}: {value}")
        };
    }
    for (label, text) in paragraphs(transcript, labels) {
        let _ = match label {
            Some(label) if markdown => writeln!(out, "\n**{label}:** {text}"),
            Some(label) => writeln!(out, "\n{label}: {text}"),
            None if markdown => writeln!(out, "\n{}", escape_block_start(&text)),
            None => writeln!(out, "\n{text}"),
        };
    }
    out
}

/// Un paragrafo che in Markdown comincerebbe un elenco, un titolo o una citazione (`- Sì.`,
/// `1. Primo`, `# `, `> `) resta testo.
fn escape_block_start(text: &str) -> String {
    let digits = text.len() - text.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    if digits > 0 && text[digits..].starts_with(['.', ')']) {
        format!("{}\\{}", &text[..digits], &text[digits..])
    } else if text.starts_with(['#', '>', '-', '+', '*']) {
        format!("\\{text}")
    } else {
        text.to_string()
    }
}

/// Le Frasi unite in paragrafi. Con le etichette, un paragrafo per turno: le Frasi consecutive
/// della stessa voce si uniscono. Senza, un paragrafo nuovo dopo una pausa lunga.
fn paragraphs(transcript: &Transcript, labels: &Labels) -> Vec<(Option<String>, String)> {
    let label_of = |phrase: &Phrase| phrase.label(labels, &transcript.parlanti);
    let turns = transcript.phrases.iter().any(|p| label_of(p).is_some());
    let mut out: Vec<(Option<String>, String)> = Vec::new();
    let mut previous: Option<&Phrase> = None;
    for phrase in &transcript.phrases {
        let label = label_of(phrase);
        let same = previous.is_some_and(|previous| {
            if turns {
                label_of(previous) == label
            } else {
                silence_ms(previous, phrase) <= PARAGRAPH_PAUSE_MS
            }
        });
        match out.last_mut() {
            Some((_, text)) if same => {
                text.push(' ');
                text.push_str(&phrase.text);
            }
            _ => out.push((label, phrase.text.clone())),
        }
        previous = Some(phrase);
    }
    out
}

/// Il silenzio reale tra due Frasi. I loro tempi comprendono l'hangover in fondo alla prima e il
/// prefill in testa alla seconda, quindi tra Frasi separate da un silenzio il buco misurato è più
/// corto di quei due. Due Frasi contigue (spezzate per durata massima) restano a 0.
// ponytail: dopo una Frase chiusa senza hangover (taglio a 18 s seguito da silenzio, Pausa) la stima
// eccede di 0,7 s; se serve, la pipeline dovrà dire come si è chiusa la Frase.
fn silence_ms(previous: &Phrase, next: &Phrase) -> u32 {
    let params = Params::default();
    match next.inizio_ms.saturating_sub(previous.fine_ms) {
        0 => 0,
        gap => gap + params.hangover_ms + params.prefill_ms,
    }
}

/// `1:02:03` oltre l'ora, altrimenti `2:03`.
fn duration(ms: u32) -> String {
    let seconds = ms / 1000;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phrase(inizio_ms: u32, fine_ms: u32, text: &str) -> Phrase {
        Phrase {
            inizio_ms,
            fine_ms,
            text: text.into(),
            ingresso: Ingresso::Mix,
            parlante: None,
        }
    }

    fn voice(ingresso: Ingresso, parlante: Option<u32>, text: &str) -> Phrase {
        Phrase {
            ingresso,
            parlante,
            ..phrase(0, 0, text)
        }
    }

    fn transcript(phrases: Vec<Phrase>) -> Transcript {
        Transcript {
            title: "Riunione".into(),
            date: "2026-10-03 17:05".into(),
            durata_ms: Some(3_723_400),
            model: "Nemotron".into(),
            speech_language: SpeechLanguage::It,
            phrases,
            parlanti: BTreeMap::new(),
        }
    }

    fn markdown(transcript: &Transcript) -> String {
        render(transcript, &Labels::of(Language::It), CopiaCome::Markdown)
    }

    #[test]
    fn il_markdown_ha_titolo_intestazione_e_paragrafi() {
        let text = markdown(&transcript(vec![
            phrase(0, 3000, "Buongiorno."),
            phrase(3000, 5000, "Iniziamo."),
        ]));
        assert_eq!(
            text,
            "# Riunione\n\n\
             - **Data:** 2026-10-03 17:05\n\
             - **Durata:** 1:02:03\n\
             - **Modello:** Nemotron\n\
             - **Lingua del parlato:** Italiano\n\
             \nBuongiorno. Iniziamo.\n"
        );
    }

    #[test]
    fn l_intestazione_segue_la_lingua_dell_interfaccia_e_omette_la_durata_ignota() {
        let mut document = transcript(vec![phrase(0, 900, "Hello.")]);
        document.durata_ms = None;
        document.speech_language = SpeechLanguage::Auto;
        let text = render(&document, &Labels::of(Language::En), CopiaCome::Markdown);
        assert!(!text.contains("1:02:03"), "{text}");
        assert!(text.contains("- **Model:** Nemotron\n"), "{text}");
        assert!(text.contains(":** Automatic\n"), "{text}");
    }

    #[test]
    fn un_paragrafo_nuovo_dopo_oltre_2_s_di_silenzio() {
        // I tempi comprendono hangover (700 ms) e prefill (300 ms): un buco misurato di 1 s è un
        // silenzio di 2 s.
        let text = markdown(&transcript(vec![
            phrase(0, 2000, "Uno."),
            phrase(3000, 4000, "Due."),
            phrase(5010, 6000, "Tre."),
            // Spezzata per durata massima: contigua, stesso paragrafo.
            phrase(6000, 7000, "Quattro."),
        ]));
        assert!(text.ends_with("\nUno. Due.\n\nTre. Quattro.\n"), "{text}");
    }

    #[test]
    fn con_le_etichette_un_paragrafo_per_turno_e_le_frasi_della_stessa_voce_unite() {
        let text = markdown(&transcript(vec![
            voice(Ingresso::Microfono, None, "Mi senti?"),
            voice(Ingresso::Microfono, None, "Pronto?"),
            voice(Ingresso::Sistema, Some(1), "Sì."),
            voice(Ingresso::Sistema, Some(3), "Anch'io."),
            voice(Ingresso::Sistema, Some(3), "Ciao."),
            voice(Ingresso::Mix, Some(2), "Eccomi."),
        ]));
        assert!(
            text.ends_with(
                "\n**Microfono:** Mi senti? Pronto?\n\
                 \n**Audio di sistema · Parlante 1:** Sì.\n\
                 \n**Audio di sistema · Parlante 3:** Anch'io. Ciao.\n\
                 \n**Parlante 2:** Eccomi.\n"
            ),
            "{text}"
        );
    }

    #[test]
    fn i_nomi_dei_parlanti_sostituiscono_parlante_n_solo_nel_loro_ingresso() {
        let mut document = transcript(vec![
            voice(Ingresso::Sistema, Some(1), "Ciao."),
            voice(Ingresso::Microfono, Some(1), "Salve."),
            voice(Ingresso::Sistema, Some(2), "Eccomi."),
        ]);
        document
            .parlanti
            .insert(Ingresso::Sistema.parlante_key(1), "Mario".into());
        let text = markdown(&document);
        assert!(
            text.ends_with(
                "\n**Audio di sistema · Mario:** Ciao.\n\
                 \n**Microfono · Parlante 1:** Salve.\n\
                 \n**Audio di sistema · Parlante 2:** Eccomi.\n"
            ),
            "{text}"
        );
        assert_eq!(Ingresso::Mix.parlante_key(3), "mix:3");
    }

    #[test]
    fn il_testo_semplice_non_ha_sintassi_markdown() {
        let document = transcript(vec![
            voice(Ingresso::Mix, Some(1), "Uno."),
            voice(Ingresso::Mix, Some(2), "Due."),
        ]);
        let text = render(&document, &Labels::of(Language::It), CopiaCome::Testo);
        assert_eq!(
            text,
            "Riunione\n\n\
             Data: 2026-10-03 17:05\n\
             Durata: 1:02:03\n\
             Modello: Nemotron\n\
             Lingua del parlato: Italiano\n\
             \nParlante 1: Uno.\n\
             \nParlante 2: Due.\n"
        );
    }

    #[test]
    fn un_paragrafo_che_sembra_un_elenco_o_un_titolo_resta_testo_in_markdown() {
        let paragraph = |text: &str| {
            let text = markdown(&transcript(vec![phrase(0, 1000, text)]));
            text.rsplit("\n\n").next().unwrap().to_string()
        };
        assert_eq!(paragraph("- Sì."), "\\- Sì.\n");
        assert_eq!(paragraph("1. Primo punto."), "1\\. Primo punto.\n");
        assert_eq!(paragraph("# uno"), "\\# uno\n");
        assert_eq!(paragraph("2026 è l'anno."), "2026 è l'anno.\n");
        let plain = render(
            &transcript(vec![phrase(0, 1000, "- Sì.")]),
            &Labels::of(Language::It),
            CopiaCome::Testo,
        );
        assert!(plain.ends_with("\n- Sì.\n"), "{plain}");
    }

    #[test]
    fn le_frasi_si_inseriscono_in_ordine_di_inizio() {
        let mut document = transcript(Vec::new());
        for (inizio_ms, ingresso, text) in [
            (0, Ingresso::Microfono, "Mi senti?"),
            // Il microfono è più avanti: la risposta dell'audio di sistema arriva dopo, ma prima.
            (4000, Ingresso::Microfono, "Allora?"),
            (2000, Ingresso::Sistema, "Sì."),
            (4000, Ingresso::Sistema, "Ti sento."),
        ] {
            document.insert(Phrase {
                inizio_ms,
                ..voice(ingresso, None, text)
            });
        }
        let texts: Vec<&str> = document.phrases.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(texts, ["Mi senti?", "Sì.", "Allora?", "Ti sento."]);
        let text = markdown(&document);
        assert!(
            text.ends_with(
                "\n**Microfono:** Mi senti?\n\
                 \n**Audio di sistema:** Sì.\n\
                 \n**Microfono:** Allora?\n\
                 \n**Audio di sistema:** Ti sento.\n"
            ),
            "{text}"
        );
    }

    #[test]
    fn la_durata_sotto_l_ora_non_ha_le_ore() {
        assert_eq!(duration(0), "0:00");
        assert_eq!(duration(65_999), "1:05");
        assert_eq!(duration(3_600_000), "1:00:00");
    }

    #[test]
    fn ogni_lingua_ha_i_testi_dell_intestazione() {
        let languages = [
            Language::It,
            Language::En,
            Language::Fr,
            Language::Es,
            Language::De,
            Language::Pl,
        ];
        let speech = [
            SpeechLanguage::Auto,
            SpeechLanguage::It,
            SpeechLanguage::En,
            SpeechLanguage::Fr,
            SpeechLanguage::Es,
            SpeechLanguage::De,
            SpeechLanguage::Pl,
        ];
        for language in languages {
            let labels = Labels::of(language);
            for pointer in [
                "/transcript/date",
                "/transcript/duration",
                "/transcript/model",
                "/transcript/parlante",
                "/speechLanguage/label",
                "/settings/recording/inputs/mic",
                "/settings/recording/inputs/system",
            ] {
                assert!(!labels.get(pointer).is_empty(), "{language:?} {pointer}");
            }
            for s in speech {
                assert!(!labels.speech_language(s).is_empty(), "{language:?} {s:?}");
            }
        }
    }
}
