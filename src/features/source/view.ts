import type { OpenedTape, TapeInfo } from "@/bindings";
import { movedPath } from "@/features/source/file-name";
import { isBusy, type Status } from "@/features/status/status";
import {
  type Conversation,
  EMPTY_CONVERSATION,
  type PhraseRef,
} from "@/features/transcription/phrases";

/** Un Tape aperto: le Frasi e le informazioni. */
export interface TapeView {
  conversation: Conversation;
  info: TapeInfo | null;
  path: string;
}

/**
 * La vista della Sorgente: il suo percorso (testo e informazioni stanno nel modulo dell'Attività),
 * il Tape consultato durante un'Attività, con testo e informazioni propri, e cosa c'è al centro.
 */
export interface SourceView {
  browsed: TapeView | null;
  /** La Frase di un risultato della ricerca, evidenziata nel Tape aperto. */
  highlight: PhraseRef | null;
  homeOpen: boolean;
  libraryOpen: boolean;
  /** Il Tape del doppio clic in Esplora file, finché non si può aprire. */
  pending: string | null;
  /** Cresce per riaprire la Sorgente dall'inizio del testo: è nella chiave della sua vista. */
  revision: number;
  source: string | null;
}

export const INITIAL_VIEW: SourceView = {
  browsed: null,
  highlight: null,
  homeOpen: false,
  libraryOpen: false,
  pending: null,
  revision: 0,
  source: null,
};

/** Trasformazioni del testo e delle informazioni di un Tape appena scritto. */
export interface TapeChange {
  change?: (c: Conversation) => Conversation;
  info?: (i: TapeInfo | null) => TapeInfo | null;
}

/**
 * Cosa fa aprire `path`: durante un'Attività si consulta (`browse`), tranne la Sorgente su cui
 * lavora Trascrivi o Riconosci, che riporta alla vista dell'Attività; senza, diventa la Sorgente
 * (`source`), e dalla Libreria (`fromLibrary`) la Sorgente già aperta riparte dall'inizio.
 * `browse` e `source` chiedono alla finestra di leggere il Tape e mandare il risultato.
 */
export type Opening = "activity" | "browse" | "restart" | "source";

export function opening(
  { source }: SourceView,
  status: Status,
  path: string,
  fromLibrary: boolean
): Opening {
  if (isBusy(status)) {
    // Solo Trascrivi lavora sulla Sorgente; durante una Registrazione è un Tape come gli altri.
    const working =
      status.phase === "transcribing" || status.phase === "diarizing";
    return path === source && working ? "activity" : "browse";
  }
  return fromLibrary && path === source ? "restart" : "source";
}

/** Il Tape del doppio clic da aprire ora: nessuna Attività in corso e nessuna conferma aperta. */
export function pendingTape(
  { pending }: SourceView,
  status: Status,
  confirming: boolean
): string | null {
  return isBusy(status) || confirming ? null : pending;
}

export type SourceViewAction =
  /**
   * Il file o il Tape diventa la Sorgente, con la Frase trovata; senza percorso nessuna (il
   * Cestino). `restart`: la sua vista riparte dall'inizio. `keepView`: cambia solo il percorso e
   * la vista mostrata resta (l'Ogg tenuto da una Registrazione).
   */
  | {
      type: "sourceOpened";
      path: string | null;
      tape?: OpenedTape;
      phrase?: PhraseRef;
      restart?: boolean;
      keepView?: boolean;
    }
  | { type: "tapeBrowsed"; path: string; tape: OpenedTape; phrase?: PhraseRef }
  /** Un Tape della barra laterale, della Libreria o della ricerca, di Apri file, del doppio clic o rilasciato. */
  | {
      type: "openRequested";
      path: string;
      phrase?: PhraseRef;
      fromLibrary?: boolean;
    }
  /** "Attività in corso" nella barra laterale. */
  | { type: "activityShown" }
  | { type: "libraryShown" }
  | { type: "homeShown" }
  | { type: "tapeRequested"; path: string }
  | ({ type: "tapeChanged"; path: string } & TapeChange)
  | { type: "tapeMoved"; from: string; to: string }
  | { type: "tapeTrashed"; path: string };

export function sourceView(
  state: SourceView,
  action: SourceViewAction,
  status: Status
): SourceView {
  const { browsed, source } = state;
  switch (action.type) {
    case "sourceOpened":
      if (action.keepView) {
        return { ...state, source: action.path };
      }
      return {
        ...state,
        highlight: action.phrase ?? null,
        homeOpen: false,
        revision: state.revision + (action.restart ? 1 : 0),
        source: action.path,
      };
    case "tapeBrowsed": {
      const { info, parlanti, phrases } = action.tape;
      return {
        ...state,
        browsed: {
          conversation: { ...EMPTY_CONVERSATION, parlanti, phrases },
          info,
          path: action.path,
        },
        highlight: action.phrase ?? null,
        homeOpen: false,
      };
    }
    case "openRequested":
      return requested(state, action, status);
    case "activityShown":
      return {
        ...state,
        browsed: null,
        // La Frase trovata era del Tape consultato.
        highlight: null,
        homeOpen: false,
        libraryOpen: false,
      };
    case "libraryShown":
      return { ...state, libraryOpen: true };
    case "homeShown":
      return { ...state, homeOpen: true, libraryOpen: false };
    case "tapeRequested":
      return { ...state, pending: action.path };
    case "tapeChanged":
      return browsed?.path === action.path
        ? { ...state, browsed: changed(browsed, action) }
        : state;
    case "tapeMoved": {
      const { from, to } = action;
      return {
        ...state,
        browsed: browsed && {
          ...browsed,
          path: movedPath(browsed.path, from, to),
        },
        source: source && movedPath(source, from, to),
      };
    }
    case "tapeTrashed":
      return {
        ...state,
        browsed: browsed?.path === action.path ? null : browsed,
        source: source === action.path ? null : source,
      };
    default:
      return action satisfies never;
  }
}

function requested(
  state: SourceView,
  {
    fromLibrary = false,
    path,
    phrase,
  }: { fromLibrary?: boolean; path: string; phrase?: PhraseRef },
  status: Status
): SourceView {
  const next = { ...state, libraryOpen: false };
  // Il Tape del doppio clic resta in attesa finché non diventa la Sorgente.
  const opened = {
    ...next,
    pending: state.pending === path ? null : state.pending,
  };
  switch (opening(state, status, path, fromLibrary)) {
    case "activity":
      return {
        ...next,
        browsed: null,
        highlight: phrase ?? null,
        homeOpen: false,
      };
    // Il Tape già aperto: torna alla Frase trovata, o all'inizio del testo.
    case "restart":
      return {
        ...opened,
        highlight: phrase ?? null,
        homeOpen: false,
        revision: state.revision + 1,
      };
    case "source":
      return opened;
    default:
      return next;
  }
}

function changed(view: TapeView, { change, info }: TapeChange): TapeView {
  return {
    ...view,
    conversation: change ? change(view.conversation) : view.conversation,
    info: info ? info(view.info) : view.info,
  };
}
