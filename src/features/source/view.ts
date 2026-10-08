import type { OpenedTape, TapeInfo } from "@/bindings";
import { movedPath } from "@/features/source/file-name";
import {
  isBusy,
  type Status,
  transcribesSource,
} from "@/features/status/status";
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
  consulted: TapeView | null;
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
  consulted: null,
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
 * Cosa fa aprire `path`: durante un'Attività si consulta (`consult`), tranne la Sorgente su cui
 * lavora Trascrivi o Riconosci, che riporta alla vista dell'Attività; senza, diventa la Sorgente
 * (`source`), e dalla Libreria (`fromLibrary`) la Sorgente già aperta riparte dall'inizio.
 * `consult` e `source` chiedono alla finestra di leggere il Tape e mandare il risultato.
 * La finestra lo decide una volta e lo passa a `openRequested`.
 */
export type Opening = "activity" | "consult" | "restart" | "source";

export function opening(
  { source }: SourceView,
  status: Status,
  path: string,
  fromLibrary: boolean
): Opening {
  if (isBusy(status)) {
    // Durante una Registrazione la Sorgente è un Tape come gli altri.
    return path === source && transcribesSource(status)
      ? "activity"
      : "consult";
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
  | {
      type: "tapeConsulted";
      path: string;
      tape: OpenedTape;
      phrase?: PhraseRef;
    }
  /** Un Tape della barra laterale, della Libreria o della ricerca, di Apri file, del doppio clic o rilasciato. */
  | {
      type: "openRequested";
      path: string;
      phrase?: PhraseRef;
      opening: Opening;
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
  action: SourceViewAction
): SourceView {
  const { consulted, source } = state;
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
    case "tapeConsulted": {
      const { info, parlanti, phrases } = action.tape;
      return {
        ...state,
        consulted: {
          conversation: { ...EMPTY_CONVERSATION, parlanti, phrases },
          info,
          path: action.path,
        },
        highlight: action.phrase ?? null,
        homeOpen: false,
      };
    }
    case "openRequested":
      return requested(state, action);
    case "activityShown":
      return {
        ...state,
        consulted: null,
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
      return consulted?.path === action.path
        ? { ...state, consulted: changed(consulted, action) }
        : state;
    case "tapeMoved": {
      const { from, to } = action;
      return {
        ...state,
        consulted: consulted && {
          ...consulted,
          path: movedPath(consulted.path, from, to),
        },
        source: source && movedPath(source, from, to),
      };
    }
    case "tapeTrashed":
      return {
        ...state,
        consulted: consulted?.path === action.path ? null : consulted,
        source: source === action.path ? null : source,
      };
    default:
      return action satisfies never;
  }
}

function requested(
  state: SourceView,
  {
    opening: how,
    path,
    phrase,
  }: Extract<SourceViewAction, { type: "openRequested" }>
): SourceView {
  const next = { ...state, libraryOpen: false };
  // Il Tape del doppio clic resta in attesa finché non diventa la Sorgente.
  const opened = {
    ...next,
    pending: state.pending === path ? null : state.pending,
  };
  switch (how) {
    case "activity":
      return {
        ...next,
        consulted: null,
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
