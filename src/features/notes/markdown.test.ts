import { expect, test } from "bun:test";
import { EditorSelection, EditorState } from "@codemirror/state";
import {
  formatta,
  inserisciTempo,
  linkIn,
  type Marcatura,
  marcature,
  markdownDelleNote,
  spunta,
} from "@/features/notes/markdown";

/** Una Nota con il cursore in `cursore` (in fondo se manca). */
function nota(doc: string, cursore = doc.length, ancora = cursore) {
  return EditorState.create({
    doc,
    extensions: markdownDelleNote,
    selection: EditorSelection.single(ancora, cursore),
  });
}

/**
 * Come si legge la Nota: i segni nascosti spariscono, un punto d'elenco diventa «• » e una casella
 * «☐ » o «☑ », con lo spazio dopo il segno. Con `focus` la riga del cursore mostra il Markdown.
 */
function letta(state: EditorState, focus = true, durataMs = 0): string {
  const sostituzioni = marcature(state, focus, durataMs)
    .filter(
      (m) =>
        m.tipo === "segno" ||
        m.tipo === "punto" ||
        m.tipo === "casella" ||
        m.tipo === "tempo"
    )
    .sort((a, b) => a.da - b.da);
  let testo = "";
  let pos = 0;
  for (const m of sostituzioni) {
    testo += state.doc.sliceString(pos, m.da);
    if (m.tipo === "punto") {
      testo += "• ";
    } else if (m.tipo === "casella") {
      testo += m.fatta ? "☑ " : "☐ ";
    } else if (m.tipo === "tempo") {
      testo += `▶${m.ms / 1000}s`;
    }
    pos = m.a;
  }
  return testo + state.doc.sliceString(pos);
}

/** Gli stili della Nota, come `[tipo, testo]`. */
function stili(state: EditorState, focus = true) {
  return marcature(state, focus)
    .filter((m) =>
      ["titolo", "grassetto", "corsivo", "codice", "link", "numero"].includes(
        m.tipo
      )
    )
    .map((m: Marcatura) => [
      m.tipo === "titolo" ? `titolo${m.livello}` : m.tipo,
      state.doc.sliceString(m.da, m.a),
    ]);
}

test("fuori dalla riga del cursore i segni spariscono e il testo resta impaginato", () => {
  const state = nota(
    "## Decisioni\nIl rilascio è il **14 novembre**, _forse_ con `v2`.\n\n",
    0
  );
  // Il cursore sta sul titoletto: lì i segni restano.
  expect(letta(state)).toBe(
    "## Decisioni\nIl rilascio è il 14 novembre, forse con v2.\n\n"
  );
  expect(stili(state)).toEqual([
    ["titolo2", "## Decisioni"],
    ["grassetto", "**14 novembre**"],
    ["corsivo", "_forse_"],
    ["codice", "`v2`"],
  ]);
});

test("la riga del cursore mostra i segni solo con il focus nel campo", () => {
  const doc = "## Decisioni\nUno **due**";
  expect(letta(nota(doc))).toBe("Decisioni\nUno **due**");
  expect(letta(nota(doc), false)).toBe("Decisioni\nUno due");
  // Una selezione su più righe le mostra tutte.
  expect(letta(nota(doc, 0, doc.length))).toBe(doc);
  expect(marcature(nota(doc), true).filter((m) => m.tipo === "attiva")).toEqual(
    [{ a: doc.length, da: 13, tipo: "attiva" }]
  );
});

test("un link mostra solo il suo testo e porta l'indirizzo", () => {
  const state = nota(
    "Vedi [Studio Ferri](https://studioferri.it) e www.esempio.it\n",
    0
  );
  expect(letta(state, false)).toBe("Vedi Studio Ferri e www.esempio.it\n");
  expect(
    marcature(state, false)
      .filter((m) => m.tipo === "link")
      .map((m) => (m.tipo === "link" ? m.url : ""))
  ).toEqual(["https://studioferri.it", "https://www.esempio.it"]);
});

test("elenchi e caselle si leggono come punti e caselle da spuntare", () => {
  const state = nota(
    "- uno\n- [ ] Sara: preventivo\n- [x] Lucia: calendario\n1. primo\n",
    0
  );
  expect(letta(state, false)).toBe(
    "• uno\n☐ Sara: preventivo\n☑ Lucia: calendario\n1. primo\n"
  );
  // Sulla riga del cursore la casella torna Markdown.
  expect(letta(nota("- [ ] fatto", 0))).toBe("- [ ] fatto");
});

test("un clic sulla casella cambia solo [ ] o [x]", () => {
  const doc = "- [ ] Sara\n- [x] Lucia\n";
  /** La Nota dopo un clic sulla casella `n`. */
  const clic = (prima: EditorState, n: number) => {
    const caselle = marcature(prima, false).filter((m) => m.tipo === "casella");
    return prima.update(spunta(prima, caselle[n]?.da ?? -1) ?? {}).state;
  };
  const una = clic(nota(doc, 0), 0);
  expect(una.doc.toString()).toBe("- [x] Sara\n- [x] Lucia\n");
  const due = clic(una, 1);
  expect(due.doc.toString()).toBe("- [x] Sara\n- [ ] Lucia\n");
  // Fuori da una casella non succede niente.
  expect(spunta(due, 8)).toBeNull();
});

test("linkIn dà l'indirizzo del link sotto il puntatore", () => {
  const state = nota("[mail](mailto:a@b.it) e <https://x.it> ok", 0);
  expect(linkIn(state, 2)).toBe("mailto:a@b.it");
  expect(linkIn(state, 27)).toBe("https://x.it");
  expect(linkIn(state, 40)).toBeNull();
});

/** Applica `tipo` alla Nota `doc` con la selezione `[ancora, cursore]`: testo e selezione. */
function formattata(
  doc: string,
  tipo: Parameters<typeof formatta>[1],
  ancora: number,
  cursore = ancora
) {
  const state = nota(doc, cursore, ancora);
  const dopo = state.update(formatta(state, tipo)).state;
  const { from, to } = dopo.selection.main;
  const testo = dopo.doc.toString();
  return `${testo.slice(0, from)}|${testo.slice(from, to)}${from === to ? "" : "|"}${testo.slice(to)}`;
}

test("grassetto e corsivo avvolgono la selezione e la tolgono se c'è già", () => {
  expect(formattata("uno due", "grassetto", 4, 7)).toBe("uno **|due|**");
  expect(formattata("uno **due**", "grassetto", 6, 9)).toBe("uno |due|");
  expect(formattata("uno **due**", "grassetto", 4, 11)).toBe("uno |due|");
  expect(formattata("uno ", "grassetto", 4)).toBe("uno **|**");
  expect(formattata("uno due", "corsivo", 4, 7)).toBe("uno _|due|_");
  expect(formattata("uno _due_", "corsivo", 5, 8)).toBe("uno |due|");
});

test("link mette la selezione tra quadre e il cursore dove va l'indirizzo", () => {
  expect(formattata("vedi sito", "link", 5, 9)).toBe("vedi [sito](|)");
  expect(formattata("vedi ", "link", 5)).toBe("vedi [|]()");
});

test("elenco e da fare mettono o tolgono il segno a ogni riga", () => {
  expect(formattata("uno\ndue", "elenco", 0, 7)).toBe("- |uno\n- due|");
  expect(formattata("- uno\n- due", "elenco", 0, 11)).toBe("|uno\ndue|");
  expect(formattata("- uno", "daFare", 3)).toBe("- [ ] u|no");
  expect(formattata("  pane", "daFare", 6)).toBe("  - [ ] pane|");
  expect(formattata("- [x] uno", "daFare", 9)).toBe("uno|");
  expect(formattata("- [ ] uno", "elenco", 9)).toBe("- uno|");
});

test("fuori dalla riga del cursore un Tempo nella Nota diventa una pillola, entro la durata", () => {
  const doc = "Rilascio [12:40] e [1:02:05], `[0:30]`\nOra [0:30]";
  // Il Tape dura un'ora: [1:02:05] è oltre e resta testo, come il tempo nel codice.
  expect(letta(nota(doc), true, 3_600_000)).toBe(
    "Rilascio ▶760s e [1:02:05], [0:30]\nOra [0:30]"
  );
  expect(letta(nota(doc), false, 3_600_000)).toBe(
    "Rilascio ▶760s e [1:02:05], [0:30]\nOra ▶30s"
  );
  // Senza durata (un Tape ancora senza audio) nessuna pillola.
  expect(letta(nota(doc), false)).toBe(
    "Rilascio [12:40] e [1:02:05], [0:30]\nOra [0:30]"
  );
});

/** Le tabelle della Nota come le vede il lettore: allineamenti e righe di celle in testo. */
function tabelle(state: EditorState, focus = true, durataMs = 0) {
  return marcature(state, focus, durataMs).flatMap((m) =>
    m.tipo === "tabella"
      ? [
          {
            allineamenti: m.allineamenti,
            righe: m.righe.map((riga) =>
              riga.map((cella) =>
                cella.pezzi
                  .map((p) =>
                    p.ms === undefined
                      ? [...p.stili, p.url ?? ""]
                          .filter(Boolean)
                          .reduce((t, s) => `${s}(${t})`, p.testo)
                      : `▶${p.ms / 1000}s`
                  )
                  .join("")
              )
            ),
            testo: state.doc.sliceString(m.da, m.a),
          },
        ]
      : []
  );
}

test("fuori dal cursore una tabella si legge a celle, allineate come dice la riga dei trattini", () => {
  const doc =
    "Prima\n\n| Chi | Quando | Note |\n|:--|:-:|--:|\n| **Sara** | [12:40] | [sito](https://x.it) |\n|  | dopo\n\nDopo";
  const state = nota(doc, 0);
  expect(tabelle(state, true, 3_600_000)).toEqual([
    {
      allineamenti: ["sinistra", "centro", "destra"],
      righe: [
        ["Chi", "Quando", "Note"],
        ["grassetto(Sara)", "▶760s", "https://x.it(link(sito))"],
        // Una cella vuota resta vuota; una riga corta ha le celle che mancano vuote.
        ["", "dopo", ""],
      ],
      testo: doc.slice(7, -6),
    },
  ]);
  // Con il cursore nella tabella si vede il Markdown.
  expect(tabelle(nota(doc, 10))).toEqual([]);
  expect(tabelle(nota(doc, 10), false)).toHaveLength(1);
});

test("un clic su una cella porta il cursore all'inizio del suo testo", () => {
  const doc = "| A | B |\n|---|---|\n| uno | due |";
  const [tabella] = marcature(nota(doc, 0), false).filter(
    (m) => m.tipo === "tabella"
  );
  const celle = tabella?.tipo === "tabella" ? tabella.righe[1] : [];
  expect(celle?.map((c) => doc.slice(c.da, c.da + 3))).toEqual(["uno", "due"]);
});

/** Le marcature di riga di un tipo, come `[testo della riga, bordo del riquadro di codice]`. */
function righe(state: EditorState, tipo: string, focus = true) {
  return marcature(state, focus)
    .filter((m) => m.tipo === tipo)
    .map((m) => [state.doc.lineAt(m.da).text, "bordo" in m ? m.bordo : null]);
}

test("fuori dal cursore un blocco di codice perde le righe dei ``` e resta un riquadro", () => {
  const doc = "Prima\n```js\nuno\ndue\n```\nDopo";
  expect(righe(nota(doc, 0), "nascosta")).toEqual([
    ["```js", null],
    ["```", null],
  ]);
  expect(righe(nota(doc, 0), "rigaCodice")).toEqual([
    ["uno", "inizio"],
    ["due", "fine"],
  ]);
  // Con il cursore nel blocco i ``` tornano e fanno parte del riquadro.
  expect(righe(nota(doc, 14), "nascosta")).toEqual([]);
  expect(righe(nota(doc, 14), "rigaCodice")).toEqual([
    ["```js", "inizio"],
    ["uno", ""],
    ["due", ""],
    ["```", "fine"],
  ]);
  // Un blocco di una riga, ancora senza chiusura, ha il riquadro intero.
  expect(righe(nota("```\nuno", 0), "rigaCodice", false)).toEqual([
    ["uno", "solo"],
  ]);
});

test("una citazione perde i > fuori dalla riga del cursore e ha la barretta su ogni riga", () => {
  const doc = "> Il cliente\n> conferma\n\nDopo";
  expect(letta(nota(doc, 0))).toBe("> Il cliente\nconferma\n\nDopo");
  expect(righe(nota(doc, 0), "citazione")).toEqual([
    ["> Il cliente", null],
    ["> conferma", null],
  ]);
});

test("un separatore --- diventa una linea fuori dalla riga del cursore", () => {
  const doc = "Uno\n\n---\n\nDue";
  expect(letta(nota(doc, 0))).toBe("Uno\n\n\n\nDue");
  expect(righe(nota(doc, 0), "separatore")).toEqual([["---", null]]);
  expect(righe(nota(doc, 6), "separatore")).toEqual([]);
  expect(letta(nota(doc, 6))).toBe(doc);
});

test("fuori dalla riga del cursore una voce d'elenco rientra del suo livello, senza spazi scritti", () => {
  const doc = "- uno\n  - due\n    - [ ] tre\n1. primo\n10. decimo\n";
  const state = nota(doc, 0);
  expect(letta(state, false)).toBe(
    "• uno\n• due\n☐ tre\n1. primo\n10. decimo\n"
  );
  expect(
    marcature(state, false).flatMap((m) =>
      m.tipo === "voce"
        ? [[state.doc.lineAt(m.da).text, m.livello, m.numerata]]
        : []
    )
  ).toEqual([
    ["- uno", 0, false],
    ["  - due", 1, false],
    ["    - [ ] tre", 2, false],
    ["1. primo", 0, true],
    ["10. decimo", 0, true],
  ]);
  // Il numero resta com'è scritto, con lo spazio che lo segue.
  expect(stili(state, false).filter(([t]) => t === "numero")).toEqual([
    ["numero", "1. "],
    ["numero", "10. "],
  ]);
  // La riga del cursore mostra il Markdown, senza rientro.
  expect(righe(nota(doc, 8), "voce").map(([t]) => t)).not.toContain("  - due");
});

test("una riga vuota tra due blocchi è più bassa, anche con il cursore, ma non nel codice", () => {
  const doc = "Uno\n\n  \nDue\n```\n\n```";
  const vuote = (state: EditorState) =>
    marcature(state, true)
      .filter((m) => m.tipo === "vuota")
      .map((m) => state.doc.lineAt(m.da).number);
  expect(vuote(nota(doc, 0))).toEqual([2, 3]);
  expect(vuote(nota(doc, 4))).toEqual([2, 3]);
});

test("una tabella dentro una citazione resta Markdown, una dentro una voce rientra con lei", () => {
  expect(
    tabelle(nota("> | a | b |\n> |---|---|\n> | 1 | 2 |", 0), false)
  ).toEqual([]);
  const doc = "1. voce\n\n   | a | b |\n   |---|---|\n   | 1 | 2 |";
  const [t] = marcature(nota(doc, 0), false).filter(
    (m) => m.tipo === "tabella"
  );
  expect(t?.tipo === "tabella" ? t.voce : undefined).toEqual({
    casella: false,
    cifre: 1,
    livello: 0,
    numerata: true,
  });
  expect(tabelle(nota(doc, 0), false)[0]?.righe).toEqual([
    ["a", "b"],
    ["1", "2"],
  ]);
});

test("in una cella un | scritto \\| si legge senza la barra", () => {
  expect(
    tabelle(nota("| a |\n|---|\n| x \\| y |", 0), false)[0]?.righe
  ).toEqual([["a"], ["x | y"]]);
});

test("un blocco di codice vuoto tiene i ``` anche fuori dal cursore", () => {
  expect(righe(nota("```\n```", 0), "nascosta", false)).toEqual([]);
  expect(righe(nota("```\n```", 0), "rigaCodice", false)).toEqual([
    ["```", "inizio"],
    ["```", "fine"],
  ]);
});

test("una voce numerata sa quante cifre ha e se ha una casella; le righe che continuano la voce rientrano con lei", () => {
  const doc = "100. [ ] cento\n- uno\n  segue\n\n  altro paragrafo\n- due\n";
  const state = nota(doc, 0);
  const voci = marcature(state, false).flatMap((m) =>
    m.tipo === "voce" || m.tipo === "continua"
      ? [[m.tipo, state.doc.lineAt(m.da).text, m.cifre, m.casella]]
      : []
  );
  expect(voci).toEqual([
    ["voce", "100. [ ] cento", 3, true],
    ["voce", "- uno", 0, false],
    ["continua", "  segue", 0, false],
    ["continua", "  altro paragrafo", 0, false],
    ["voce", "- due", 0, false],
  ]);
  expect(letta(state, false)).toBe(
    "100. ☐ cento\n• uno\nsegue\n\naltro paragrafo\n• due\n"
  );
});

test("Ctrl+T scrive il tempo del player al cursore, al posto della selezione", () => {
  const scritto = (
    doc: string,
    ms: number,
    ancora: number,
    cursore = ancora
  ) => {
    const state = nota(doc, cursore, ancora);
    const dopo = state.update(inserisciTempo(state, ms)).state;
    const { head } = dopo.selection.main;
    const testo = dopo.doc.toString();
    return `${testo.slice(0, head)}|${testo.slice(head)}`;
  };
  expect(scritto("Decisione: ", 760_400, 11)).toBe("Decisione: [12:40] |");
  expect(scritto("Ora qui", 3_725_000, 4, 7)).toBe("Ora [1:02:05] |");
  expect(scritto("Ora\ndopo", 5000, 3)).toBe("Ora [0:05] |\ndopo");
});
