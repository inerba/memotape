import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import {
  Bold,
  Check,
  CircleAlert,
  Copy,
  Ellipsis,
  Italic,
  Link,
  List,
  ListTodo,
  Pencil,
  Plus,
  Trash2,
} from "lucide-react";
import {
  type ChangeEvent,
  type ComponentType,
  type KeyboardEvent,
  type MouseEvent,
  type RefObject,
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands } from "@/bindings";
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
import { Switch } from "@/components/ui/switch";
import { MenuItem } from "@/features/library/tape-actions";
import { keepFocus, type PlayerState } from "@/features/player/player";
import { SWITCH_CLASS } from "@/features/settings/setting-switch";
import { useSettings } from "@/features/settings/settings-context";
import {
  type Azione,
  ariaTasti,
  conTasti,
} from "@/features/shortcuts/shortcuts";
import { useScorciatoia } from "@/features/shortcuts/shortcuts-provider";
import { errorText } from "@/features/status/status";
import { applica, ibrido, type Tempi } from "./ibrido";
import { type Formato, inserisciTempo } from "./markdown";
import {
  type Firma,
  type NotaAperta,
  nomeRifiutato,
  oraDellaFirma,
} from "./notes";
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

/** Il tempo di Ctrl+T e, se c'è `move`, il salto delle pillole. */
type Ascolto = Pick<PlayerState, "positionMs"> &
  Partial<Pick<PlayerState, "move">>;

interface NotesTabProps {
  /**
   * Le Note della Registrazione: si salvano sempre da sole, senza Salva né interruttore, e
   * «Salvate» dice che entrano nel Tape allo Stop.
   */
  bozza?: boolean;
  /** La durata del Tape: un tempo da lì in poi resta testo. */
  durataMs: number;
  onError: (error: AppError) => void;
  /** Un avviso, come «Nota copiata». */
  onMessage: (text: string) => void;
  /**
   * Manca mentre si trascrive il Tape: niente salti né Ctrl+T. Durante la Registrazione ha solo il
   * suo tempo, pause escluse.
   */
  player?: Ascolto;
  readOnly: boolean;
  /** Cresce con Non salvare: l'editor riparte dal testo su disco. */
  scarti: number;
}

/**
 * La scheda Note di un Tape (ADR-0032): la riga dei nomi delle Note, con «+ Nuova nota» e, a
 * destra, Salva e l'interruttore Salva da solo; sotto la Nota aperta nell'editor ibrido. Doppio
 * clic sul nome o F2 rinomina; il «…» rinomina, copia ed elimina. Senza Salva da solo una Nota con
 * modifiche ha un pallino. Sotto la riga, chi ha scritto la Nota l'ultima volta e quando; se un
 * Assistente l'ha cambiata mentre aveva modifiche, un avviso che resta offre «Tieni le mie» e «Usa
 * le sue». In sola lettura (un'Attività sul Tape) si leggono e si copiano. Vuota finché le Note
 * non sono lette.
 */
export function NotesTab({
  note,
  ...props
}: NotesTabProps & { note: NoteDelTape | null }) {
  return note ? <NoteLette note={note} {...props} /> : null;
}

function NoteLette({
  bozza,
  durataMs,
  note,
  onError,
  onMessage,
  player,
  readOnly,
  scarti,
}: NotesTabProps & { note: NoteDelTape }) {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  // Le Note della Registrazione si salvano sempre da sole.
  const salvaDaSolo = bozza || (settings.noteSalvaDaSolo ?? true);
  const interruttore = useId();
  const [rinominando, setRinominando] = useState<string | null>(null);
  const [eliminando, setEliminando] = useState<string | null>(null);
  const view = useRef<EditorView | null>(null);
  const { aperta, bozze, crea, elimina, rinomina, salvaTutte } = note;
  const nota = note.note.find((n) => n.nome === aperta) ?? note.note[0];
  const modificate = Object.keys(bozze).length > 0;
  const salvate = salvaDaSolo && !modificate && nota.salvato !== null;
  // La Nota aperta cambiata da un Assistente mentre aveva modifiche: l'utente sceglie.
  const sua = nota.nome in note.conflitti;
  const tieniLeMie = useCallback(
    () => note.tieniLeMie(nota.nome),
    [note, nota.nome]
  );
  const usaLeSue = useCallback(
    () => note.usaLeSue(nota.nome),
    [note, nota.nome]
  );
  useScorciatoia("salvaNote", readOnly ? null : salvaTutte);
  const salvaSubito = useCallback(() => salvaTutte(), [salvaTutte]);
  // Accesa, le bozze in sospeso si salvano subito.
  const cambiaSalvaDaSolo = useCallback(
    async (acceso: boolean) => {
      const error = await save(
        (current) => ({ ...current, noteSalvaDaSolo: acceso }),
        "noteSalvaDaSolo"
      );
      if (error) {
        onError(error);
      } else if (acceso) {
        salvaTutte();
      }
    },
    [onError, salvaTutte, save]
  );

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
                modificata={!salvaDaSolo && n.nome in bozze}
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
        <div className="ml-auto flex items-center gap-3 text-muted-foreground text-xs">
          {salvaDaSolo ? null : (
            <Button
              aria-keyshortcuts={ariaTasti("salvaNote")}
              className="h-7 bg-card"
              disabled={readOnly || !modificate}
              onClick={salvaSubito}
              size="sm"
              title={conTasti(t, t("notes.save"), "salvaNote")}
              variant="outline"
            >
              {t("notes.save")}
            </Button>
          )}
          {salvate ? (
            <span className="flex items-center gap-1">
              <Check className="size-3.5 text-play" />
              {t(bozza ? "notes.savedDraft" : "notes.saved")}
            </span>
          ) : null}
          {bozza ? null : (
            <label
              className="flex cursor-pointer items-center gap-2 whitespace-nowrap"
              htmlFor={interruttore}
            >
              <Switch
                checked={salvaDaSolo}
                className={SWITCH_CLASS}
                id={interruttore}
                onCheckedChange={cambiaSalvaDaSolo}
              />
              {t("notes.autosave")}
            </label>
          )}
        </div>
      </div>
      <div className="flex h-8 items-center gap-1 pb-3.5">
        {nota.firma ? (
          <span className="truncate text-muted-foreground text-xs">
            <FirmaNota firma={nota.firma} />
          </span>
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
      {sua ? (
        <div
          className="mb-4 flex flex-wrap items-center gap-3 rounded-xl border bg-card px-4 py-3 text-sm"
          role="alert"
        >
          <CircleAlert className="size-4 shrink-0 text-destructive" />
          <span className="min-w-0 flex-1">
            {t("notes.conflict.text", { nome: nota.nome })}
          </span>
          <Button onClick={tieniLeMie} size="sm" variant="outline">
            {t("notes.conflict.keep")}
          </Button>
          <Button onClick={usaLeSue} size="sm" variant="outline">
            {t("notes.conflict.theirs")}
          </Button>
        </div>
      ) : null}
      <NotaEditor
        bozza={bozze[nota.nome]}
        durataMs={durataMs}
        key={`${nota.nome}:${scarti}:${note.versione}`}
        nota={nota}
        onError={onError}
        player={player}
        readOnly={readOnly}
        salva={note.salva}
        salvaDaSolo={salvaDaSolo}
        scrivi={note.scrivi}
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

/** Chi ha scritto la Nota l'ultima volta e quando: «Assistente (Claude) · 17:52», «Tu · 18:03». */
function FirmaNota({ firma: { autore, modificata } }: { firma: Firma }) {
  const { i18n, t } = useTranslation();
  const time = oraDellaFirma(modificata, new Date(), i18n.language);
  if (autore.tipo === "utente") {
    return t("notes.by.user", { time });
  }
  return autore.client
    ? t("notes.by.assistantClient", { client: autore.client, time })
    : t("notes.by.assistant", { time });
}

/**
 * Il nome di una Nota nella riga: un clic la apre, doppio clic o F2 la rinominano. `modificata`:
 * il pallino delle modifiche non salvate.
 */
function NomeNota({
  modificata,
  nome,
  onOpen,
  onRename,
  selected,
}: {
  modificata: boolean;
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
  const { t } = useTranslation();
  return (
    <button
      aria-selected={selected}
      className="flex h-[30px] max-w-[22ch] items-center gap-1.5 rounded-lg px-2.5 font-medium text-muted-foreground text-sm transition-colors hover:bg-accent/60 hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-selected:bg-accent aria-selected:text-foreground"
      onClick={open}
      onDoubleClick={rename}
      onKeyDown={keyDown}
      role="tab"
      title={nome}
      type="button"
    >
      <span className="truncate">{nome}</span>
      {modificata ? (
        <span
          aria-label={t("notes.unsaved")}
          className="size-1.5 shrink-0 rounded-full bg-foreground"
          role="img"
          title={t("notes.unsaved")}
        />
      ) : null}
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
 * L'editor ibrido della Nota `nota` (ADR-0032), Markdown impaginato fuori dalla riga del cursore,
 * con la barretta del formato sulla selezione; nasce con la `bozza`, se c'è, o con il testo su
 * disco. Ogni modifica va a `scrivi`; con `salvaDaSolo` si salva dopo `PAUSA_MS` di pausa, uscendo
 * dal campo e lasciando la Nota o la scheda. I Tempi nella Nota sono pillole che portano lì il
 * `player`; Ctrl+T scrive al cursore il suo tempo.
 */
function NotaEditor({
  bozza,
  durataMs,
  nota,
  onError,
  player,
  readOnly,
  salva,
  salvaDaSolo,
  scrivi,
  view,
}: {
  bozza?: string;
  durataMs: number;
  nota: NotaAperta;
  onError: (error: AppError) => void;
  player?: Ascolto;
  readOnly: boolean;
  salva: (nome: string, bozza: string) => Promise<boolean>;
  salvaDaSolo: boolean;
  scrivi: (nome: string, testo: string) => void;
  view: RefObject<EditorView | null>;
}) {
  const { t } = useTranslation();
  const host = useRef<HTMLDivElement>(null);
  const sola = useRef(new Compartment());
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const salvaOra = useRef<() => void>(() => undefined);
  const { nome, salvato } = nota;
  const [barretta, setBarretta] = useState<Posizione | null>(null);
  const formatta = useCallback(
    (e: MouseEvent<HTMLButtonElement>) => {
      const formato = e.currentTarget.dataset.formato as Formato;
      if (view.current) {
        applica(view.current, formato);
      }
    },
    [view]
  );

  // Le pillole leggono durata, player e lingua quando si ridisegnano; l'editor Salva da solo.
  const attuali = useRef({ durataMs, player, salvaDaSolo, t });
  attuali.current = { durataMs, player, salvaDaSolo, t };
  const tempi = useRef<Tempi>({
    durataMs: () => attuali.current.durataMs,
    etichetta: (time) => attuali.current.t("notes.jump", { time }),
    salta: (ms) => attuali.current.player?.move?.(ms, "jump"),
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

  // L'editor vive quanto la Nota aperta: con Salva da solo uscendo salva la bozza.
  // biome-ignore lint/correctness/useExhaustiveDependencies: testo e readOnly iniziali; readOnly cambia con il Compartment
  useEffect(() => {
    if (!host.current) {
      return;
    }
    const salvaBozza = () => {
      clearTimeout(timer.current);
      if (attuali.current.salvaDaSolo) {
        salva(nome, editor.state.doc.toString());
      }
    };
    const editor = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: bozza ?? salvato ?? "",
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
              scrivi(nome, update.state.doc.toString());
              clearTimeout(timer.current);
              timer.current = setTimeout(salvaBozza, PAUSA_MS);
            }
          }),
          EditorView.domEventHandlers({ blur: salvaBozza }),
        ],
      }),
    });
    view.current = editor;
    salvaOra.current = salvaBozza;
    return () => {
      salvaBozza();
      if (view.current === editor) {
        view.current = null;
      }
      editor.destroy();
    };
  }, [nome]);

  // Diventando di sola lettura (lo Stop della Registrazione) salva subito, senza aspettare la pausa.
  useEffect(() => {
    view.current?.dispatch({
      effects: sola.current.reconfigure(soloLettura(readOnly)),
    });
    if (readOnly) {
      salvaOra.current();
    }
  }, [readOnly, view]);

  return (
    // Il campo è di CodeMirror: la barretta gli sta accanto, non dentro.
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
            const etichetta = t(`notes.format.${chiave}`);
            const titolo = scorciatoia
              ? conTasti(t, etichetta, scorciatoia)
              : etichetta;
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
