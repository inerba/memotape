import { AlertTriangle, ChevronDown, Info, Pencil, Users } from "lucide-react";
import {
  type FocusEvent,
  Fragment,
  type KeyboardEvent,
  type ReactNode,
  useCallback,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import type { LibraryList, TapeInfo } from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { dateTimeInput, dayText } from "@/features/library/library";
import { NameInput } from "@/features/library/name-input";
import { tapeDetails } from "@/features/library/tape-details";
import { elapsedText } from "@/features/recording/recording";
import { fileName, folderOf } from "@/features/source/file-name";
import {
  diarizationCompleted,
  diarizationText,
} from "@/features/status/status";

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

/** I titoli degli altri Tape nella cartella di `path`: quelli che una rinomina non può prendere. */
export function siblingTitles(library: LibraryList, path: string): string[] {
  const lower = path.toLowerCase();
  const folder = folderOf(lower);
  return library.tapes
    .filter(
      (b) =>
        b.path.toLowerCase() !== lower &&
        folderOf(b.path.toLowerCase()) === folder
    )
    .map((b) => b.titolo);
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
            className="h-auto py-1 font-display text-[2rem] leading-tight md:text-[2rem]"
            initial={titolo}
            label={t("library.renameName")}
            onCancel={cancel}
            onSubmit={submit}
            taken={siblingTitles(library, path)}
          />
        ) : (
          <button
            className="block max-w-full cursor-text text-pretty rounded-md text-left text-[2rem] decoration-2 decoration-muted-foreground/30 underline-offset-8 hover:underline focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:cursor-default disabled:no-underline"
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
 * Le informazioni di un Tape sotto il titolo: Parlanti, correzioni e avvisi restano visibili, i
 * dettagli tecnici (origine, modello, Lingua del parlato, Ingressi) stanno nel popover Dettagli.
 */
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
  return (
    <>
      {parlanti > 0 ? (
        <Chip icon={<Users />}>{t("tape.parlanti", { count: parlanti })}</Chip>
      ) : null}
      {info.correttoAMano ? (
        <Chip icon={<Pencil />}>{t("tape.manuallyCorrected")}</Chip>
      ) : null}
      <PopoverMenu
        className="h-7 gap-1 rounded-full px-2.5 font-normal has-[>svg]:px-2.5 [&_svg]:size-3.5"
        icon={
          <>
            <Info />
            {t("tape.details.label")}
            <ChevronDown />
          </>
        }
        id="details"
        label={t("tape.details.label")}
        panelClassName="w-80"
        size="sm"
      >
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 px-2.5 py-2 text-[0.84rem]">
          {tapeDetails(info, t, i18n.language).map(({ label, value }) => (
            <Fragment key={label}>
              <dt className="text-muted-foreground">{label}</dt>
              <dd className="min-w-0 [overflow-wrap:anywhere]">{value}</dd>
            </Fragment>
          ))}
        </dl>
      </PopoverMenu>
      {/* Senza modello il Tape non ha testo: niente "incompleto". */}
      {info.modello && !info.completa ? (
        <Chip icon={<AlertTriangle />} tone="warning">
          {t("library.info.incomplete")}
        </Chip>
      ) : null}
      {/* Completata si legge già dai Parlanti: la chip segnala solo ciò che non è andato. */}
      {info.diarizzazione && !diarizationCompleted(info.diarizzazione) ? (
        <Chip icon={<Users />} tone="warning">
          {diarizationText(info.diarizzazione, t)}
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
    <header className="flex flex-col gap-2 pt-12 pb-6 [@media(max-height:700px)]:pt-6 [@media(max-height:700px)]:pb-4">
      {/* Nelle finestre basse il titolo cede spazio al testo. */}
      <h1 className="font-display font-medium text-[2.75rem] leading-[1.1] tracking-[-0.015em] [@media(max-height:700px)]:text-4xl">
        {title}
      </h1>
      {meta ? (
        <p className="text-[1.0625rem] text-muted-foreground tabular-nums">
          {meta}
        </p>
      ) : null}
      {chips ? (
        <div className="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1">
          {chips}
        </div>
      ) : null}
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
    "inline-flex max-w-full items-center gap-1.5 truncate text-sm [&_svg]:size-3.5 [&_svg]:shrink-0";
  // Le informazioni sono testo tenue; solo un avviso ha il bordo, per distinguersi.
  if (tone === "warning") {
    className +=
      " rounded-lg border border-destructive/30 px-2.5 py-1 text-destructive";
  } else {
    className += " text-muted-foreground";
  }
  return (
    <span className={className} title={title}>
      {icon}
      <span className="truncate">{children}</span>
    </span>
  );
}
