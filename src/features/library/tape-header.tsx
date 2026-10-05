import type { TFunction } from "i18next";
import {
  AlertTriangle,
  AudioLines,
  FileAudio,
  Languages,
  Users,
} from "lucide-react";
import {
  type FocusEvent,
  type KeyboardEvent,
  type ReactNode,
  useCallback,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import type { LibraryList, TapeInfo } from "@/bindings";
import { dateTimeInput, dayText } from "@/features/library/library";
import { NameInput } from "@/features/library/name-input";
import { elapsedText } from "@/features/recording/recording";
import { speechLanguageName } from "@/features/settings/settings";
import { fileName, folderOf } from "@/features/source/file-name";

const EXTENSION = /\.tape$/i;

/** La voce del Tape `path` nella Libreria, se ci sta, senza distinguere maiuscole e minuscole. */
export function entryOf(library: LibraryList, path: string) {
  const lower = path.toLowerCase();
  return library.tapes.find((b) => b.path.toLowerCase() === lower);
}

/** Il titolo del Tape `path`: quello della Libreria, o il nome del file. */
export function titleOf(library: LibraryList, path: string): string {
  return (
    entryOf(library, path)?.titolo ?? fileName(path).replace(EXTENSION, "")
  );
}

/**
 * La testata del documento di un Tape: il titolo grande, che un clic rinomina, il giorno, l'ora e la
 * durata, e le informazioni essenziali come etichette.
 */
export function TapeHeader({
  disabled,
  info,
  library,
  onCreato,
  onRename,
  parlanti,
  path,
}: {
  /** Ci lavora l'Attività in corso: niente rinomina né cambio di data. */
  disabled: boolean;
  info: TapeInfo | null;
  library: LibraryList;
  /** La data e l'ora nuove, locali: `2026-10-03T17:05`. */
  onCreato: (path: string, local: string) => void;
  onRename: (path: string, titolo: string) => void;
  /** Quanti Parlanti ha il testo. */
  parlanti: number;
  path: string;
}) {
  const { i18n, t } = useTranslation();
  const [renaming, setRenaming] = useState(false);
  const entry = entryOf(library, path);
  const titolo = titleOf(library, path);
  const folder = folderOf(path.toLowerCase());
  const siblings = library.tapes
    .filter((b) => b !== entry && folderOf(b.path.toLowerCase()) === folder)
    .map((b) => b.titolo);

  const start = useCallback(() => setRenaming(true), []);
  const cancel = useCallback(() => setRenaming(false), []);
  const submit = useCallback(
    (name: string) => {
      setRenaming(false);
      onRename(path, name);
    },
    [onRename, path]
  );
  const changeCreato = useCallback(
    (local: string) => onCreato(path, local),
    [onCreato, path]
  );

  return (
    <DocumentHeader
      chips={
        info ? (
          <InfoChips
            info={info}
            inLibrary={entry !== undefined}
            parlanti={parlanti}
          />
        ) : null
      }
      meta={
        info ? (
          <CreatoMeta
            disabled={disabled}
            info={info}
            onChange={changeCreato}
            text={[
              dayText(info.creato, new Date(), t, i18n.language),
              new Date(info.creato).toLocaleTimeString(i18n.language, {
                hour: "2-digit",
                minute: "2-digit",
              }),
              elapsedText(info.durataMs),
            ].join(" · ")}
          />
        ) : null
      }
      title={
        renaming ? (
          <NameInput
            className="h-auto py-1 font-display text-[2.5rem] leading-tight md:text-[2.5rem]"
            initial={titolo}
            label={t("library.renameName")}
            onCancel={cancel}
            onSubmit={submit}
            taken={siblings}
          />
        ) : (
          <button
            className="block max-w-full cursor-text text-balance rounded-md text-left decoration-2 decoration-muted-foreground/30 underline-offset-8 hover:underline focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:cursor-default disabled:no-underline"
            disabled={disabled}
            onClick={start}
            title={t("library.rename")}
            type="button"
          >
            {titolo}
          </button>
        )
      }
    />
  );
}

/**
 * La riga del giorno, dell'ora e della durata: un clic apre il campo data e ora. Invio o l'uscita
 * dal campo salvano, Esc annulla.
 */
function CreatoMeta({
  disabled,
  info,
  onChange,
  text,
}: {
  disabled: boolean;
  info: TapeInfo;
  onChange: (local: string) => void;
  text: string;
}) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState(false);
  const initial = dateTimeInput(info.creato);
  const start = useCallback(() => setEditing(true), []);
  const commit = useCallback(
    (e: FocusEvent<HTMLInputElement>) => {
      setEditing(false);
      // Vuoto: il campo non ha una data intera.
      if (e.currentTarget.value && e.currentTarget.value !== initial) {
        onChange(e.currentTarget.value);
      }
    },
    [initial, onChange]
  );
  const keyDown = useCallback((e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.currentTarget.blur();
    } else if (e.key === "Escape") {
      // Esc annulla: il campo torna al valore di partenza prima di uscire.
      e.currentTarget.value = e.currentTarget.defaultValue;
      e.currentTarget.blur();
    }
  }, []);
  if (editing) {
    return (
      <input
        aria-label={t("tape.dateLabel")}
        autoFocus
        className="h-8 rounded-md border bg-card px-2 text-base text-foreground tabular-nums"
        defaultValue={initial}
        onBlur={commit}
        onKeyDown={keyDown}
        type="datetime-local"
      />
    );
  }
  return (
    <button
      className="cursor-text rounded-md text-left decoration-muted-foreground/30 underline-offset-4 hover:underline focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:cursor-default disabled:no-underline"
      disabled={disabled}
      onClick={start}
      title={t("tape.changeDate")}
      type="button"
    >
      {text}
    </button>
  );
}

/**
 * Da dove viene il Tape: il file d'origine, o per una Registrazione con gli Ingressi separati
 * microfono e audio di sistema. Del mix di una Registrazione gli ingressi non si sanno.
 */
function sourceText(info: TapeInfo, t: TFunction): string {
  if (info.origine) {
    return t("tape.file", { name: info.origine });
  }
  return info.ingressiSeparati ? t("tape.ingressi") : t("tape.recording");
}

/** Le informazioni essenziali di un Tape come etichette. */
function InfoChips({
  inLibrary,
  info,
  parlanti,
}: {
  inLibrary: boolean;
  info: TapeInfo;
  parlanti: number;
}) {
  const { i18n, t } = useTranslation();
  const language =
    info.linguaParlato === "auto"
      ? t("speechLanguage.auto")
      : speechLanguageName(info.linguaParlato, i18n.language);
  return (
    <>
      <Chip
        icon={info.origine ? <FileAudio /> : <AudioLines />}
        title={info.origine ?? undefined}
      >
        {sourceText(info, t)}
      </Chip>
      {parlanti > 0 ? (
        <Chip icon={<Users />}>{t("tape.parlanti", { count: parlanti })}</Chip>
      ) : null}
      {/* Senza modello il Tape non ha testo: niente modello, lingua né "incompleto". */}
      {info.modello ? (
        <Chip icon={<Languages />}>{`${info.modello} · ${language}`}</Chip>
      ) : null}
      {info.modello && !info.completa ? (
        <Chip icon={<AlertTriangle />} tone="warning">
          {t("library.info.incomplete")}
        </Chip>
      ) : null}
      {inLibrary ? null : <Chip>{t("library.info.outside")}</Chip>}
    </>
  );
}

/** La testata di un documento: titolo, riga del giorno e della durata, etichette. */
export function DocumentHeader({
  chips,
  meta,
  title,
}: {
  chips?: ReactNode;
  meta?: ReactNode;
  title: ReactNode;
}) {
  return (
    <header className="flex flex-col gap-2 pt-12 pb-6">
      <h1 className="font-display font-medium text-[2.75rem] leading-[1.1] tracking-[-0.015em]">
        {title}
      </h1>
      {meta ? (
        <p className="text-[1.0625rem] text-muted-foreground tabular-nums">
          {meta}
        </p>
      ) : null}
      {chips ? <div className="mt-2 flex flex-wrap gap-2">{chips}</div> : null}
    </header>
  );
}

/** Un'informazione breve accanto al titolo. */
export function Chip({
  children,
  icon,
  title,
  tone,
}: {
  children: ReactNode;
  icon?: ReactNode;
  title?: string;
  tone?: "warning";
}) {
  let className =
    "inline-flex max-w-full items-center gap-1.5 truncate rounded-lg border px-2.5 py-1 text-sm [&_svg]:size-3.5 [&_svg]:shrink-0";
  if (tone === "warning") {
    className += " border-destructive/30 text-destructive";
  } else {
    className += " bg-card text-foreground/85";
  }
  return (
    <span className={className} title={title}>
      {icon}
      <span className="truncate">{children}</span>
    </span>
  );
}
