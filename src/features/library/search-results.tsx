import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type AppError,
  type BinoEntry,
  commands,
  events,
  type SearchHit,
  type SearchResult,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import { raccoltaLabel } from "@/features/library/library";
import { markedParts } from "@/features/library/search";
import { elapsedText } from "@/features/recording/recording";
import type { PhraseRef } from "@/features/transcription/phrases";

/** Quanto si aspetta dopo l'ultimo tasto prima di cercare. */
const DEBOUNCE_MS = 150;

/**
 * I risultati della ricerca di `query` nella Raccolta `raccolta` (`null` Tutta la Libreria, `""`
 * Senza raccolta) e in fondo "Cerca in tutta la Libreria". Si ricerca a ogni `library-changed`: un
 * Bino rinominato o spostato ha un altro percorso.
 */
export function SearchResults({
  onError,
  onOpen,
  query,
  raccolta,
  selected,
}: {
  onError: (error: AppError) => void;
  onOpen: (path: string, phrase?: PhraseRef) => void;
  query: string;
  raccolta: string | null;
  selected: string | null;
}) {
  const { t } = useTranslation();
  // Allargata a tutta la Libreria con il pulsante in fondo.
  const [wide, setWide] = useState(false);
  const [results, setResults] = useState<SearchResult[] | null>(null);
  const scope = wide ? null : raccolta;

  useEffect(() => {
    let stale = false;
    const search = async () => {
      const result = await commands.librarySearch(query, scope);
      // Un'altra ricerca è partita nel frattempo.
      if (stale) {
        return;
      }
      if (result.status === "ok") {
        setResults(result.data);
      } else {
        onError(result.error);
      }
    };
    const timer = setTimeout(search, DEBOUNCE_MS);
    const changed = events.libraryChanged.listen(search);
    return () => {
      stale = true;
      clearTimeout(timer);
      changed.then((stop) => stop());
    };
  }, [onError, query, scope]);

  const widen = useCallback(() => setWide(true), []);

  return (
    <div className="flex flex-col gap-2">
      {results?.length === 0 ? (
        <p className="px-2 text-muted-foreground text-sm">
          {t("library.noResults")}
        </p>
      ) : null}
      {results?.map((result) => (
        <ResultItem
          key={result.bino.path}
          onOpen={onOpen}
          result={result}
          selected={result.bino.path === selected}
          showRaccolta={scope === null}
        />
      ))}
      {scope === null ? null : (
        <Button className="justify-start" onClick={widen} variant="link">
          {t("library.searchAll")}
        </Button>
      )}
    </div>
  );
}

function ResultItem({
  onOpen,
  result,
  selected,
  showRaccolta,
}: {
  onOpen: (path: string, phrase?: PhraseRef) => void;
  result: SearchResult;
  selected: boolean;
  showRaccolta: boolean;
}) {
  const { t } = useTranslation();
  const { bino } = result;
  const open = useCallback(() => onOpen(bino.path), [bino.path, onOpen]);
  return (
    <section>
      <button
        aria-current={selected ? "page" : undefined}
        className="flex w-full items-baseline gap-2 rounded-lg px-2 py-1.5 text-left font-medium text-sm transition-colors hover:bg-sidebar-accent/60 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-[current=page]:bg-sidebar-accent"
        onClick={open}
        title={bino.path}
        type="button"
      >
        <span className="min-w-0 flex-1 truncate">{bino.titolo}</span>
        {showRaccolta ? (
          <span className="max-w-24 shrink-0 truncate font-normal text-muted-foreground text-xs">
            {raccoltaLabel(bino.raccolta ?? "", t)}
          </span>
        ) : null}
      </button>
      <ul>
        {result.frasi.map((hit) => (
          <li key={`${hit.ingresso}:${hit.phraseId}`}>
            <HitItem bino={bino} hit={hit} onOpen={onOpen} />
          </li>
        ))}
      </ul>
    </section>
  );
}

function HitItem({
  bino,
  hit,
  onOpen,
}: {
  bino: BinoEntry;
  hit: SearchHit;
  onOpen: (path: string, phrase?: PhraseRef) => void;
}) {
  const open = useCallback(
    () => onOpen(bino.path, { ingresso: hit.ingresso, phraseId: hit.phraseId }),
    [bino.path, hit.ingresso, hit.phraseId, onOpen]
  );
  return (
    <button
      className="flex w-full gap-2 rounded-lg px-2 py-1 text-left text-xs leading-relaxed transition-colors hover:bg-sidebar-accent/60 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
      onClick={open}
      type="button"
    >
      <span className="shrink-0 text-muted-foreground tabular-nums">
        {elapsedText(hit.inizioMs)}
      </span>
      <span className="line-clamp-2 min-w-0">
        {markedParts(hit.estratto).map((part, i) =>
          part.mark ? (
            // biome-ignore lint/suspicious/noArrayIndexKey: le parti non cambiano ordine
            <mark className="rounded-sm bg-play-soft text-foreground" key={i}>
              {part.text}
            </mark>
          ) : (
            part.text
          )
        )}
      </span>
    </button>
  );
}
