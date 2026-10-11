import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import { Check } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands } from "@/bindings";
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

/**
 * La scheda Note di un Tape: un editor CodeMirror del Markdown grezzo della prima Nota (ADR-0032).
 * In un Tape senza Note il primo carattere crea la Nota con il nome predefinito. Si salva dopo
 * `PAUSA_MS` di pausa, uscendo dal campo e lasciando la scheda.
 */
export function NotesTab({
  onError,
  onSaved,
  path,
  readOnly,
  scelta,
}: {
  onError: (error: AppError) => void;
  /** Il Tape `path` riscritto, con la sua nuova data di modifica. */
  onSaved: (path: string, modificato: number | null) => void;
  path: string;
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
          EditorView.updateListener.of((update) => {
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
      <div className="select-text" ref={host} />
    </div>
  );
}

function soloLettura(readOnly: boolean) {
  return [EditorState.readOnly.of(readOnly), EditorView.editable.of(!readOnly)];
}
