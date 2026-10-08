import type { OpenedTape, TapeInfo } from "@/bindings";
import { movedPath } from "@/features/source/file-name";
import {
  type Conversation,
  EMPTY_CONVERSATION,
} from "@/features/transcription/phrases";

/** Un Tape aperto: le Frasi e le informazioni. */
export interface TapeView {
  conversation: Conversation;
  info: TapeInfo | null;
  path: string;
}

/**
 * La vista della Sorgente: il suo percorso (testo e informazioni stanno nel modulo dell'Attività)
 * e il Tape consultato durante un'Attività, con testo e informazioni propri.
 */
export interface SourceView {
  browsed: TapeView | null;
  source: string | null;
}

export const INITIAL_VIEW: SourceView = { browsed: null, source: null };

/** Trasformazioni del testo e delle informazioni di un Tape appena scritto. */
export interface TapeChange {
  change?: (c: Conversation) => Conversation;
  info?: (i: TapeInfo | null) => TapeInfo | null;
}

export type SourceViewAction =
  /** Il file o il Tape diventa la Sorgente; senza percorso nessuna (il Cestino). */
  | { type: "sourceOpened"; path: string | null; tape?: OpenedTape }
  | { type: "tapeBrowsed"; path: string; tape: OpenedTape }
  | { type: "browsingClosed" }
  | ({ type: "tapeChanged"; path: string } & TapeChange)
  | { type: "tapeMoved"; from: string; to: string }
  | { type: "tapeTrashed"; path: string };

export function sourceView(
  state: SourceView,
  action: SourceViewAction
): SourceView {
  const { browsed, source } = state;
  switch (action.type) {
    case "sourceOpened":
      return { ...state, source: action.path };
    case "tapeBrowsed": {
      const { info, parlanti, phrases } = action.tape;
      return {
        ...state,
        browsed: {
          conversation: { ...EMPTY_CONVERSATION, parlanti, phrases },
          info,
          path: action.path,
        },
      };
    }
    case "browsingClosed":
      return { ...state, browsed: null };
    case "tapeChanged":
      return browsed?.path === action.path
        ? { ...state, browsed: changed(browsed, action) }
        : state;
    case "tapeMoved": {
      const { from, to } = action;
      return {
        browsed: browsed && {
          ...browsed,
          path: movedPath(browsed.path, from, to),
        },
        source: source && movedPath(source, from, to),
      };
    }
    case "tapeTrashed":
      return {
        browsed: browsed?.path === action.path ? null : browsed,
        source: source === action.path ? null : source,
      };
    default:
      return action satisfies never;
  }
}

function changed(view: TapeView, { change, info }: TapeChange): TapeView {
  return {
    ...view,
    conversation: change ? change(view.conversation) : view.conversation,
    info: info ? info(view.info) : view.info,
  };
}
