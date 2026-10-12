import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { syntaxTree } from "@codemirror/language";
import {
  type ChangeSpec,
  EditorSelection,
  type EditorState,
  type Line,
  type TransactionSpec,
} from "@codemirror/state";
import { elapsedText } from "@/features/recording/recording";
import { type TempoNota, tempiNellaNota } from "./tempi";

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
 * da spuntare, `tempo` la pillola di un Tempo nella Nota; gli altri sono stili. `attiva` è la riga
 * del cursore, che mostra il Markdown. `tabella` prende il posto delle righe di una tabella,
 * `nascosta` toglie una riga intera (i ``` di un blocco di codice) e `rigaCodice` è una riga del
 * riquadro di un blocco di codice. `citazione` e `separatore` sono le righe di una citazione e di
 * un `---`. `voce` è la prima riga di una voce d'elenco, che rientra come dice la sua `Voce`, e
 * `continua` un'altra riga della stessa voce; `numero` il numero di una voce di un elenco numerato. `vuota` è una riga vuota tra due blocchi, più bassa.
 */
export type Marcatura = { a: number; da: number } & (
  | {
      tipo:
        | "segno"
        | "punto"
        | "grassetto"
        | "corsivo"
        | "codice"
        | "attiva"
        | "nascosta"
        | "citazione"
        | "separatore"
        | "numero"
        | "vuota";
    }
  | { livello: number; tipo: "titolo" }
  | { tipo: "link"; url: string }
  | { fatta: boolean; tipo: "casella" }
  | { ms: number; tipo: "tempo" }
  | {
      allineamenti: Allineamento[];
      righe: Cella[][];
      tipo: "tabella";
      /** La voce d'elenco in cui sta la tabella, che rientra con lei. */
      voce: Voce | null;
    }
  | { bordo: Bordo; tipo: "rigaCodice" }
  | (Voce & { tipo: "voce" | "continua" })
);

/**
 * Quanto rientra una voce d'elenco: il `livello` di annidamento e la larghezza del segno, cioè le
 * `cifre` del numero di un elenco numerato (0 nei puntati) e se dopo c'è una `casella`.
 */
export interface Voce {
  casella: boolean;
  cifre: number;
  livello: number;
  numerata: boolean;
}

/** Dove sta una riga nel riquadro di un blocco di codice: prima, ultima, unica o in mezzo. */
export type Bordo = "inizio" | "fine" | "solo" | "";

/** Come la riga dei trattini allinea una colonna: `:--`, `:-:`, `--:` o nessuno. */
export type Allineamento = "sinistra" | "centro" | "destra" | null;

/** Una cella di una tabella: dove comincia il suo testo e i pezzi da mostrare. */
export interface Cella {
  da: number;
  pezzi: Pezzo[];
}

/**
 * Un tratto del testo di una cella con i suoi stili e, se è un link, l'indirizzo; con `ms` è la
 * pillola di un Tempo nella Nota e `testo` il tempo scritto.
 */
export interface Pezzo {
  ms?: number;
  stili: Stile[];
  testo: string;
  url?: string;
}

type Stile = "grassetto" | "corsivo" | "codice" | "link";

/** Un nodo dell'albero di Lezer, quanto serve qui. */
interface Nodo {
  firstChild: Nodo | null;
  from: number;
  lastChild: Nodo | null;
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
 * Le marcature della Nota. Fuori dalle righe attive i segni spariscono e caselle, punti d'elenco
 * e pillole dei Tempi nella Nota (solo prima della fine del Tape lungo `durataMs`) prendono il loro
 * posto; le righe attive (solo con `focus`) mostrano il Markdown, con gli stili.
 */
export function marcature(
  state: EditorState,
  focus: boolean,
  durataMs = 0
): Marcatura[] {
  const attive = focus ? righeAttive(state) : new Set<number>();
  const fuori = (pos: number) => !attive.has(state.doc.lineAt(pos).number);
  /** Un blocco su più righe mostra il Markdown se una sua riga è attiva. */
  const bloccoFuori = (da: number, a: number) => {
    const ultima = state.doc.lineAt(a).number;
    for (let n = state.doc.lineAt(da).number; n <= ultima; n += 1) {
      if (attive.has(n)) {
        return false;
      }
    }
    return true;
  };
  const tempi = tempiNellaNota(state.doc.toString()).filter(
    (t) => t.ms < durataMs
  );
  const lista: Marcatura[] = [];
  /** I tratti delle tabelle mostrate a celle: i loro Tempi stanno nelle celle. */
  const tabelle: [number, number][] = [];
  /** I tratti dei blocchi di codice: le loro righe vuote sono codice. */
  const codici: [number, number][] = [];
  for (const n of attive) {
    const riga = state.doc.line(n);
    lista.push({ a: riga.to, da: riga.from, tipo: "attiva" });
  }
  syntaxTree(state).iterate({
    enter: (ref) => {
      const nodo: Nodo = ref.node;
      const { from, name, to } = nodo;
      // Una tabella in una citazione resta Markdown: i `>` stanno dentro le sue righe.
      if (name === "Table" && aCapo(state, from) && bloccoFuori(from, to)) {
        const t = tabella(state, nodo, tempi);
        tabelle.push([t.da, t.a]);
        lista.push(t);
        return false;
      }
      if (name === "FencedCode") {
        codici.push([from, to]);
        lista.push(...codice(state, nodo, bloccoFuori(from, to)));
      } else {
        lista.push(...righeDelBlocco(state, nodo, fuori));
      }
      const s = stile(state, nodo);
      if (s) {
        lista.push(s);
      }
      if (!fuori(from)) {
        return;
      }
      for (const segno of segni(state, nodo)) {
        lista.push(segno);
      }
    },
  });
  for (const { a, da, ms } of tempi) {
    if (fuori(da) && !tabelle.some(([x, y]) => x <= da && da < y)) {
      lista.push({ a, da, ms, tipo: "tempo" });
    }
  }
  for (let n = 1; n <= state.doc.lines; n += 1) {
    const riga = state.doc.line(n);
    if (
      riga.text.trim() === "" &&
      !codici.some(([x, y]) => x <= riga.from && riga.from <= y)
    ) {
      lista.push({ a: riga.to, da: riga.from, tipo: "vuota" });
    }
  }
  return lista;
}

/**
 * Le marcature di riga del nodo: titolo e righe di una citazione sempre, il separatore e il rientro
 * di una voce d'elenco solo sulle righe `fuori` da quella del cursore.
 */
function righeDelBlocco(
  state: EditorState,
  nodo: Nodo,
  fuori: (pos: number) => boolean
): Marcatura[] {
  const { from, name, parent, to } = nodo;
  const riga = state.doc.lineAt(from);
  const titolo = TITOLO.exec(name);
  if (titolo) {
    const livello = Number(titolo[1]);
    return [{ a: riga.to, da: riga.from, livello, tipo: "titolo" }];
  }
  if (name === "Blockquote" && parent?.name !== "Blockquote") {
    const lista: Marcatura[] = [];
    const ultima = state.doc.lineAt(to).number;
    for (let n = riga.number; n <= ultima; n += 1) {
      const r = state.doc.line(n);
      lista.push({ a: r.to, da: r.from, tipo: "citazione" });
    }
    return lista;
  }
  if (name === "HorizontalRule" && fuori(from)) {
    return [{ a: to, da: from, tipo: "separatore" }];
  }
  if (name === "ListItem" && aCapo(state, from)) {
    const voce = voceDi(state, nodo);
    const lista: Marcatura[] = fuori(from)
      ? [{ ...voce, a: riga.to, da: riga.from, tipo: "voce" }]
      : [];
    for (const r of continuazioni(state, nodo)) {
      if (fuori(r.from)) {
        const testo = r.from + spazi(state, r.from);
        lista.push({ ...voce, a: r.to, da: r.from, tipo: "continua" });
        if (testo > r.from) {
          lista.push({ a: testo, da: r.from, tipo: "segno" });
        }
      }
    }
    return lista;
  }
  return [];
}

/** Come rientra la voce d'elenco `nodo`. */
function voceDi(state: EditorState, nodo: Nodo): Voce {
  let livello = -1;
  for (let n = nodo.parent; n; n = n.parent) {
    if (n.name === "BulletList" || n.name === "OrderedList") {
      livello += 1;
    }
  }
  const numerata = nodo.parent?.name === "OrderedList";
  const segno = figlio(nodo, "ListMark");
  const cifre =
    numerata && segno
      ? state.doc.sliceString(segno.from, segno.to).length - 1
      : 0;
  const casella = segno?.nextSibling?.name === "Task";
  return { casella, cifre, livello, numerata };
}

/**
 * Le righe della voce `nodo` dopo la prima che sono testo suo: non vuote, non di una voce annidata,
 * né di un blocco di codice o di una tabella.
 */
function continuazioni(state: EditorState, nodo: Nodo): Line[] {
  const trovate: Line[] = [];
  const ultima = state.doc.lineAt(nodo.to).number;
  for (let n = state.doc.lineAt(nodo.from).number + 1; n <= ultima; n += 1) {
    const riga = state.doc.line(n);
    if (riga.text.trim() === "") {
      continue;
    }
    let sopra: Nodo | null = syntaxTree(state).resolveInner(
      riga.from + spazi(state, riga.from),
      1
    );
    while (
      sopra &&
      sopra.name !== "ListItem" &&
      sopra.name !== "FencedCode" &&
      sopra.name !== "Table"
    ) {
      sopra = sopra.parent;
    }
    if (sopra?.name === "ListItem" && sopra.from === nodo.from) {
      trovate.push(riga);
    }
  }
  return trovate;
}

/**
 * Le righe del blocco di codice `nodo`: fuori dal cursore quelle dei ``` spariscono, le altre
 * fanno il riquadro.
 */
function codice(state: EditorState, nodo: Nodo, fuori: boolean): Marcatura[] {
  const prima = state.doc.lineAt(nodo.from).number;
  const ultima = state.doc.lineAt(nodo.to).number;
  const chiusa = ultima > prima && nodo.lastChild?.name === "CodeMark";
  const recinti = chiusa ? [prima, ultima] : [prima];
  // Un blocco senza righe di codice tiene i ``` per non sparire.
  const vuoto = recinti.length === ultima - prima + 1;
  const nascoste = new Set(fuori && !vuoto ? recinti : []);
  const lista: Marcatura[] = [];
  const visibili: number[] = [];
  for (let n = prima; n <= ultima; n += 1) {
    const riga = state.doc.line(n);
    if (nascoste.has(n)) {
      lista.push({ a: riga.to, da: riga.from, tipo: "nascosta" });
    } else {
      visibili.push(n);
    }
  }
  for (const [i, n] of visibili.entries()) {
    const riga = state.doc.line(n);
    const inizio = i === 0;
    const fine = i + 1 === visibili.length;
    let bordo: Bordo = "";
    if (inizio && fine) {
      bordo = "solo";
    } else if (inizio) {
      bordo = "inizio";
    } else if (fine) {
      bordo = "fine";
    }
    lista.push({ a: riga.to, bordo, da: riga.from, tipo: "rigaCodice" });
  }
  return lista;
}

/** Lo stile in linea del nodo: grassetto, corsivo, codice o un link con il suo indirizzo. */
function stile(state: EditorState, nodo: Nodo): Marcatura | null {
  const { from, name, to } = nodo;
  if (name === "StrongEmphasis") {
    return { a: to, da: from, tipo: "grassetto" };
  }
  if (name === "Emphasis") {
    return { a: to, da: from, tipo: "corsivo" };
  }
  if (name === "InlineCode") {
    return { a: to, da: from, tipo: "codice" };
  }
  if (
    name === "Link" ||
    name === "Autolink" ||
    (name === "URL" && nudo(nodo))
  ) {
    const url = urlDi(state, nodo);
    return url ? { a: to, da: from, tipo: "link", url } : null;
  }
  return null;
}

/** Le colonne della riga dei trattini, come `|:--|:-:|--:|`. */
function allineamenti(testo: string): Allineamento[] {
  return testo
    .trim()
    .replace(/^\||\|$/g, "")
    .split("|")
    .map((c) => {
      const t = c.trim();
      if (t.startsWith(":") && t.endsWith(":")) {
        return "centro";
      }
      if (t.endsWith(":")) {
        return "destra";
      }
      return t.startsWith(":") ? "sinistra" : null;
    });
}

/**
 * La tabella `nodo` a celle, sulle sue righe intere. Le celle si contano tra i `|` (una cella vuota
 * non ha nodo); una riga ha tante celle quante l'intestazione, come in GitHub.
 */
function tabella(
  state: EditorState,
  nodo: Nodo,
  tempi: TempoNota[]
): Marcatura & { tipo: "tabella" } {
  let colonne: Allineamento[] = [];
  let larghezza: number | null = null;
  const file: Cella[][] = [];
  for (let riga = nodo.firstChild; riga; riga = riga.nextSibling) {
    if (riga.name === "TableDelimiter") {
      colonne = allineamenti(state.doc.sliceString(riga.from, riga.to));
      continue;
    }
    const celle = celleDellaRiga(state, riga, tempi);
    larghezza ??= celle.length;
    const vuota = { da: riga.to, pezzi: [] };
    file.push(Array.from({ length: larghezza }, (_, i) => celle[i] ?? vuota));
  }
  return {
    a: state.doc.lineAt(nodo.to).to,
    allineamenti: Array.from(
      { length: larghezza ?? 0 },
      (_, i) => colonne[i] ?? null
    ),
    da: state.doc.lineAt(nodo.from).from,
    righe: file,
    tipo: "tabella",
    voce: voceSopra(state, nodo),
  };
}

/** La voce d'elenco più vicina che contiene `nodo`, o `null`. */
function voceSopra(state: EditorState, nodo: Nodo): Voce | null {
  for (let n = nodo.parent; n; n = n.parent) {
    if (n.name === "ListItem") {
      return voceDi(state, n);
    }
  }
  return null;
}

/** Le celle di una riga di tabella, tra i suoi `|`. */
function celleDellaRiga(
  state: EditorState,
  riga: Nodo,
  tempi: TempoNota[]
): Cella[] {
  const tagli = [riga.from - 1];
  for (let c = riga.firstChild; c; c = c.nextSibling) {
    if (c.name === "TableDelimiter") {
      tagli.push(c.from);
    }
  }
  tagli.push(riga.to);
  const celle: Cella[] = [];
  for (const [i, fine] of tagli.slice(1).entries()) {
    const inizio = (tagli[i] as number) + 1;
    const testo = state.doc.sliceString(inizio, fine);
    // I `|` in testa e in coda chiudono la riga, non aprono una cella.
    if (testo.trim() === "" && (i === 0 || i + 2 === tagli.length)) {
      continue;
    }
    const da = inizio + testo.length - testo.trimStart().length;
    const a = Math.max(da, fine - (testo.length - testo.trimEnd().length));
    celle.push({ da, pezzi: pezzi(state, da, a, tempi) });
  }
  return celle;
}

/** Il testo tra `da` e `a` a pezzi: senza i segni, con gli stili e le pillole dei Tempi. */
function pezzi(
  state: EditorState,
  da: number,
  a: number,
  tempi: TempoNota[]
): Pezzo[] {
  const stili: Marcatura[] = [];
  const nascosti: Marcatura[] = [];
  syntaxTree(state).iterate({
    enter: (ref) => {
      const s = stile(state, ref.node);
      if (s) {
        stili.push(s);
      }
      for (const m of segni(state, ref.node)) {
        nascosti.push(m);
      }
      // «\|» si legge «|».
      if (ref.name === "Escape") {
        nascosti.push({ a: ref.from + 1, da: ref.from, tipo: "segno" });
      }
    },
    from: da,
    to: a,
  });
  const dentro = tempi.filter((t) => da <= t.da && t.a <= a);
  const tagli = new Set([da, a]);
  for (const m of [...stili, ...nascosti, ...dentro]) {
    for (const p of [m.da, m.a]) {
      if (da < p && p < a) {
        tagli.add(p);
      }
    }
  }
  const punti = [...tagli].sort((x, y) => x - y);
  /** Il tratto tra `p` e `q`: la pillola se un Tempo comincia lì, niente se è nascosto. */
  const tratto = (p: number, q: number): Pezzo | null => {
    const tempo = dentro.find((t) => t.da <= p && q <= t.a);
    if (tempo) {
      const testo = state.doc.sliceString(tempo.da + 1, tempo.a - 1);
      return tempo.da === p ? { ms: tempo.ms, stili: [], testo } : null;
    }
    if (nascosti.some((m) => m.da <= p && q <= m.a)) {
      return null;
    }
    const sopra = stili.filter((m) => m.da <= p && q <= m.a);
    const link = sopra.find((m) => m.tipo === "link");
    return {
      stili: sopra.map((m) => m.tipo as Stile),
      testo: state.doc.sliceString(p, q),
      ...(link?.tipo === "link" ? { url: link.url } : {}),
    };
  };
  const lista: Pezzo[] = [];
  let ultimo: Pezzo | null = null;
  for (const [i, q] of punti.slice(1).entries()) {
    const p = tratto(punti[i] as number, q);
    // Due tratti di fila con gli stessi stili sono un pezzo solo.
    if (
      p &&
      ultimo &&
      p.ms === undefined &&
      ultimo.ms === undefined &&
      ultimo.url === p.url &&
      ultimo.stili.join() === p.stili.join()
    ) {
      ultimo.testo += p.testo;
    } else if (p) {
      lista.push(p);
      ultimo = p;
    }
  }
  return lista;
}

const PAROLA = /\S/;

/**
 * Scrive al cursore, al posto della selezione, il Tempo nella Nota `ms` (il tempo del player),
 * staccato dalle parole intorno, e mette il cursore dopo.
 */
export function inserisciTempo(
  state: EditorState,
  ms: number
): TransactionSpec {
  const tempo = `[${elapsedText(ms)}]`;
  return state.changeByRange(({ from, to }) => {
    const prima = PAROLA.test(state.doc.sliceString(from - 1, from)) ? " " : "";
    const spazio = state.doc.sliceString(to, to + 1) === " ";
    const insert = `${prima}${tempo}${spazio ? "" : " "}`;
    return {
      changes: { from, insert, to },
      range: EditorSelection.cursor(from + insert.length + (spazio ? 1 : 0)),
    };
  });
}

/** Gli spazi da `pos` in poi, sulla stessa riga. */
function spazi(state: EditorState, pos: number): number {
  const resto = state.doc.sliceString(pos, state.doc.lineAt(pos).to);
  return resto.length - resto.trimStart().length;
}

/** Prima di `pos` sulla sua riga solo spazi: una voce d'elenco a capo, non dentro una citazione. */
function aCapo(state: EditorState, pos: number): boolean {
  const riga = state.doc.lineAt(pos);
  return state.doc.sliceString(riga.from, pos).trim() === "";
}

/** I segni del nodo da nascondere fuori dalla riga del cursore, con caselle e punti. */
function segni(state: EditorState, nodo: Nodo): Marcatura[] {
  const { from, name, parent, to } = nodo;
  const sopra = parent?.name ?? "";
  if (name === "HeaderMark" && sopra.startsWith("ATXHeading")) {
    // Con gli spazi che seguono: «## Titolo» si legge «Titolo».
    return [{ a: to + spazi(state, to), da: from, tipo: "segno" }];
  }
  if (name === "QuoteMark") {
    // Con lo spazio che segue: «> Testo» si legge «Testo».
    const spazio = state.doc.sliceString(to, to + 1) === " " ? 1 : 0;
    return [{ a: to + spazio, da: from, tipo: "segno" }];
  }
  if (name === "HorizontalRule") {
    return [{ a: to, da: from, tipo: "segno" }];
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
  if (name === "ListMark" && parent?.name === "ListItem") {
    return segniDellaVoce(state, nodo);
  }
  if (name === "TaskMarker") {
    const fatta = state.doc.sliceString(from + 1, to - 1).toLowerCase() === "x";
    return [{ a: to + spazi(state, to), da: from, fatta, tipo: "casella" }];
  }
  return [];
}

/**
 * Il segno `nodo` di una voce d'elenco: il rientro scritto sparisce (la voce rientra del suo
 * livello), il numero resta con lo spazio che lo segue, il segno di una casella sparisce con lo
 * spazio e gli altri diventano un punto.
 */
function segniDellaVoce(state: EditorState, nodo: Nodo): Marcatura[] {
  const { from, to } = nodo;
  const riga = state.doc.lineAt(from);
  const lista: Marcatura[] =
    from > riga.from && aCapo(state, from)
      ? [{ a: from, da: riga.from, tipo: "segno" }]
      : [];
  const dopo = to + spazi(state, to);
  if (nodo.parent?.parent?.name !== "BulletList") {
    lista.push({ a: dopo, da: from, tipo: "numero" });
    return lista;
  }
  const compito = nodo.nextSibling?.name === "Task" ? nodo.nextSibling : null;
  const casella = compito ? figlio(compito, "TaskMarker") : null;
  lista.push(
    casella
      ? { a: casella.from, da: from, tipo: "segno" }
      : { a: dopo, da: from, tipo: "punto" }
  );
  return lista;
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
