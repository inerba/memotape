import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { syntaxTree } from "@codemirror/language";
import {
  type ChangeSpec,
  EditorSelection,
  type EditorState,
  type TransactionSpec,
} from "@codemirror/state";

/**
 * Il Markdown delle Note come il Live Preview di Obsidian (ADR-0032): quello che si vede di ogni
 * pezzo del testo e i comandi di formato. Puro, sopra lo stato di CodeMirror: la vista
 * (`ibrido.ts`) trasforma le marcature in decorazioni. Il testo non si ricostruisce mai: i comandi
 * cambiano solo i segni che toccano.
 */

/** Il Markdown con le estensioni di GitHub: caselle e link nudi. */
export const markdownDelleNote = markdown({ base: markdownLanguage });

/**
 * Come si mostra un tratto del testo: `segno` sparisce, `punto` diventa «•», `casella` una casella
 * da spuntare; gli altri sono stili. `attiva` è la riga del cursore, che mostra il Markdown.
 */
export type Marcatura = { a: number; da: number } & (
  | {
      tipo: "segno" | "punto" | "grassetto" | "corsivo" | "codice" | "attiva";
    }
  | { livello: number; tipo: "titolo" }
  | { tipo: "link"; url: string }
  | { fatta: boolean; tipo: "casella" }
);

/** Un nodo dell'albero di Lezer, quanto serve qui. */
interface Nodo {
  firstChild: Nodo | null;
  from: number;
  name: string;
  nextSibling: Nodo | null;
  parent: Nodo | null;
  to: number;
}

const TITOLO = /^ATXHeading(\d)$/;

/** Le righe con un pezzo di selezione: con il focus mostrano il Markdown. */
function righeAttive(state: EditorState): Set<number> {
  const attive = new Set<number>();
  for (const r of state.selection.ranges) {
    const ultima = state.doc.lineAt(r.to).number;
    for (let n = state.doc.lineAt(r.from).number; n <= ultima; n += 1) {
      attive.add(n);
    }
  }
  return attive;
}

function figlio(nodo: Nodo, nome: string): Nodo | null {
  for (let c = nodo.firstChild; c; c = c.nextSibling) {
    if (c.name === nome) {
      return c;
    }
  }
  return null;
}

/**
 * L'indirizzo di un link: `Link` con l'indirizzo tra tonde, `Autolink` tra `<>`, o un `URL` nudo
 * (`www.…` diventa `https://www.…`, un indirizzo email `mailto:`). `null` per gli altri nodi.
 */
function urlDi(state: EditorState, nodo: Nodo): string | null {
  const url = nodo.name === "URL" ? nodo : figlio(nodo, "URL");
  if (
    !url ||
    (nodo.name !== "URL" && nodo.name !== "Link" && nodo.name !== "Autolink")
  ) {
    return null;
  }
  const testo = state.doc.sliceString(url.from, url.to).replace(/^<|>$/g, "");
  if (testo.startsWith("www.")) {
    return `https://${testo}`;
  }
  return !testo.includes(":") && testo.includes("@")
    ? `mailto:${testo}`
    : testo;
}

/** Un `URL` nudo, fuori da un link o da un'immagine. */
function nudo(nodo: Nodo): boolean {
  const sopra = nodo.parent?.name;
  return sopra !== "Link" && sopra !== "Autolink" && sopra !== "Image";
}

/**
 * Le marcature della Nota. Fuori dalle righe attive i segni spariscono e caselle e punti
 * d'elenco prendono il loro posto; le righe attive (solo con `focus`) mostrano il Markdown, con
 * gli stili.
 */
export function marcature(state: EditorState, focus: boolean): Marcatura[] {
  const attive = focus ? righeAttive(state) : new Set<number>();
  const fuori = (pos: number) => !attive.has(state.doc.lineAt(pos).number);
  const lista: Marcatura[] = [];
  for (const n of attive) {
    const riga = state.doc.line(n);
    lista.push({ a: riga.to, da: riga.from, tipo: "attiva" });
  }
  syntaxTree(state).iterate({
    enter: (ref) => {
      const nodo: Nodo = ref.node;
      const { from, name, to } = nodo;
      const titolo = TITOLO.exec(name);
      if (titolo) {
        const riga = state.doc.lineAt(from);
        lista.push({
          a: riga.to,
          da: riga.from,
          livello: Number(titolo[1]),
          tipo: "titolo",
        });
      } else if (name === "StrongEmphasis") {
        lista.push({ a: to, da: from, tipo: "grassetto" });
      } else if (name === "Emphasis") {
        lista.push({ a: to, da: from, tipo: "corsivo" });
      } else if (name === "InlineCode") {
        lista.push({ a: to, da: from, tipo: "codice" });
      } else if (
        name === "Link" ||
        name === "Autolink" ||
        (name === "URL" && nudo(nodo))
      ) {
        const url = urlDi(state, nodo);
        if (url) {
          lista.push({ a: to, da: from, tipo: "link", url });
        }
      }
      if (!fuori(from)) {
        return;
      }
      for (const segno of segni(state, nodo)) {
        lista.push(segno);
      }
    },
  });
  return lista;
}

/** I segni del nodo da nascondere fuori dalla riga del cursore, con caselle e punti. */
function segni(state: EditorState, nodo: Nodo): Marcatura[] {
  const { from, name, parent, to } = nodo;
  const sopra = parent?.name ?? "";
  if (name === "HeaderMark" && sopra.startsWith("ATXHeading")) {
    // Con gli spazi che seguono: «## Titolo» si legge «Titolo».
    const dopo = state.doc.sliceString(to, parent?.to ?? to);
    const spazi = dopo.length - dopo.trimStart().length;
    return [{ a: to + spazi, da: from, tipo: "segno" }];
  }
  if (
    name === "EmphasisMark" ||
    (name === "CodeMark" && sopra === "InlineCode")
  ) {
    return [{ a: to, da: from, tipo: "segno" }];
  }
  if ((name === "Link" || name === "Autolink") && urlDi(state, nodo)) {
    return segniDelLink(nodo);
  }
  if (name === "ListMark" && parent?.parent?.name === "BulletList") {
    // Il segno d'elenco di una casella sparisce con lo spazio; gli altri diventano un punto.
    const compito = nodo.nextSibling?.name === "Task" ? nodo.nextSibling : null;
    const casella = compito ? figlio(compito, "TaskMarker") : null;
    return casella
      ? [{ a: casella.from, da: from, tipo: "segno" }]
      : [{ a: to, da: from, tipo: "punto" }];
  }
  if (name === "TaskMarker") {
    const fatta = state.doc.sliceString(from + 1, to - 1).toLowerCase() === "x";
    return [{ a: to, da: from, fatta, tipo: "casella" }];
  }
  return [];
}

/**
 * «[testo](indirizzo)» si legge «testo»: via la quadra iniziale e tutto dalla chiusa in poi.
 * «<indirizzo>» perde le parentesi angolari.
 */
function segniDelLink(nodo: Nodo): Marcatura[] {
  const { from, name, to } = nodo;
  const aperta = nodo.firstChild;
  const chiusa = aperta?.nextSibling;
  if (name === "Link" && aperta && chiusa) {
    return [
      { a: aperta.to, da: aperta.from, tipo: "segno" },
      { a: to, da: chiusa.from, tipo: "segno" },
    ];
  }
  const url = figlio(nodo, "URL");
  return url
    ? [
        { a: url.from, da: from, tipo: "segno" },
        { a: to, da: url.to, tipo: "segno" },
      ]
    : [];
}

/** Il nodo più interno in `pos` e i suoi antenati, dal basso. */
function* risali(state: EditorState, pos: number): Generator<Nodo> {
  for (
    let nodo: Nodo | null = syntaxTree(state).resolveInner(pos, 1);
    nodo;
    nodo = nodo.parent
  ) {
    yield nodo;
  }
}

/** L'indirizzo del link in `pos`, o `null`. */
export function linkIn(state: EditorState, pos: number): string | null {
  for (const nodo of risali(state, pos)) {
    if (
      nodo.name === "Link" ||
      nodo.name === "Autolink" ||
      (nodo.name === "URL" && nudo(nodo))
    ) {
      return urlDi(state, nodo);
    }
  }
  return null;
}

/** Spunta o toglie la casella in `pos`, cambiando solo il carattere tra le quadre. */
export function spunta(
  state: EditorState,
  pos: number
): TransactionSpec | null {
  for (const nodo of risali(state, pos)) {
    if (nodo.name === "TaskMarker") {
      const fatta = state.doc.sliceString(nodo.from + 1, nodo.to - 1) !== " ";
      return {
        changes: {
          from: nodo.from + 1,
          insert: fatta ? " " : "x",
          to: nodo.to - 1,
        },
      };
    }
  }
  return null;
}

export type Formato = "grassetto" | "corsivo" | "link" | "elenco" | "daFare";

const AVVOLGI: Partial<Record<Formato, string>> = {
  corsivo: "_",
  grassetto: "**",
};

/** Il comando di formato sulla selezione: avvolge, mette il link o cambia il segno delle righe. */
export function formatta(
  state: EditorState,
  formato: Formato
): TransactionSpec {
  const segno = AVVOLGI[formato];
  if (segno) {
    return avvolgi(state, segno);
  }
  if (formato === "link") {
    return state.changeByRange(({ from, to }) => ({
      changes: [
        { from, insert: "[" },
        { from: to, insert: "]()" },
      ],
      range: EditorSelection.cursor(from === to ? from + 1 : to + 3),
    }));
  }
  return righe(state, formato === "daFare" ? "- [ ] " : "- ");
}

/** Avvolge la selezione in `segno`, o lo toglie se c'è già, dentro o fuori. */
function avvolgi(state: EditorState, segno: string): TransactionSpec {
  const l = segno.length;
  return state.changeByRange(({ from, to }) => {
    const testo = state.doc.sliceString(from, to);
    if (
      state.doc.sliceString(from - l, from) === segno &&
      state.doc.sliceString(to, to + l) === segno
    ) {
      return {
        changes: [
          { from: from - l, to: from },
          { from: to, to: to + l },
        ],
        range: EditorSelection.range(from - l, to - l),
      };
    }
    if (
      testo.length >= 2 * l &&
      testo.startsWith(segno) &&
      testo.endsWith(segno)
    ) {
      const dentro = testo.slice(l, -l);
      return {
        changes: { from, insert: dentro, to },
        range: EditorSelection.range(from, from + dentro.length),
      };
    }
    return {
      changes: [
        { from, insert: segno },
        { from: to, insert: segno },
      ],
      range: EditorSelection.range(from + l, to + l),
    };
  });
}

/** Il segno d'elenco o di casella in testa a una riga, dopo il rientro. */
const PREFISSO = /^(\s*)((?:[-*+] (\[[ xX]\] )?)?)/;

/**
 * Mette `prefisso` («- » o «- [ ] ») alle righe della selezione al posto del loro segno, o lo
 * toglie se ce l'hanno già tutte. Le righe vuote di una selezione lunga restano come sono.
 */
function righe(state: EditorState, prefisso: string): TransactionSpec {
  const casella = prefisso !== "- ";
  const linee = [...righeAttive(state)].map((n) => state.doc.line(n));
  const scritte = linee.length > 1 ? linee.filter((l) => l.text.trim()) : linee;
  const prefissi = scritte.map((linea) => {
    const [, rientro = "", segno = "", quadre] =
      PREFISSO.exec(linea.text) ?? [];
    const da = linea.from + rientro.length;
    return {
      a: da + segno.length,
      da,
      gia: segno !== "" && Boolean(quadre) === casella,
    };
  });
  const togli = prefissi.every((s) => s.gia);
  const changes = state.changes(
    prefissi.map(
      ({ a, da }): ChangeSpec => ({
        from: da,
        insert: togli ? "" : prefisso,
        to: a,
      })
    )
  );
  return {
    changes,
    selection: EditorSelection.create(
      state.selection.ranges.map((r) =>
        r.empty
          ? EditorSelection.cursor(changes.mapPos(r.head, 1))
          : r.map(changes)
      ),
      state.selection.mainIndex
    ),
  };
}
