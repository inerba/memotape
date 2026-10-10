import {
  ArrowRight,
  Clock,
  FileUp,
  Library,
  Search,
  Settings,
} from "lucide-react";
import {
  type ChangeEvent,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
  useCallback,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router";
import type { AppError, LibraryList, TapeEntry } from "@/bindings";
import { BrandMark } from "@/components/brand-mark";
import { Button } from "@/components/ui/button";
import {
  durationWords,
  groupByDate,
  RECENTI_MAX,
  recentWhen,
} from "@/features/library/library";
import { recentTitle } from "@/features/library/recent-tapes";
import { SearchResults } from "@/features/library/search-results";
import {
  ariaTasti,
  conTasti,
  useScorciatoia,
} from "@/features/shortcuts/shortcuts-provider";
import type { PhraseRef } from "@/features/transcription/phrases";

/** L'Attività in corso in breve: il testo, l'avanzamento se c'è e se è una Registrazione. */
export interface ActivitySummary {
  percent: number | null;
  recording: boolean;
  text: string;
}

/**
 * La barra laterale: il marchio, Nuova registrazione (`record`) e Importa un file, la ricerca
 * (Ctrl+K), i Tape recenti di tutta la Libreria per giorno (o i risultati della ricerca), l'Attività
 * in corso (solo se c'è) e in fondo la Libreria completa e Impostazioni.
 */
export function Sidebar({
  activity,
  busy,
  cancelling,
  list,
  onActivity,
  onCancel,
  onError,
  onImport,
  onHome,
  onOpen,
  onShowAll,
  record,
  selected,
  showingAll,
  showingHome,
  update,
}: {
  /** L'Attività in corso, se c'è. */
  activity: ActivitySummary | null;
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
  onShowAll: () => void;
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

  // Ctrl+K porta alla ricerca da qualunque punto della finestra.
  const focusSearch = useCallback(() => {
    search.current?.focus();
    search.current?.select();
  }, []);
  useScorciatoia("cerca", focusSearch);
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
  const groupLabel = (key: string) => {
    if (key === "today" || key === "yesterday" || key === "week") {
      return t(`library.groups.${key}`);
    }
    const [year, month] = key.split("-").map(Number);
    return monthFormat.format(new Date(year ?? 0, (month ?? 1) - 1, 1));
  };
  const groups = groupByDate(list.tapes, new Date(), RECENTI_MAX);

  return (
    <aside
      aria-label={t("library.title")}
      className="flex w-72 shrink-0 flex-col border-sidebar-border border-r bg-sidebar text-sidebar-foreground"
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
        <div className="min-h-0 flex-1 overflow-y-auto px-3 pb-3">
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
                      <li key={tape.path}>
                        <TapeItem
                          group={group.key}
                          onOpen={onOpen}
                          selected={tape.path === selected}
                          tape={tape}
                          title={recentTitle(tape, list.tapes, t)}
                        />
                      </li>
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

function TapeItem({
  group,
  tape,
  onOpen,
  selected,
  title,
}: {
  /** Il gruppo per data, che decide quanto dire del giorno. */
  group: string;
  tape: TapeEntry;
  onOpen: (path: string) => void;
  selected: boolean;
  title: string;
}) {
  const { i18n } = useTranslation();
  const open = useCallback(() => onOpen(tape.path), [tape.path, onOpen]);
  return (
    <button
      aria-current={selected ? "page" : undefined}
      className="group flex w-full items-center gap-3 rounded-lg px-2 py-2.5 text-left transition-colors hover:bg-sidebar-accent/60 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-[current=page]:bg-sidebar-accent"
      onClick={open}
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
  );
}
