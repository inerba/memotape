import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import { Check, Copy, Ellipsis, Pencil, Plus, Trash2 } from "lucide-react";
import {
  type ChangeEvent,
  type KeyboardEvent,
  type RefObject,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { closeMenu, PopoverMenu } from "@/components/popover-menu";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { MenuItem } from "@/features/library/tape-actions";
import { errorText } from "@/features/status/status";
import { type NotaAperta, nomeRifiutato } from "./notes";
import type { NoteDelTape } from "./use-note";

/** Pausa nella scrittura dopo cui la Nota si salva da sola. */
const PAUSA_MS = 1000;

/** Il menu «…» della Nota aperta. */
const MENU = "nota";

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
 * La scheda Note di un Tape (ADR-0032): la riga dei nomi delle Note, con «+ Nuova nota», e sotto la
 * Nota aperta in un editor CodeMirror del Markdown grezzo. Doppio clic sul nome o F2 rinomina; il
 * «…» rinomina, copia ed elimina. In sola lettura (un'Attività sul Tape) si leggono e si copiano.
 * Vuota finché le Note non sono lette.
 */
export function NotesTab({
  note,
  ...props
}: {
  note: NoteDelTape | null;
  /** Un avviso, come «Nota copiata». */
  onMessage: (text: string) => void;
  readOnly: boolean;
}) {
  return note ? <NoteLette note={note} {...props} /> : null;
}

function NoteLette({
  note,
  onMessage,
  readOnly,
}: {
  note: NoteDelTape;
  onMessage: (text: string) => void;
  readOnly: boolean;
}) {
  const { t } = useTranslation();
  const [rinominando, setRinominando] = useState<string | null>(null);
  const [eliminando, setEliminando] = useState<string | null>(null);
  const [salvate, setSalvate] = useState(false);
  const view = useRef<EditorView | null>(null);
  const { aperta, crea, elimina, rinomina } = note;
  const nota = note.note.find((n) => n.nome === aperta) ?? note.note[0];

  const nuova = useCallback(() => setRinominando(crea()), [crea]);
  const fineRinomina = useCallback(() => setRinominando(null), []);
  const conferma = useCallback(
    (nome: string, nuovo: string) => {
      setRinominando(null);
      rinomina(nome, nuovo);
    },
    [rinomina]
  );
  const rinominaAperta = useCallback(() => {
    closeMenu(MENU);
    setRinominando(nota.nome);
  }, [nota.nome]);
  const copia = useCallback(async () => {
    closeMenu(MENU);
    await navigator.clipboard.writeText(
      view.current?.state.doc.toString() ?? ""
    );
    onMessage(t("notes.copied", { nome: nota.nome }));
  }, [nota.nome, onMessage, t]);
  const chiediElimina = useCallback(() => {
    closeMenu(MENU);
    setEliminando(nota.nome);
  }, [nota.nome]);
  const chiudiElimina = useCallback((open: boolean) => {
    if (!open) {
      setEliminando(null);
    }
  }, []);
  const eliminaNota = useCallback(() => {
    if (eliminando !== null) {
      elimina(eliminando);
    }
  }, [elimina, eliminando]);

  return (
    <div className="flex flex-col">
      <div className="flex flex-wrap items-center gap-1 pt-1 pb-1.5">
        <div
          aria-label={t("notes.list")}
          className="flex flex-wrap items-center gap-1"
          role="tablist"
        >
          {note.note.map((n) =>
            n.nome === rinominando ? (
              <NomeInModifica
                altre={note.note.filter((o) => o !== n).map((o) => o.nome)}
                key={n.nome}
                nome={n.nome}
                onCancel={fineRinomina}
                onSubmit={conferma}
              />
            ) : (
              <NomeNota
                key={n.nome}
                nome={n.nome}
                onOpen={note.apri}
                onRename={readOnly ? undefined : setRinominando}
                selected={n.nome === nota.nome}
              />
            )
          )}
        </div>
        {readOnly ? null : (
          <Button
            className="text-muted-foreground"
            onClick={nuova}
            size="sm"
            variant="ghost"
          >
            <Plus />
            {t("notes.new")}
          </Button>
        )}
      </div>
      <div className="flex h-8 items-center gap-1 pb-3.5 text-muted-foreground text-xs">
        {salvate ? (
          <>
            <Check className="size-3.5 text-play" />
            {t("notes.saved")}
          </>
        ) : null}
        <PopoverMenu
          className="ml-auto size-7"
          icon={<Ellipsis />}
          id={MENU}
          label={t("notes.more", { nome: nota.nome })}
          variant="ghost"
        >
          <MenuItem
            disabled={readOnly}
            icon={<Pencil />}
            kbd="F2"
            onClick={rinominaAperta}
          >
            {t("notes.rename")}
          </MenuItem>
          <MenuItem icon={<Copy />} onClick={copia}>
            {t("notes.copy")}
          </MenuItem>
          <hr className="my-1" />
          <MenuItem
            className="text-destructive [&_svg]:text-destructive"
            disabled={readOnly}
            icon={<Trash2 />}
            onClick={chiediElimina}
          >
            {t("notes.delete.action")}
          </MenuItem>
        </PopoverMenu>
      </div>
      <NotaEditor
        key={nota.nome}
        nota={nota}
        onSalvate={setSalvate}
        readOnly={readOnly}
        salva={note.salva}
        view={view}
      />
      <AlertDialog onOpenChange={chiudiElimina} open={eliminando !== null}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t("notes.delete.title", { nome: eliminando })}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t("notes.delete.description")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("notes.delete.keep")}</AlertDialogCancel>
            <AlertDialogAction onClick={eliminaNota}>
              {t("notes.delete.confirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

/** Il nome di una Nota nella riga: un clic la apre, doppio clic o F2 la rinominano. */
function NomeNota({
  nome,
  onOpen,
  onRename,
  selected,
}: {
  nome: string;
  onOpen: (nome: string) => void;
  onRename?: (nome: string) => void;
  selected: boolean;
}) {
  const open = useCallback(() => onOpen(nome), [nome, onOpen]);
  const rename = useCallback(() => onRename?.(nome), [nome, onRename]);
  const keyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "F2" && onRename) {
        e.preventDefault();
        onRename(nome);
      }
    },
    [nome, onRename]
  );
  return (
    <button
      aria-selected={selected}
      className="h-[30px] max-w-[22ch] truncate rounded-lg px-2.5 font-medium text-muted-foreground text-sm transition-colors hover:bg-accent/60 hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-selected:bg-accent aria-selected:text-foreground"
      onClick={open}
      onDoubleClick={rename}
      onKeyDown={keyDown}
      role="tab"
      title={nome}
      type="button"
    >
      {nome}
    </button>
  );
}

/**
 * Il nome di una Nota rinominato sul posto: Invio conferma un nome valido, Esc o l'uscita dal campo
 * annullano. Sotto, perché il nome non va.
 */
function NomeInModifica({
  altre,
  nome,
  onCancel,
  onSubmit,
}: {
  /** I nomi delle altre Note del Tape. */
  altre: string[];
  nome: string;
  onCancel: () => void;
  onSubmit: (nome: string, nuovo: string) => void;
}) {
  const { t } = useTranslation();
  const [valore, setValore] = useState(nome);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    input.current?.focus();
    input.current?.select();
  }, []);
  const nuovo = valore.trim();
  const rifiuto = nuovo === nome ? null : nomeRifiutato(nuovo, altre);
  const change = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => setValore(e.target.value),
    []
  );
  const keyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter" && !rifiuto) {
        onSubmit(nome, nuovo);
      } else if (e.key === "Escape") {
        onCancel();
      }
    },
    [nome, nuovo, onCancel, onSubmit, rifiuto]
  );
  return (
    <span className="relative">
      <Input
        aria-invalid={rifiuto !== null}
        aria-label={t("notes.renameLabel")}
        className="h-[30px] w-[16ch]"
        onBlur={onCancel}
        onChange={change}
        onKeyDown={keyDown}
        ref={input}
        value={valore}
      />
      {rifiuto ? (
        <span
          className="absolute top-full left-0 z-10 mt-1 whitespace-nowrap text-destructive text-xs"
          role="alert"
        >
          {errorText(rifiuto, t)}
        </span>
      ) : null}
    </span>
  );
}

/**
 * L'editor della Nota `nota`, che nasce con il suo testo su disco. Si salva dopo `PAUSA_MS` di
 * pausa, uscendo dal campo e lasciando la Nota o la scheda.
 */
function NotaEditor({
  nota,
  onSalvate,
  readOnly,
  salva,
  view,
}: {
  nota: NotaAperta;
  /** Se su disco c'è il testo dell'editor. */
  onSalvate: (salvate: boolean) => void;
  readOnly: boolean;
  salva: (nome: string, bozza: string) => Promise<boolean>;
  view: RefObject<EditorView | null>;
}) {
  const { t } = useTranslation();
  const host = useRef<HTMLDivElement>(null);
  const sola = useRef(new Compartment());
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const { nome, salvato } = nota;

  // L'editor vive quanto la Nota aperta: uscendo salva la bozza.
  // biome-ignore lint/correctness/useExhaustiveDependencies: testo e readOnly iniziali; readOnly cambia con il Compartment
  useEffect(() => {
    if (!host.current) {
      return;
    }
    const salvaBozza = () => {
      clearTimeout(timer.current);
      const bozza = editor.state.doc.toString();
      salva(nome, bozza).then((ok) => {
        // Lasciata la Nota, l'esito non riguarda più l'editor mostrato.
        if (view.current === editor) {
          onSalvate(ok && editor.state.doc.toString() === bozza);
        }
      });
    };
    const editor = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: salvato ?? "",
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
              onSalvate(false);
              clearTimeout(timer.current);
              timer.current = setTimeout(salvaBozza, PAUSA_MS);
            }
          }),
          EditorView.domEventHandlers({ blur: salvaBozza }),
        ],
      }),
    });
    view.current = editor;
    onSalvate(salvato !== null);
    return () => {
      salvaBozza();
      if (view.current === editor) {
        view.current = null;
      }
      editor.destroy();
    };
  }, [nome]);

  useEffect(() => {
    view.current?.dispatch({
      effects: sola.current.reconfigure(soloLettura(readOnly)),
    });
  }, [readOnly, view]);

  return <div className="select-text" ref={host} />;
}

function soloLettura(readOnly: boolean) {
  return [EditorState.readOnly.of(readOnly), EditorView.editable.of(!readOnly)];
}
