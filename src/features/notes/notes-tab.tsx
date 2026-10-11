import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import { Bold, Check, Italic, Link, List, ListTodo } from "lucide-react";
import {
  type ComponentType,
  type MouseEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands } from "@/bindings";
import { keepFocus, type PlayerState } from "@/features/player/player";
import { type Azione, conTasti } from "@/features/shortcuts/shortcuts";
import { useScorciatoia } from "@/features/shortcuts/shortcuts-provider";
import { applica, ibrido, type Tempi } from "./ibrido";
import { type Formato, inserisciTempo } from "./markdown";
import { type NotaAperta, notaAperta, salvataggio } from "./notes";

/** Pausa nella scrittura dopo cui la Nota si salva da sola. */
const PAUSA_MS = 1000;

/** Il testo da lettura del documento, con i token di DESIGN.md; il cursore come nei Turni. */
const TEMA = EditorView.theme({
  ".cm-content": {
    caretColor: "var(--foreground)",
    minHeight: "22rem",
    padding: "0",
  },
  ".cm-line": { padding: "0" },
  ".cm-placeholder": { color: "var(--muted-foreground)" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: "1.7" },
  "&": { color: "var(--foreground)", fontSize: "1.0625rem" },
  "&.cm-focused": { outline: "none" },
});

/** Un pulsante della barretta del formato, come quelli della barretta del Turno. */
const ICON_BUTTON =
  "flex size-7 items-center justify-center rounded-md text-foreground/80 transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 [&_svg]:size-4";

/** I pulsanti della barretta: formato, icona, nome (`notes.format.*`) e scorciatoia. */
const FORMATI: {
  chiave: string;
  formato: Formato;
  Icona: ComponentType;
  scorciatoia?: Azione;
}[] = [
  {
    chiave: "bold",
    formato: "grassetto",
    Icona: Bold,
    scorciatoia: "grassetto",
  },
  {
    chiave: "italic",
    formato: "corsivo",
    Icona: Italic,
    scorciatoia: "corsivo",
  },
  { chiave: "link", formato: "link", Icona: Link, scorciatoia: "link" },
  { chiave: "list", formato: "elenco", Icona: List },
  { chiave: "task", formato: "daFare", Icona: ListTodo },
];

/** Dove sta la barretta sospesa, sopra la selezione e relativa al campo. */
interface Posizione {
  left: number;
  top: number;
}

/**
 * La scheda Note di un Tape: l'editor ibrido della prima Nota (ADR-0032), Markdown impaginato fuori
 * dalla riga del cursore, con la barretta del formato sulla selezione.
 * In un Tape senza Note il primo carattere crea la Nota con il nome predefinito. Si salva dopo
 * `PAUSA_MS` di pausa, uscendo dal campo e lasciando la scheda. I Tempi nella Nota sono pillole che
 * portano lì il `player`; Ctrl+T scrive al cursore il suo tempo.
 */
export function NotesTab({
  durataMs,
  onError,
  onSaved,
  path,
  readOnly,
  player,
  scelta,
}: {
  /** La durata del Tape: un tempo da lì in poi resta testo. */
  durataMs: number;
  onError: (error: AppError) => void;
  /** Il Tape `path` riscritto, con la sua nuova data di modifica. */
  onSaved: (path: string, modificato: number | null) => void;
  path: string;
  /** Manca durante la Registrazione e mentre si trascrive il Tape: niente salti. */
  player?: Pick<PlayerState, "move" | "positionMs">;
  readOnly: boolean;
  /** La Nota da aprire, per nome: quella di un risultato della ricerca. */
  scelta?: string;
}) {
  const { t } = useTranslation();
  const [nota, setNota] = useState<NotaAperta | null>(null);
  const [salvate, setSalvate] = useState(false);
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const sola = useRef(new Compartment());
  // Il testo su disco e i salvataggi, uno dopo l'altro: ognuno parte dal testo dell'ultimo.
  const salvato = useRef<string | null>(null);
  const coda = useRef<Promise<void>>(Promise.resolve());
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const [barretta, setBarretta] = useState<Posizione | null>(null);
  const formatta = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    const formato = e.currentTarget.dataset.formato as Formato;
    if (view.current) {
      applica(view.current, formato);
    }
  }, []);

  useEffect(() => {
    let attuale = true;
    commands.tapeNote(path).then((result) => {
      if (!attuale) {
        return;
      }
      if (result.status === "error") {
        onError(result.error);
        return;
      }
      const aperta = notaAperta(result.data, t("notes.defaultName"), scelta);
      salvato.current = aperta.salvato;
      setSalvate(aperta.salvato !== null);
      setNota(aperta);
    });
    return () => {
      attuale = false;
    };
  }, [onError, path, scelta, t]);

  const salva = useCallback(() => {
    clearTimeout(timer.current);
    const nome = nota?.nome;
    const editor = view.current;
    if (nome === undefined || !editor) {
      return;
    }
    const bozza = editor.state.doc.toString();
    coda.current = coda.current.then(async () => {
      const da = salvataggio(nome, salvato.current, bozza);
      if (!da) {
        return;
      }
      const result = await commands.saveNota(path, nome, da.testo, da.atteso);
      if (result.status === "error") {
        onError(result.error);
        return;
      }
      salvato.current = da.testo;
      onSaved(path, result.data);
      setSalvate(view.current?.state.doc.toString() === da.testo);
    });
  }, [nota?.nome, onError, onSaved, path]);

  // L'editor nasce con la Nota letta e vive quanto la scheda: uscendo salva la bozza.
  const salvaRef = useRef(salva);
  salvaRef.current = salva;
  // Le pillole leggono durata, player e lingua quando si ridisegnano.
  const attuali = useRef({ durataMs, player, t });
  attuali.current = { durataMs, player, t };
  const tempi = useRef<Tempi>({
    durataMs: () => attuali.current.durataMs,
    etichetta: (time) => attuali.current.t("notes.jump", { time }),
    salta: (ms) => attuali.current.player?.move(ms, "jump"),
  });
  useScorciatoia(
    "tempoNota",
    readOnly || !player
      ? null
      : () => {
          const editor = view.current;
          if (editor) {
            editor.dispatch({
              ...inserisciTempo(editor.state, player.positionMs),
              scrollIntoView: true,
              userEvent: "input.time",
            });
            editor.focus();
          }
        }
  );
  const testo = nota?.salvato;
  const caricata = nota !== null;
  // biome-ignore lint/correctness/useExhaustiveDependencies: testo e readOnly iniziali; readOnly cambia con il Compartment
  useEffect(() => {
    if (!(caricata && host.current)) {
      return;
    }
    const editor = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: testo ?? "",
        extensions: [
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          EditorView.lineWrapping,
          EditorView.contentAttributes.of({
            "aria-label": t("notes.label"),
            spellcheck: "false",
          }),
          placeholder(t("notes.placeholder")),
          sola.current.of(soloLettura(readOnly)),
          TEMA,
          ibrido(
            (url) =>
              commands.openLink(url).then((result) => {
                if (result.status === "error") {
                  onError(result.error);
                }
              }),
            tempi.current
          ),
          EditorView.updateListener.of((update) => {
            if (
              update.selectionSet ||
              update.focusChanged ||
              update.docChanged ||
              update.geometryChanged
            ) {
              misuraBarretta(update.view, host.current, setBarretta);
            }
            if (update.docChanged) {
              setSalvate(false);
              clearTimeout(timer.current);
              timer.current = setTimeout(() => salvaRef.current(), PAUSA_MS);
            }
          }),
          EditorView.domEventHandlers({ blur: () => salvaRef.current() }),
        ],
      }),
    });
    view.current = editor;
    return () => {
      salvaRef.current();
      view.current = null;
      editor.destroy();
    };
  }, [caricata]);

  useEffect(() => {
    view.current?.dispatch({
      effects: sola.current.reconfigure(soloLettura(readOnly)),
    });
  }, [readOnly]);

  return (
    <div className="flex flex-col">
      <p className="flex h-6 items-center gap-1 pb-3.5 text-muted-foreground text-xs">
        {salvate ? (
          <>
            <Check className="size-3.5 text-play" />
            {t("notes.saved")}
          </>
        ) : null}
      </p>
      {/* Il campo è di CodeMirror: la barretta gli sta accanto, non dentro. */}
      <div className="relative">
        <div className="select-text" ref={host} />
        {barretta ? (
          <div
            aria-label={t("notes.format.label")}
            className="absolute z-10 flex -translate-y-full items-center gap-0.5 rounded-lg border bg-card p-0.5 shadow-float"
            role="toolbar"
            style={{ left: barretta.left, top: barretta.top }}
          >
            {FORMATI.map(({ chiave, formato, Icona, scorciatoia }) => {
              const nome = t(`notes.format.${chiave}`);
              const titolo = scorciatoia
                ? conTasti(t, nome, scorciatoia)
                : nome;
              return (
                <span className="contents" key={formato}>
                  {formato === "elenco" ? (
                    <span aria-hidden className="mx-0.5 h-4 w-px bg-border" />
                  ) : null}
                  <button
                    aria-label={titolo}
                    className={ICON_BUTTON}
                    data-formato={formato}
                    onClick={formatta}
                    onPointerDown={keepFocus}
                    title={titolo}
                    type="button"
                  >
                    <Icona />
                  </button>
                </span>
              );
            })}
          </div>
        ) : null}
      </div>
    </div>
  );
}

/**
 * Mette la barretta del formato sopra l'inizio della selezione, se c'è una selezione nel campo
 * con il focus e si può scrivere; altrimenti la toglie. Legge la geometria quando CodeMirror lo
 * permette.
 */
function misuraBarretta(
  view: EditorView,
  campo: HTMLElement | null,
  imposta: (posizione: Posizione | null) => void
) {
  view.requestMeasure({
    read: (v): Posizione | null => {
      const { main } = v.state.selection;
      const inizio =
        v.hasFocus && !main.empty && !v.state.readOnly && campo
          ? v.coordsAtPos(main.from)
          : null;
      if (!(inizio && campo)) {
        return null;
      }
      const box = campo.getBoundingClientRect();
      return {
        left: Math.max(0, inizio.left - box.left),
        top: inizio.top - box.top - 6,
      };
    },
    write: imposta,
  });
}

function soloLettura(readOnly: boolean) {
  return [EditorState.readOnly.of(readOnly), EditorView.editable.of(!readOnly)];
}
