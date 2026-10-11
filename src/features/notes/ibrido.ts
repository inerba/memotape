import { syntaxTree } from "@codemirror/language";
import { type Extension, Prec, type Range } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  keymap,
  ViewPlugin,
  type ViewUpdate,
  WidgetType,
} from "@codemirror/view";
import {
  type Formato,
  formatta,
  linkIn,
  type Marcatura,
  marcature,
  markdownDelleNote,
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

/** La pillola di un Tempo nella Nota: un clic porta lì il player, anche in sola lettura. */
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
    const pillola = document.createElement("button");
    pillola.type = "button";
    pillola.className = "cm-tempo";
    pillola.tabIndex = -1;
    pillola.title = this.tempi.etichetta(this.testo);
    pillola.setAttribute("aria-label", pillola.title);
    const icona = document.createElementNS(SVG, "svg");
    icona.setAttribute("viewBox", "0 0 24 24");
    icona.setAttribute("aria-hidden", "true");
    const triangolo = document.createElementNS(SVG, "path");
    triangolo.setAttribute("d", "M7 4.5v15l12-7.5z");
    icona.append(triangolo);
    pillola.append(icona, this.testo);
    // Il cursore resta dov'è: la pillola porta il player, non sposta la scrittura.
    pillola.addEventListener("mousedown", (e) => e.preventDefault());
    pillola.addEventListener("click", (e) => {
      e.preventDefault();
      this.tempi.salta(this.ms);
    });
    return pillola;
  }
  override ignoreEvent() {
    return true;
  }
}

const NASCOSTO = Decoration.replace({});
const PUNTO = Decoration.replace({ widget: new Punto() });
const STILE = {
  attiva: Decoration.line({ class: "cm-attiva" }),
  codice: Decoration.mark({ class: "cm-codice" }),
  corsivo: Decoration.mark({ class: "cm-corsivo" }),
  grassetto: Decoration.mark({ class: "cm-grassetto" }),
  link: Decoration.mark({ class: "cm-link" }),
};

function decorazione(
  state: EditorView["state"],
  m: Marcatura,
  tempi: Tempi
): Range<Decoration> | null {
  switch (m.tipo) {
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
      return STILE.attiva.range(m.da);
    default:
      return m.da === m.a ? null : STILE[m.tipo].range(m.da, m.a);
  }
}

function decorazioni(view: EditorView, tempi: Tempi): DecorationSet {
  const ranges = marcature(view.state, view.hasFocus, tempi.durataMs())
    .map((m) => decorazione(view.state, m, tempi))
    .filter((r) => r !== null);
  return Decoration.set(ranges, true);
}

function anteprima(tempi: Tempi) {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = decorazioni(view, tempi);
      }
      update(u: ViewUpdate) {
        if (
          u.docChanged ||
          u.selectionSet ||
          u.focusChanged ||
          syntaxTree(u.startState) !== syntaxTree(u.state)
        ) {
          this.decorations = decorazioni(u.view, tempi);
        }
      }
    },
    { decorations: (plugin) => plugin.decorations }
  );
}

/** Il tema dell'editor ibrido, solo con i token di DESIGN.md. */
const TEMA = EditorView.theme({
  ".cm-attiva": {
    backgroundColor: "color-mix(in oklch, var(--accent) 45%, transparent)",
    borderRadius: "6px",
  },
  ".cm-attiva .cm-link": { cursor: "text" },
  ".cm-casella": {
    accentColor: "var(--play)",
    cursor: "pointer",
    margin: "0 0.3em 0 0",
    transform: "translateY(1px)",
    width: "1em",
  },
  ".cm-codice": {
    backgroundColor: "var(--muted)",
    borderRadius: "5px",
    fontSize: "0.9em",
  },
  ".cm-corsivo": { fontStyle: "italic" },
  ".cm-grassetto": { fontWeight: "600" },
  ".cm-link": {
    cursor: "pointer",
    textDecoration: "underline",
    textDecorationColor:
      "color-mix(in oklch, var(--foreground) 35%, transparent)",
    textUnderlineOffset: "3px",
  },
  ".cm-punto": { color: "var(--muted-foreground)" },
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
  ".cm-titolo": { fontWeight: "600" },
  ".cm-titolo-1": { fontSize: "1.25em" },
  ".cm-titolo-2": { fontSize: "1.0625em" },
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
    anteprima(tempi),
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
