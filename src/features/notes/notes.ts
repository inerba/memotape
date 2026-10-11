import type { AppError, Nota, NotaInRegistrazione } from "@/bindings";

/** Chi ha scritto una Nota l'ultima volta e quando. */
export type Firma = Pick<Nota, "autore" | "modificata">;

/**
 * Una Nota della riga dei nomi: il nome e il testo su disco, `null` se la Nota non c'è ancora, con
 * la firma dell'ultima scrittura (manca in una Nota nuova).
 */
export interface NotaAperta {
  firma?: Firma;
  nome: string;
  salvato: string | null;
}

/** Il salvataggio di una bozza: `atteso` è il testo su disco da cui parte (ADR-0032). */
export interface Salvataggio {
  atteso: string;
  nome: string;
  testo: string;
}

/** Il nome più lungo di una Nota, in caratteri, come `tape::NOME_NOTA_MAX`. */
const NOME_MAX = 60;

/**
 * Le Note della riga dei nomi, in ordine di creazione. In un Tape senza Note una sola, nuova, con
 * il nome `predefinito`: nasce con il primo carattere.
 */
export function noteIniziali(note: Nota[], predefinito: string): NotaAperta[] {
  return note.length > 0
    ? note.map(daDisco)
    : [{ nome: predefinito, salvato: null }];
}

function daDisco({ autore, modificata, nome, testo }: Nota): NotaAperta {
  return { firma: { autore, modificata }, nome, salvato: testo };
}

/**
 * Le Note `suDisco`, rilette al ritorno in primo piano (un Assistente può averle cambiate), al
 * posto di quelle mostrate, `note`, che hanno le `bozze`. Una Nota senza bozza prende testo e firma
 * del disco. Una con bozza cambiata su disco resta com'era, con la bozza e il testo da cui parte, e
 * va nei `conflitti` con la Nota su disco: l'utente sceglie «Tieni le mie» o «Usa le sue». Le Note
 * sparite dal disco se ne vanno, tranne le nuove e quelle con bozza, che restano da creare.
 */
export function rilette(
  note: NotaAperta[],
  suDisco: Nota[],
  bozze: Readonly<Record<string, string>>,
  predefinito: string
): { conflitti: Record<string, Nota>; note: NotaAperta[] } {
  const conflitti: Record<string, Nota> = {};
  const lette = suDisco.map((d) => {
    const locale = note.find((n) => n.nome === d.nome);
    const bozza = bozze[d.nome];
    if (
      locale &&
      bozza !== undefined &&
      bozza !== d.testo &&
      d.testo !== (locale.salvato ?? "")
    ) {
      conflitti[d.nome] = d;
      return locale;
    }
    return daDisco(d);
  });
  const resto = note
    .filter(
      (n) =>
        !suDisco.some((d) => d.nome === n.nome) &&
        (n.salvato === null || n.nome in bozze)
    )
    .map((n) => ({ nome: n.nome, salvato: null }));
  const tutte = [...lette, ...resto];
  return {
    conflitti,
    note: tutte.length > 0 ? tutte : noteIniziali([], predefinito),
  };
}

/** L'ora della firma `modificata` (ISO 8601): solo l'ora se è di oggi, se no anche il giorno. */
export function oraDellaFirma(
  modificata: string,
  adesso: Date,
  locale: string
): string {
  const quando = new Date(modificata);
  const ora: Intl.DateTimeFormatOptions = {
    hour: "2-digit",
    minute: "2-digit",
  };
  return quando.toDateString() === adesso.toDateString()
    ? quando.toLocaleTimeString(locale, ora)
    : quando.toLocaleString(locale, { day: "numeric", month: "short", ...ora });
}

/** La Nota da aprire: quella di nome `scelta` (un risultato della ricerca) se c'è, se no la prima. */
export function apertaIniziale(note: NotaAperta[], scelta?: string): string {
  return (note.find((n) => n.nome === scelta) ?? note[0]).nome;
}

/**
 * Il nome di una Nota nuova: `nome(n)` (per esempio «Nota 3») con `n` il numero delle Note più uno,
 * o il primo seguente non usato.
 */
export function nomeNuovo(nomi: string[], nome: (n: number) => string): string {
  let n = nomi.length + 1;
  while (nomi.includes(nome(n))) {
    n += 1;
  }
  return nome(n);
}

/**
 * L'errore del backend per il nome `nome`, se lo rifiuterebbe: vuoto, oltre 60 caratteri o tra
 * `altre`, i nomi delle altre Note del Tape. `null` se va bene.
 */
export function nomeRifiutato(nome: string, altre: string[]): AppError | null {
  if (nome.trim() === "" || [...nome].length > NOME_MAX) {
    return { code: "noteNameInvalid", detail: nome };
  }
  return altre.includes(nome) ? { code: "noteNameTaken", detail: nome } : null;
}

/** Le Note della riga dei nomi, in ordine di creazione, e il nome di quella aperta. */
export interface RigaDelleNote {
  aperta: string;
  note: NotaAperta[];
}

/** `riga` con una Nota nuova in fondo, aperta, di nome `nome(n)` («Nota 2», «Nota 3»…). */
export function conNotaNuova(
  riga: RigaDelleNote,
  nome: (n: number) => string
): RigaDelleNote {
  const nuova = nomeNuovo(
    riga.note.map((n) => n.nome),
    nome
  );
  return {
    aperta: nuova,
    note: [...riga.note, { nome: nuova, salvato: null }],
  };
}

/** Perché la Nota `nome` non si può chiamare `nuovo` tra le `note` del Tape; `null` se si può. */
export function rinominaRifiutata(
  note: NotaAperta[],
  nome: string,
  nuovo: string
): AppError | null {
  return nomeRifiutato(
    nuovo,
    note.filter((n) => n.nome !== nome).map((n) => n.nome)
  );
}

/** `riga` con la Nota `nome` chiamata `nuovo`; resta aperta se lo era. */
export function conNome(
  riga: RigaDelleNote,
  nome: string,
  nuovo: string
): RigaDelleNote {
  return {
    aperta: riga.aperta === nome ? nuovo : riga.aperta,
    note: riga.note.map((n) => (n.nome === nome ? { ...n, nome: nuovo } : n)),
  };
}

/** `riga` con `salvato` su disco per la Nota `nome`, e la `firma` della scrittura se c'è. */
export function conTesto(
  riga: RigaDelleNote,
  nome: string,
  salvato: string,
  firma?: Firma
): RigaDelleNote {
  return {
    ...riga,
    note: riga.note.map((n) =>
      n.nome === nome ? { ...n, ...(firma && { firma }), salvato } : n
    ),
  };
}

/**
 * `riga` senza la Nota `nome`. Se era aperta si apre la seguente, o la precedente se era l'ultima;
 * senza più Note torna la Nota nuova con il nome `predefinito`.
 */
export function senzaNota(
  riga: RigaDelleNote,
  nome: string,
  predefinito: string
): RigaDelleNote {
  const i = riga.note.findIndex((n) => n.nome === nome);
  const resto = riga.note.filter((n) => n.nome !== nome);
  if (resto.length === 0) {
    return { aperta: predefinito, note: noteIniziali([], predefinito) };
  }
  const aperta =
    riga.aperta === nome
      ? resto[Math.min(i, resto.length - 1)].nome
      : riga.aperta;
  return { aperta, note: resto };
}

/**
 * Salva con `salva` ogni bozza di `bozze` (per nome): `null` se ora sono tutte su disco, se no il
 * primo motivo per cui una non lo è.
 */
export async function salvaOgni(
  bozze: Readonly<Record<string, string>>,
  salva: (nome: string, bozza: string) => Promise<AppError | null>
): Promise<AppError | null> {
  const esiti = await Promise.all(
    Object.entries(bozze).map(([nome, bozza]) => salva(nome, bozza))
  );
  return esiti.find((e) => e !== null) ?? null;
}

/**
 * Cosa salvare della `bozza` della Nota `nome`, il cui testo su disco è `salvato` (`null` se non
 * c'è): niente se coincide, e una Nota nuova ancora vuota non si scrive.
 */
export function salvataggio(
  nome: string,
  salvato: string | null,
  bozza: string
): Salvataggio | null {
  const atteso = salvato ?? "";
  return bozza === atteso ? null : { atteso, nome, testo: bozza };
}

/**
 * Le Note della Registrazione da mandare al backend, che le tiene nel file accanto all'Ogg: quelle
 * scritte almeno una volta, anche se ora vuote, in ordine. Una Nota nuova mai scritta non c'è.
 */
export function noteInRegistrazione(note: NotaAperta[]): NotaInRegistrazione[] {
  return note.flatMap((n) =>
    n.salvato === null ? [] : [{ nome: n.nome, testo: n.salvato }]
  );
}
