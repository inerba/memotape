import { syntaxTree } from "@codemirror/language";
import {
  type EditorState,
  type Extension,
  Prec,
  type Range,
  StateEffect,
  StateField,
} from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  keymap,
  WidgetType,
} from "@codemirror/view";
import {
  type Allineamento,
  type Formato,
  formatta,
  linkIn,
  type Marcatura,
  marcature,
  markdownDelleNote,
  type Pezzo,
  spunta,
} from "./markdown";

/**
 * L'editor ibrido delle Note, come il Live Preview di Obsidian (ADR-0032): le marcature di
 * `markdown.ts` diventano decorazioni di CodeMirror. Il testo resta quello scritto.
 */

/** Applica `formato` alla selezione; `false` in sola lettura. */
export function applica(view: EditorView, formato: Formato): boolean {
  if (view.state.readOnly) {
    return false;
  }
  view.dispatch({
    ...formatta(view.state, formato),
    scrollIntoView: true,
    userEvent: "input.format",
  });
  return true;
}

class Punto extends WidgetType {
  override eq() {
    return true;
  }
  override toDOM() {
    const punto = document.createElement("span");
    punto.className = "cm-punto";
    punto.textContent = "•";
    return punto;
  }
}

class Casella extends WidgetType {
  readonly fatta: boolean;
  constructor(fatta: boolean) {
    super();
    this.fatta = fatta;
  }
  override eq(altra: Casella) {
    return altra.fatta === this.fatta;
  }
  override toDOM(view: EditorView) {
    const casella = document.createElement("input");
    casella.type = "checkbox";
    casella.className = "cm-casella";
    casella.checked = this.fatta;
    casella.tabIndex = -1;
    // Il clic cambia il testo, non la casella: la vista si ridisegna con il testo nuovo.
    casella.addEventListener("mousedown", (e) => {
      e.preventDefault();
      if (view.state.readOnly) {
        return;
      }
      const spec = spunta(view.state, view.posAtDOM(casella));
      if (spec) {
        view.dispatch({ ...spec, userEvent: "input.check" });
      }
    });
    casella.addEventListener("click", (e) => e.preventDefault());
    return casella;
  }
  override ignoreEvent() {
    return true;
  }
}

/** Quello che serve alle pillole dei Tempi nella Nota; letto quando si ridisegnano. */
export interface Tempi {
  /** La durata del Tape: un tempo da lì in poi resta testo. */
  durataMs: () => number;
  /** «Ascolta da 12:40» per il tempo scritto `testo`. */
  etichetta: (testo: string) => string;
  /** Porta il player a `ms`. */
  salta: (ms: number) => void;
}

const SVG = "http://www.w3.org/2000/svg";

/** La pillola del Tempo nella Nota `testo`: un clic porta lì il player, anche in sola lettura. */
function pillola(testo: string, ms: number, tempi: Tempi): HTMLElement {
  const bottone = document.createElement("button");
  bottone.type = "button";
  bottone.className = "cm-tempo";
  bottone.tabIndex = -1;
  bottone.title = tempi.etichetta(testo);
  bottone.setAttribute("aria-label", bottone.title);
  const icona = document.createElementNS(SVG, "svg");
  icona.setAttribute("viewBox", "0 0 24 24");
  icona.setAttribute("aria-hidden", "true");
  const triangolo = document.createElementNS(SVG, "path");
  triangolo.setAttribute("d", "M7 4.5v15l12-7.5z");
  icona.append(triangolo);
  bottone.append(icona, testo);
  // Il cursore resta dov'è: la pillola porta il player, non sposta la scrittura.
  bottone.addEventListener("mousedown", (e) => e.preventDefault());
  bottone.addEventListener("click", (e) => {
    e.preventDefault();
    tempi.salta(ms);
  });
  return bottone;
}

class Pillola extends WidgetType {
  readonly ms: number;
  readonly testo: string;
  readonly tempi: Tempi;
  constructor(testo: string, ms: number, tempi: Tempi) {
    super();
    this.testo = testo;
    this.ms = ms;
    this.tempi = tempi;
  }
  override eq(altra: Pillola) {
    return altra.testo === this.testo && altra.ms === this.ms;
  }
  override toDOM() {
    return pillola(this.testo, this.ms, this.tempi);
  }
  override ignoreEvent() {
    return true;
  }
}

/** Un pezzo di una cella: testo nei suoi stili, con l'indirizzo se è un link, o una pillola. */
function pezzo(p: Pezzo, tempi: Tempi): Node {
  if (p.ms !== undefined) {
    return pillola(p.testo, p.ms, tempi);
  }
  let nodo: Node = document.createTextNode(p.testo);
  for (const stile of p.stili) {
    const span = document.createElement("span");
    span.className = `cm-${stile}`;
    span.append(nodo);
    nodo = span;
  }
  if (p.url && nodo instanceof HTMLElement) {
    nodo.dataset.url = p.url;
  }
  return nodo;
}

const ALLINEA: Record<NonNullable<Allineamento>, string> = {
  centro: "center",
  destra: "right",
  sinistra: "left",
};

/**
 * Una tabella fuori dal cursore. Un clic su un link lo apre, su una pillola porta il player, su una
 * cella mette lì il cursore e la tabella torna Markdown.
 */
class Tabella extends WidgetType {
  readonly tabella: Marcatura & { tipo: "tabella" };
  readonly apriLink: (url: string) => void;
  readonly tempi: Tempi;
  readonly chiave: string;
  constructor(
    tabella: Marcatura & { tipo: "tabella" },
    apriLink: (url: string) => void,
    tempi: Tempi
  ) {
    super();
    this.tabella = tabella;
    this.apriLink = apriLink;
    this.tempi = tempi;
    this.chiave = JSON.stringify([tabella.allineamenti, tabella.righe]);
  }
  override eq(altra: Tabella) {
    return altra.chiave === this.chiave;
  }
  override toDOM(view: EditorView) {
    const { allineamenti, righe } = this.tabella;
    const box = document.createElement("div");
    box.className = "cm-tabella";
    const table = document.createElement("table");
    const testa = document.createElement("thead");
    const corpo = document.createElement("tbody");
    righe.forEach((riga, r) => {
      const tr = document.createElement("tr");
      riga.forEach((cella, c) => {
        const td = document.createElement(r === 0 ? "th" : "td");
        const allineamento = allineamenti[c];
        if (allineamento) {
          td.style.textAlign = ALLINEA[allineamento];
        }
        td.dataset.da = String(cella.da);
        td.append(...cella.pezzi.map((p) => pezzo(p, this.tempi)));
        tr.append(td);
      });
      (r === 0 ? testa : corpo).append(tr);
    });
    table.append(testa, corpo);
    box.append(table);
    box.addEventListener("mousedown", (e) => {
      if (!(e.target instanceof Element) || e.target.closest(".cm-tempo")) {
        return;
      }
      const url = e.target.closest<HTMLElement>("[data-url]")?.dataset.url;
      const cella = e.target.closest<HTMLElement>("[data-da]");
      if (url) {
        e.preventDefault();
        this.apriLink(url);
      } else if (cella && !view.state.readOnly) {
        e.preventDefault();
        view.dispatch({ selection: { anchor: Number(cella.dataset.da) } });
        view.focus();
      }
    });
    return box;
  }
  override ignoreEvent() {
    return true;
  }
}

const NASCOSTO = Decoration.replace({});
const RIGA_NASCOSTA = Decoration.replace({ block: true });
const PUNTO = Decoration.replace({ widget: new Punto() });
const STILE = {
  attiva: Decoration.line({ class: "cm-attiva" }),
  citazione: Decoration.line({ class: "cm-citazione" }),
  codice: Decoration.mark({ class: "cm-codice" }),
  corsivo: Decoration.mark({ class: "cm-corsivo" }),
  grassetto: Decoration.mark({ class: "cm-grassetto" }),
  link: Decoration.mark({ class: "cm-link" }),
  numero: Decoration.mark({ class: "cm-numero" }),
  separatore: Decoration.line({ class: "cm-separatore" }),
  vuota: Decoration.line({ class: "cm-vuota" }),
};

function decorazione(
  state: EditorState,
  m: Marcatura,
  apriLink: (url: string) => void,
  tempi: Tempi
): Range<Decoration> | null {
  switch (m.tipo) {
    case "tabella":
      return Decoration.replace({
        block: true,
        widget: new Tabella(m, apriLink, tempi),
      }).range(m.da, m.a);
    case "nascosta":
      return RIGA_NASCOSTA.range(m.da, m.a);
    case "rigaCodice":
      return Decoration.line({
        class: m.bordo
          ? `cm-riga-codice cm-riga-codice-${m.bordo}`
          : "cm-riga-codice",
      }).range(m.da);
    case "voce":
      return Decoration.line({
        attributes: { style: `--livello: ${m.livello}` },
        class: m.numerata ? "cm-voce cm-voce-numerata" : "cm-voce",
      }).range(m.da);
    case "tempo":
      return Decoration.replace({
        widget: new Pillola(
          state.doc.sliceString(m.da + 1, m.a - 1),
          m.ms,
          tempi
        ),
      }).range(m.da, m.a);
    case "segno":
    case "punto":
      // Un segno non va mai a capo: una sostituzione non può attraversare le righe.
      if (m.da === m.a || state.doc.sliceString(m.da, m.a).includes("\n")) {
        return null;
      }
      return (m.tipo === "punto" ? PUNTO : NASCOSTO).range(m.da, m.a);
    case "casella":
      return Decoration.replace({ widget: new Casella(m.fatta) }).range(
        m.da,
        m.a
      );
    case "titolo":
      return Decoration.line({
        class: `cm-titolo cm-titolo-${m.livello}`,
      }).range(m.da);
    case "attiva":
    case "citazione":
    case "separatore":
    case "vuota":
      return STILE[m.tipo].range(m.da);
    default:
      return m.da === m.a ? null : STILE[m.tipo].range(m.da, m.a);
  }
}

/** Il focus nel campo, che lo stato non ha: la riga del cursore lo chiede. */
const fuoco = StateEffect.define<boolean>();

/**
 * Le decorazioni delle marcature, in un campo dello stato: tabelle e righe nascoste dei blocchi
 * di codice sostituiscono righe intere, e CodeMirror le accetta solo da qui, non da un plugin.
 */
function anteprima(apriLink: (url: string) => void, tempi: Tempi): Extension {
  const calcola = (state: EditorState, focus: boolean): DecorationSet =>
    Decoration.set(
      marcature(state, focus, tempi.durataMs())
        .map((m) => decorazione(state, m, apriLink, tempi))
        .filter((r) => r !== null),
      true
    );
  const campo = StateField.define<{
    decorazioni: DecorationSet;
    focus: boolean;
  }>({
    create: (state) => ({ decorazioni: calcola(state, false), focus: false }),
    provide: (f) => EditorView.decorations.from(f, (v) => v.decorazioni),
    update: (valore, tr) => {
      let { focus } = valore;
      let cambiato = false;
      for (const e of tr.effects) {
        if (e.is(fuoco)) {
          focus = e.value;
          cambiato = true;
        }
      }
      if (
        cambiato ||
        tr.docChanged ||
        tr.selection ||
        syntaxTree(tr.startState) !== syntaxTree(tr.state)
      ) {
        return { decorazioni: calcola(tr.state, focus), focus };
      }
      return valore;
    },
  });
  return [
    campo,
    EditorView.focusChangeEffect.of((_, focusing) => fuoco.of(focusing)),
  ];
}

/** Il font dei titoli di DESIGN.md: `--font-display` di Tailwind non esiste come variabile CSS. */
const DISPLAY = "Commissioner Variable, ui-sans-serif, system-ui, sans-serif";

/** Il tema dell'editor ibrido, solo con i token di DESIGN.md. */
const TEMA = EditorView.theme({
  ".cm-attiva": {
    backgroundColor: "color-mix(in oklch, var(--accent) 45%, transparent)",
    borderRadius: "6px",
  },
  ".cm-attiva .cm-link": { cursor: "text" },
  // Casella, punto e numero sono larghi quanto il rientro sospeso della voce.
  ".cm-casella": {
    accentColor: "var(--foreground)",
    cursor: "pointer",
    margin: "0 0.25em 0 0",
    transform: "translateY(1px)",
    width: "1em",
  },
  ".cm-codice": {
    backgroundColor: "var(--muted)",
    borderRadius: "5px",
    fontSize: "0.9em",
  },
  ".cm-content > .cm-line.cm-titolo:first-child": { paddingTop: "0" },
  ".cm-corsivo": { fontStyle: "italic" },
  ".cm-grassetto": { fontWeight: "600" },
  ".cm-line.cm-citazione": {
    borderLeft: "3px solid var(--border)",
    color: "var(--muted-foreground)",
    paddingLeft: "0.9em",
  },
  ".cm-line.cm-riga-codice": {
    backgroundColor: "var(--muted)",
    fontFamily: "var(--font-mono, ui-monospace, Consolas, monospace)",
    fontSize: "0.88em",
    lineHeight: "1.6",
    padding: "0 0.9em",
  },
  ".cm-line.cm-riga-codice-fine, .cm-line.cm-riga-codice-solo": {
    borderBottomLeftRadius: "6px",
    borderBottomRightRadius: "6px",
    paddingBottom: "0.5em",
  },
  ".cm-line.cm-riga-codice-inizio, .cm-line.cm-riga-codice-solo": {
    borderTopLeftRadius: "6px",
    borderTopRightRadius: "6px",
    paddingTop: "0.5em",
  },
  ".cm-line.cm-titolo-1": {
    fontSize: "1.6em",
    fontWeight: "500",
    lineHeight: "1.25",
    padding: "0.9em 0 0.15em",
  },
  ".cm-line.cm-titolo-1, .cm-line.cm-titolo-2, .cm-line.cm-titolo-3": {
    fontFamily: DISPLAY,
    fontVariationSettings: "'FLAR' 100, 'VOLM' 50",
    letterSpacing: "-0.01em",
  },
  ".cm-line.cm-titolo-2": {
    fontSize: "1.35em",
    fontWeight: "500",
    lineHeight: "1.3",
    padding: "0.8em 0 0.1em",
  },
  ".cm-line.cm-titolo-3": {
    fontSize: "1.15em",
    lineHeight: "1.35",
    padding: "0.6em 0 0.05em",
  },
  ".cm-line.cm-titolo-4, .cm-line.cm-titolo-5, .cm-line.cm-titolo-6": {
    paddingTop: "0.5em",
  },
  ".cm-line.cm-voce": {
    paddingLeft: "calc(var(--livello) * 1.5em + 1.25em)",
    textIndent: "-1.25em",
  },
  ".cm-line.cm-voce-numerata": {
    paddingLeft: "calc(var(--livello) * 1.5em + 2em)",
    textIndent: "-2em",
  },
  // Una riga vuota separa i blocchi: basta mezza riga.
  ".cm-line.cm-vuota": { lineHeight: "1" },
  ".cm-link": {
    cursor: "pointer",
    textDecoration: "underline",
    textDecorationColor:
      "color-mix(in oklch, var(--foreground) 35%, transparent)",
    textUnderlineOffset: "3px",
  },
  ".cm-numero": {
    color: "var(--muted-foreground)",
    display: "inline-block",
    fontVariantNumeric: "tabular-nums",
    minWidth: "2em",
    textIndent: "0",
  },
  ".cm-punto": {
    color: "var(--muted-foreground)",
    display: "inline-block",
    textIndent: "0",
    width: "1.25em",
  },
  ".cm-separatore": {
    backgroundImage: "linear-gradient(var(--border), var(--border))",
    backgroundPosition: "0 50%",
    backgroundRepeat: "no-repeat",
    backgroundSize: "100% 1px",
  },
  ".cm-tabella": { overflowX: "auto", padding: "0.35em 0 0.5em" },
  // Le celle vanno a capo tra le parole, non dentro: l'a capo dell'editor spezzerebbe «Quando».
  ".cm-tabella table": {
    borderCollapse: "collapse",
    fontSize: "0.94em",
    lineHeight: "1.5",
    overflowWrap: "normal",
    whiteSpace: "normal",
    wordBreak: "normal",
  },
  ".cm-tabella td, .cm-tabella th": {
    border: "1px solid var(--border)",
    cursor: "text",
    minWidth: "3em",
    padding: "0.4em 0.8em",
    textAlign: "left",
    verticalAlign: "top",
  },
  ".cm-tabella th": {
    backgroundColor: "color-mix(in oklch, var(--muted) 60%, transparent)",
    fontWeight: "600",
  },
  ".cm-tempo": {
    alignItems: "center",
    backgroundColor: "var(--card)",
    border: "1px solid var(--border)",
    borderRadius: "999px",
    color: "var(--muted-foreground)",
    cursor: "pointer",
    display: "inline-flex",
    fontSize: "0.82em",
    fontVariantNumeric: "tabular-nums",
    gap: "3px",
    lineHeight: "1.6",
    padding: "0 7px 0 5px",
    verticalAlign: "1px",
  },
  ".cm-tempo svg": { fill: "var(--play)", height: "10px", width: "10px" },
  ".cm-tempo:hover": {
    borderColor: "var(--input)",
    color: "var(--foreground)",
  },
  // I titoli: `#`–`###` in Commissioner come i titoli dell'app, sotto i 32 px del titolo del Tape;
  // spazio sopra per staccarli dal blocco prima, poco sotto per legarli al loro paragrafo.
  ".cm-titolo": { fontWeight: "600" },
  ".cm-titolo-5, .cm-titolo-6": { color: "var(--muted-foreground)" },
});

/**
 * Le estensioni dell'editor ibrido: Markdown, decorazioni, tasti di formato e link. `apriLink`
 * apre un indirizzo fuori dalla WebView; un clic su un link lo apre se la sua riga non mostra il
 * Markdown, altrimenti con Ctrl. `tempi` serve alle pillole dei Tempi nella Nota.
 */
export function ibrido(
  apriLink: (url: string) => void,
  tempi: Tempi
): Extension {
  const comando = (formato: Formato) => (view: EditorView) =>
    applica(view, formato);
  return [
    markdownDelleNote,
    anteprima(apriLink, tempi),
    TEMA,
    Prec.high(
      keymap.of([
        { key: "Mod-b", run: comando("grassetto") },
        { key: "Mod-i", run: comando("corsivo") },
        { key: "Mod-k", run: comando("link") },
      ])
    ),
    EditorView.domEventHandlers({
      mousedown: (e, view) => {
        if (!(e.target instanceof Element && e.target.closest(".cm-link"))) {
          return false;
        }
        const pos = view.posAtCoords({ x: e.clientX, y: e.clientY });
        const url = pos === null ? null : linkIn(view.state, pos);
        const attiva = e.target.closest(".cm-attiva") !== null;
        if (!url || (attiva && !e.ctrlKey)) {
          return false;
        }
        e.preventDefault();
        apriLink(url);
        return true;
      },
    }),
  ];
}
