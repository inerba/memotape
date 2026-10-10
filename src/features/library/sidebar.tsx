import {
  ArrowRight,
  Clock,
  FileUp,
  Library,
  Mic,
  PanelLeftClose,
  PanelLeftOpen,
  Search,
  Settings,
} from "lucide-react";
import {
  type ChangeEvent,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router";
import type { AppError, LibraryList, TapeEntry } from "@/bindings";
import { BrandMark } from "@/components/brand-mark";
import { Button } from "@/components/ui/button";
import {
  type IconaDellaStriscia,
  riapertura,
} from "@/features/library/barra-laterale";
import {
  type GroupKey,
  groupByDate,
  RECENTI_MAX,
  recentWhen,
} from "@/features/library/library";
import { NameInput } from "@/features/library/name-input";
import { recentTitle } from "@/features/library/recent-tapes";
import { SearchResults } from "@/features/library/search-results";
import {
  contextMenu,
  TapeContextMenu,
  type TapeOperations,
  tapeKeys,
  useTapeRow,
} from "@/features/library/tape-context-menu";
import { siblingTitles } from "@/features/library/tape-header";
import { ariaTasti, conTasti } from "@/features/shortcuts/shortcuts";
import { useScorciatoia } from "@/features/shortcuts/shortcuts-provider";
import type { PhraseRef } from "@/features/transcription/phrases";
import { durationWords } from "@/lib/duration";

/**
 * L'Attività in corso in breve: il testo, l'avanzamento se c'è, se è una Registrazione e se è
 * guasta (la Trascrizione dal vivo si è fermata).
 */
export interface ActivitySummary {
  guasta: boolean;
  percent: number | null;
  recording: boolean;
  text: string;
}

const SIDEBAR_ID = "barra-laterale";

/** Lo spazio a sinistra delle barre in alto per il pulsante della barra laterale. */
export const SIDEBAR_TOGGLE_PADDING = "pl-[3.25rem]";

/**
 * Mostra o nasconde la barra laterale (Ctrl+B): sta sopra la riga del titolo di ogni vista, a
 * sinistra, che per questo lascia `SIDEBAR_TOGGLE_PADDING`.
 */
export function SidebarToggle({
  aperta,
  onToggle,
}: {
  aperta: boolean;
  onToggle: () => void;
}) {
  const { t } = useTranslation();
  const label = aperta ? t("sidebar.hide") : t("sidebar.show");
  return (
    <button
      aria-controls={SIDEBAR_ID}
      aria-expanded={aperta}
      aria-keyshortcuts={ariaTasti("barraLaterale")}
      aria-label={label}
      className="absolute top-2 left-3 flex size-8 items-center justify-center rounded-lg text-foreground/80 transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 [&_svg]:size-4"
      onClick={onToggle}
      title={conTasti(t, label, "barraLaterale")}
      type="button"
    >
      {aperta ? <PanelLeftClose /> : <PanelLeftOpen />}
    </button>
  );
}

/**
 * La barra laterale: il marchio, Nuova registrazione (`record`) e Importa un file, la ricerca
 * (Ctrl+K), i Tape recenti di tutta la Libreria per giorno (o i risultati della ricerca), l'Attività
 * in corso (solo se c'è) e in fondo la Libreria completa e Impostazioni. Chiusa (`aperta` falso) è
 * una striscia di icone; Ctrl+B la apre e la chiude.
 */
export function Sidebar({
  activity,
  aperta,
  busy,
  cancelling,
  list,
  onActivity,
  onCancel,
  onError,
  onImport,
  onHome,
  onOpen,
  onRecord,
  onShowAll,
  onToggle,
  operations,
  record,
  selected,
  showingAll,
  showingHome,
  update,
}: {
  /** L'Attività in corso, se c'è. */
  activity: ActivitySummary | null;
  aperta: boolean;
  /** Un'Attività in corso: niente Importa un file. */
  busy: boolean;
  cancelling: boolean;
  list: LibraryList;
  onActivity: () => void;
  onCancel: () => void;
  onError: (error: AppError) => void;
  onImport: () => void;
  onHome: () => void;
  /** Un Tape della barra laterale o dei risultati, con la Frase trovata su cui aprirlo. */
  onOpen: (path: string, phrase?: PhraseRef) => void;
  /** Nuova registrazione: il pulsante di `record`, l'icona della striscia e Ctrl+N. */
  onRecord: () => void;
  onShowAll: () => void;
  onToggle: () => void;
  /** Il menu e i tasti dei Recenti (non dei risultati della ricerca). */
  operations: TapeOperations;
  record: ReactNode;
  /** Il Tape aperto. */
  selected: string | null;
  /** La Libreria completa è aperta. */
  showingAll: boolean;
  showingHome: boolean;
  /** Un aggiornamento disponibile, sopra Libreria e Impostazioni. */
  update?: ReactNode;
}) {
  const { i18n, t } = useTranslation();
  const [query, setQuery] = useState("");
  const searching = query.trim() !== "";
  const search = useRef<HTMLInputElement>(null);
  const recents = useRef<HTMLDivElement>(null);
  // Dove va il focus quando la barra si riapre da un'icona della striscia.
  const dopoApertura = useRef<"ricerca" | "recenti" | null>(null);

  const focusSearch = useCallback(() => {
    search.current?.focus();
    search.current?.select();
  }, []);
  const riapri = useCallback(
    (icona: IconaDellaStriscia) => {
      dopoApertura.current = riapertura(icona);
      onToggle();
    },
    [onToggle]
  );
  useEffect(() => {
    if (!aperta) {
      return;
    }
    if (dopoApertura.current === "ricerca") {
      focusSearch();
    } else if (dopoApertura.current === "recenti") {
      recents.current
        ?.querySelector<HTMLElement>("[aria-current=page], button")
        ?.focus();
    }
    dopoApertura.current = null;
  }, [aperta, focusSearch]);
  // Ctrl+K porta alla ricerca da qualunque punto della finestra, riaprendo la barra se serve.
  const cerca = useCallback(() => {
    if (aperta) {
      focusSearch();
    } else {
      riapri("cerca");
    }
  }, [aperta, focusSearch, riapri]);
  useScorciatoia("cerca", cerca);
  useScorciatoia("barraLaterale", onToggle);
  useScorciatoia("nuovaRegistrazione", onRecord);
  useScorciatoia("importa", onImport);
  const navigate = useNavigate();
  const openSettings = useCallback(() => navigate("/settings"), [navigate]);
  useScorciatoia("impostazioni", openSettings);

  const typeQuery = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => setQuery(e.target.value),
    []
  );
  const clearOnEscape = useCallback(
    (e: ReactKeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Escape") {
        setQuery("");
      }
    },
    []
  );

  const monthFormat = new Intl.DateTimeFormat(i18n.language, {
    month: "long",
    year: "numeric",
  });
  const groupLabel = (key: GroupKey) => {
    if (key === "today" || key === "yesterday" || key === "week") {
      return t(`library.groups.${key}`);
    }
    const [year, month] = key.split("-").map(Number);
    return monthFormat.format(new Date(year ?? 0, (month ?? 1) - 1, 1));
  };
  const groups = groupByDate(list.tapes, new Date(), RECENTI_MAX);

  if (!aperta) {
    return (
      <Striscia
        activity={activity}
        busy={busy}
        onHome={onHome}
        onImport={onImport}
        onRecord={onRecord}
        onRiapri={riapri}
        onShowAll={onShowAll}
        selected={selected !== null}
        showingAll={showingAll}
        showingHome={showingHome}
      />
    );
  }

  return (
    <aside
      aria-label={t("library.title")}
      className="flex w-72 shrink-0 flex-col border-sidebar-border border-r bg-sidebar text-sidebar-foreground"
      id={SIDEBAR_ID}
    >
      <div
        className="flex h-14 shrink-0 items-center gap-2.5 px-5"
        data-tauri-drag-region
      >
        <button
          aria-current={showingHome ? "page" : undefined}
          aria-label={t("home.back")}
          className="flex items-center gap-2.5 rounded-md focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
          onClick={onHome}
          title={t("home.back")}
          type="button"
        >
          <BrandMark className="size-7" small />
          <span className="font-display font-medium text-[1.375rem] tracking-[-0.01em]">
            memotape
          </span>
        </button>
      </div>
      <div className="flex flex-col gap-1 px-3 pt-1">
        {record}
        <Button
          aria-keyshortcuts={ariaTasti("importa")}
          className="h-9 justify-start gap-2.5 px-3.5 font-normal text-sidebar-foreground/85"
          disabled={busy}
          onClick={onImport}
          title={conTasti(t, t("sidebar.importFile"), "importa")}
          variant="ghost"
        >
          <FileUp />
          {t("sidebar.importFile")}
        </Button>
      </div>
      <label className="relative mx-3 mt-3 block">
        <span className="sr-only">{t("sidebar.search")}</span>
        <Search
          aria-hidden
          className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground"
        />
        <input
          aria-keyshortcuts={ariaTasti("cerca")}
          className="h-9 w-full rounded-lg border border-sidebar-border bg-background/70 pr-14 pl-9 text-sm outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:bg-background focus-visible:ring-[3px] focus-visible:ring-ring/25 [&::-webkit-search-cancel-button]:hidden"
          onChange={typeQuery}
          onKeyDown={clearOnEscape}
          placeholder={t("sidebar.search")}
          ref={search}
          type="search"
          value={query}
        />
        <kbd className="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 rounded border border-sidebar-border bg-sidebar px-1.5 py-0.5 font-sans text-[0.6875rem] text-muted-foreground">
          Ctrl K
        </kbd>
      </label>
      <nav
        aria-labelledby="sidebar-recents"
        className="mt-5 flex min-h-0 flex-1 flex-col"
      >
        <SectionTitle className="mx-3" id="sidebar-recents">
          {searching ? t("sidebar.results") : t("sidebar.recents")}
        </SectionTitle>
        <div className="min-h-0 flex-1 overflow-y-auto px-3 pb-3" ref={recents}>
          {searching ? (
            <SearchResults
              onError={onError}
              onOpen={onOpen}
              query={query}
              raccolta={null}
              selected={selected}
            />
          ) : (
            <>
              {groups.length === 0 ? (
                <p className="px-2 text-muted-foreground text-sm">
                  {t("sidebar.noRecents")}
                </p>
              ) : null}
              {groups.map((group) => (
                <section className="mb-3" key={group.key}>
                  <h3 className="px-2 pt-1 pb-1.5 text-muted-foreground text-xs first-letter:uppercase">
                    {groupLabel(group.key)}
                  </h3>
                  <ul className="flex flex-col gap-0.5">
                    {group.tapes.map((tape) => (
                      <TapeItem
                        group={group.key}
                        key={tape.path}
                        library={list}
                        menu={`recente-${list.tapes.indexOf(tape)}`}
                        operations={operations}
                        selected={tape.path === selected}
                        tape={tape}
                        title={recentTitle(tape, list.tapes, t)}
                      />
                    ))}
                  </ul>
                </section>
              ))}
              {list.tapes.length > RECENTI_MAX ? (
                <button
                  className="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-muted-foreground text-sm transition-colors hover:bg-sidebar-accent/60 hover:text-sidebar-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
                  onClick={onShowAll}
                  type="button"
                >
                  <span className="flex-1">
                    {t("sidebar.allTapes", { count: list.tapes.length })}
                  </span>
                  <ArrowRight aria-hidden className="size-3.5 shrink-0" />
                </button>
              ) : null}
            </>
          )}
        </div>
      </nav>
      {activity ? (
        <section aria-labelledby="sidebar-activity" className="px-3 pb-3">
          <SectionTitle id="sidebar-activity">
            {t("sidebar.activity")}
          </SectionTitle>
          <ActivityCard
            activity={activity}
            cancelling={cancelling}
            onActivity={onActivity}
            onCancel={onCancel}
          />
        </section>
      ) : null}
      {update}
      <div className="flex flex-col gap-0.5 border-sidebar-border border-t px-3 py-2.5">
        <Button
          aria-current={showingAll ? "page" : undefined}
          className="h-9 justify-start gap-2.5 px-2.5 font-normal aria-[current=page]:bg-sidebar-accent"
          onClick={onShowAll}
          variant="ghost"
        >
          <Library />
          <span className="flex-1 text-left">{t("library.title")}</span>
          <span className="text-muted-foreground text-xs tabular-nums">
            {list.tapes.length}
          </span>
        </Button>
        <Button
          asChild
          className="h-9 justify-start gap-2.5 px-2.5 font-normal"
          variant="ghost"
        >
          <Link
            aria-keyshortcuts={ariaTasti("impostazioni")}
            title={conTasti(t, t("settings.open"), "impostazioni")}
            to="/settings"
          >
            <Settings />
            {t("settings.open")}
          </Link>
        </Button>
      </div>
    </aside>
  );
}

const STRISCIA_BASE =
  "relative flex size-10 items-center justify-center rounded-[10px] transition-colors focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-4";
const STRISCIA_ICONA = `${STRISCIA_BASE} text-sidebar-foreground hover:bg-sidebar-accent/60 aria-[current=page]:bg-sidebar-accent data-[sel]:bg-sidebar-accent`;
/** Nuova registrazione: il pulsante pieno, come nella barra aperta. */
const STRISCIA_PRIMARIA = `${STRISCIA_BASE} bg-primary text-primary-foreground hover:bg-primary/90`;

/**
 * La barra laterale chiusa: una striscia di icone da 60 px. Nuova registrazione, Importa, Libreria
 * e Impostazioni agiscono subito; Cerca e Recenti riaprono la barra (`riapertura`). L'Attività in
 * corso è un pallino sui Recenti: salvia, mattone se è guasta.
 */
function Striscia({
  activity,
  busy,
  onHome,
  onImport,
  onRecord,
  onRiapri,
  onShowAll,
  selected,
  showingAll,
  showingHome,
}: {
  activity: ActivitySummary | null;
  busy: boolean;
  onHome: () => void;
  onImport: () => void;
  onRecord: () => void;
  onRiapri: (icona: IconaDellaStriscia) => void;
  onShowAll: () => void;
  /** Un Tape è aperto: i Recenti sono il punto in cui si è. */
  selected: boolean;
  showingAll: boolean;
  showingHome: boolean;
}) {
  const { t } = useTranslation();
  const cerca = useCallback(() => onRiapri("cerca"), [onRiapri]);
  const recenti = useCallback(() => onRiapri("recenti"), [onRiapri]);
  const recentsLabel = activity
    ? `${t("sidebar.recents")} · ${activity.text}`
    : t("sidebar.recents");
  const searchLabel = conTasti(t, t("sidebar.search"), "cerca");
  return (
    <aside
      aria-label={t("library.title")}
      className="flex w-[60px] shrink-0 flex-col items-center border-sidebar-border border-r bg-sidebar pb-3.5 text-sidebar-foreground"
      id={SIDEBAR_ID}
    >
      <div
        className="flex h-14 w-full shrink-0 items-center justify-center"
        data-tauri-drag-region
      >
        <button
          aria-current={showingHome ? "page" : undefined}
          aria-label={t("home.back")}
          className="rounded-md focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
          onClick={onHome}
          title={t("home.back")}
          type="button"
        >
          <BrandMark className="size-7" small />
        </button>
      </div>
      <div className="flex flex-col items-center gap-1.5 pt-1.5">
        <button
          aria-keyshortcuts={ariaTasti("nuovaRegistrazione")}
          aria-label={t("sidebar.newRecording")}
          className={STRISCIA_PRIMARIA}
          disabled={busy}
          onClick={onRecord}
          title={conTasti(t, t("sidebar.newRecording"), "nuovaRegistrazione")}
          type="button"
        >
          <Mic />
        </button>
        <button
          aria-keyshortcuts={ariaTasti("importa")}
          aria-label={t("sidebar.importFile")}
          className={STRISCIA_ICONA}
          disabled={busy}
          onClick={onImport}
          title={conTasti(t, t("sidebar.importFile"), "importa")}
          type="button"
        >
          <FileUp />
        </button>
        <button
          aria-keyshortcuts={ariaTasti("cerca")}
          aria-label={t("sidebar.search")}
          className={STRISCIA_ICONA}
          onClick={cerca}
          title={searchLabel}
          type="button"
        >
          <Search />
        </button>
        <button
          aria-label={recentsLabel}
          className={STRISCIA_ICONA}
          data-sel={selected || undefined}
          onClick={recenti}
          title={recentsLabel}
          type="button"
        >
          <Clock />
          {activity ? (
            <span
              aria-hidden
              className={`absolute top-1.5 right-1.5 size-2 rounded-full ring-2 ring-sidebar motion-safe:animate-pulse ${activity.guasta ? "bg-destructive" : "bg-play"}`}
            />
          ) : null}
        </button>
      </div>
      <div className="mt-auto flex flex-col items-center gap-1.5">
        <button
          aria-current={showingAll ? "page" : undefined}
          aria-label={t("library.title")}
          className={STRISCIA_ICONA}
          onClick={onShowAll}
          title={t("library.title")}
          type="button"
        >
          <Library />
        </button>
        <Link
          aria-keyshortcuts={ariaTasti("impostazioni")}
          aria-label={t("settings.open")}
          className={STRISCIA_ICONA}
          title={conTasti(t, t("settings.open"), "impostazioni")}
          to="/settings"
        >
          <Settings />
        </Link>
      </div>
    </aside>
  );
}

function SectionTitle({
  children,
  className = "",
  id,
}: {
  children: ReactNode;
  className?: string;
  id: string;
}) {
  return (
    <h2
      className={`px-2 pb-1.5 font-medium text-[0.6875rem] text-muted-foreground uppercase tracking-[0.07em] ${className}`}
      id={id}
    >
      {children}
    </h2>
  );
}

/** L'Attività in corso: un clic riporta alla sua vista. */
function ActivityCard({
  activity,
  cancelling,
  onActivity,
  onCancel,
}: {
  activity: ActivitySummary;
  cancelling: boolean;
  onActivity: () => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex w-full flex-col rounded-lg border border-sidebar-border bg-background/70">
      <button
        className="flex w-full flex-col gap-2 rounded-lg px-3 py-2.5 text-left text-sm transition-colors hover:bg-background focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
        onClick={onActivity}
        title={t("library.activity")}
        type="button"
      >
        <span className="flex items-center gap-2.5">
          <span
            aria-hidden
            className={
              activity.recording
                ? "size-2 shrink-0 rounded-full bg-destructive motion-safe:animate-pulse"
                : "size-2 shrink-0 rounded-full bg-play motion-safe:animate-pulse"
            }
          />
          <span className="min-w-0 flex-1 truncate tabular-nums">
            {activity.text}
          </span>
        </span>
        {activity.recording ? null : (
          <span className="relative h-1 overflow-hidden rounded-full bg-play-soft">
            {activity.percent === null ? (
              <span className="absolute inset-y-0 w-1/3 rounded-full bg-play motion-safe:animate-[indeterminate_1.4s_ease-in-out_infinite]" />
            ) : (
              <span
                className="absolute inset-y-0 left-0 rounded-full bg-play transition-[width] duration-300"
                style={{ width: `${activity.percent}%` }}
              />
            )}
          </span>
        )}
      </button>
      {activity.recording ? null : (
        <div className="px-3 pb-2.5">
          <progress
            aria-label={activity.text}
            className="sr-only"
            max={100}
            value={activity.percent ?? undefined}
          />
          <Button
            className="h-8 w-full"
            disabled={cancelling}
            onClick={onCancel}
            size="sm"
            variant="outline"
          >
            {cancelling
              ? t("transcription.cancelling")
              : t("transcription.cancel")}
          </Button>
        </div>
      )}
    </div>
  );
}

/**
 * Un Recente: un clic lo apre; clic destro, tasto Menu o «…» aprono il menu del Tape, F2 lo
 * rinomina sul posto e Canc chiede il Cestino.
 */
function TapeItem({
  group,
  library,
  menu,
  operations,
  selected,
  tape,
  title,
}: {
  /** Il gruppo per data, che decide quanto dire del giorno. */
  group: GroupKey;
  library: LibraryList;
  /** L'id del menu, unico nella pagina. */
  menu: string;
  operations: TapeOperations;
  selected: boolean;
  tape: TapeEntry;
  title: string;
}) {
  const { i18n, t } = useTranslation();
  const {
    cancelRename,
    modificabile,
    open,
    ref,
    renaming,
    startRename,
    submitRename,
    trash,
  } = useTapeRow<HTMLButtonElement>(tape, library.raccolte, operations);
  if (renaming) {
    return (
      <li className="px-1 py-1.5">
        <NameInput
          initial={tape.titolo}
          label={t("library.renameName")}
          onCancel={cancelRename}
          onSubmit={submitRename}
          taken={siblingTitles(library, tape.path)}
        />
      </li>
    );
  }
  return (
    <li className="group/item relative">
      {/* Invio apre già il pulsante: dai tasti del Tape servono F2 e Canc. */}
      <button
        aria-current={selected ? "page" : undefined}
        className="group flex w-full items-center gap-3 rounded-lg py-2.5 pr-9 pl-2 text-left transition-colors hover:bg-sidebar-accent/60 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-[current=page]:bg-sidebar-accent"
        onClick={open}
        onContextMenu={contextMenu(menu)}
        onKeyDown={tapeKeys(
          modificabile ? { cestinaTape: trash, rinominaTape: startRename } : {}
        )}
        ref={ref}
        title={tape.path}
        type="button"
      >
        <span className="flex min-w-0 flex-1 flex-col">
          <span className="wrap-anywhere text-sm leading-snug group-aria-[current=page]:font-medium">
            {title}
          </span>
          {title === tape.titolo ? null : (
            <span className="sr-only">{tape.titolo}</span>
          )}
          <span className="flex items-center gap-1.5 text-muted-foreground text-xs tabular-nums">
            {recentWhen(tape.creato, group, i18n.language)}
            {tape.durataMs === null ? null : (
              <span className="inline-flex items-center gap-1">
                <Clock aria-hidden className="size-[11px]" />
                {durationWords(tape.durataMs, i18n.language)}
              </span>
            )}
          </span>
        </span>
      </button>
      <TapeContextMenu
        className="absolute top-1.5 right-1 opacity-0 transition-opacity duration-150 focus-visible:opacity-100 group-focus-within/item:opacity-100 group-hover/item:opacity-100 group-has-[:popover-open]/item:opacity-100"
        id={menu}
        onRename={startRename}
        operations={operations}
        raccolte={library.raccolte}
        tape={tape}
        title={title}
      />
    </li>
  );
}
