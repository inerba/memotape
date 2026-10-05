import {
  ArrowDown,
  ArrowUp,
  AudioLines,
  FolderPen,
  FolderX,
  Plus,
  Trash2,
} from "lucide-react";
import { type ReactNode, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type AppError,
  commands,
  type LibraryList,
  type TapeEntry,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  nextOrder,
  orderOf,
  raccoltaLabel,
  sortTapes,
  type TapeColumn,
  type TapeOrder,
  tapesOf,
} from "@/features/library/library";
import { MoveSelect } from "@/features/library/move-select";
import { NameInput } from "@/features/library/name-input";
import { DocumentHeader } from "@/features/library/tape-header";
import { elapsedText } from "@/features/recording/recording";
import { folderOf } from "@/features/source/file-name";

const PILL =
  "inline-flex h-8 items-center gap-1.5 rounded-full border px-3 text-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground [&_svg]:size-3.5";

/** L'ordinamento dell'elenco, ricordato in questo PC. */
const ORDER_KEY = "memotape.order";

function savedOrder(): TapeOrder {
  try {
    return orderOf(localStorage.getItem(ORDER_KEY));
  } catch {
    return orderOf(null);
  }
}

/**
 * La Libreria completa: le Raccolte (Tutta la Libreria, Senza raccolta, le altre e Nuova Raccolta,
 * con rinomina ed eliminazione di quella scelta) e i suoi Tape, ordinabili, con Sposta in… ed
 * Elimina. La Raccolta scelta è anche quella in cui finiscono le Registrazioni e i file nuovi.
 */
export function AllTapes({
  list,
  onError,
  onMove,
  onMoved,
  onOpen,
  onRaccolta,
  onTrash,
  raccolta,
}: {
  list: LibraryList;
  onError: (error: AppError) => void;
  onMove: (path: string, raccolta: string) => void;
  /** Una Raccolta rinominata: la cartella vecchia e quella nuova. */
  onMoved: (from: string, to: string) => void;
  onOpen: (path: string) => void;
  onRaccolta: (raccolta: string | null) => void;
  onTrash: (tape: { path: string; titolo: string }) => void;
  /** `null` Tutta la Libreria, `""` Senza raccolta. */
  raccolta: string | null;
}) {
  const { i18n, t } = useTranslation();
  const [order, setOrder] = useState(savedOrder);
  // La Raccolta che si sta creando (`""`) o rinominando.
  const [naming, setNaming] = useState<string | null>(null);
  const sort = useCallback((column: TapeColumn) => {
    setOrder((current) => {
      const next = nextOrder(current, column);
      try {
        localStorage.setItem(ORDER_KEY, JSON.stringify(next));
      } catch {
        // Senza memoria del browser l'ordine vale solo finché l'app resta aperta.
      }
      return next;
    });
  }, []);

  const submitName = useCallback(
    async (nome: string) => {
      const renaming = naming;
      setNaming(null);
      if (!renaming) {
        const result = await commands.createRaccolta(nome);
        if (result.status === "error") {
          onError(result.error);
        } else {
          onRaccolta(nome);
        }
        return;
      }
      const result = await commands.renameRaccolta(renaming, nome);
      if (result.status === "error") {
        onError(result.error);
        return;
      }
      // Il Tape aperto in quella Raccolta resta aperto, nella cartella nuova.
      onMoved(`${folderOf(result.data)}\\${renaming}`, result.data);
      onRaccolta(nome);
    },
    [naming, onError, onMoved, onRaccolta]
  );
  const cancelName = useCallback(() => setNaming(null), []);
  const startCreate = useCallback(() => setNaming(""), []);
  const startRename = useCallback(() => setNaming(raccolta), [raccolta]);
  const remove = useCallback(async () => {
    if (!raccolta) {
      return;
    }
    const result = await commands.deleteRaccolta(raccolta);
    if (result.status === "error") {
      onError(result.error);
    } else {
      onRaccolta(null);
    }
  }, [onError, onRaccolta, raccolta]);

  const dateFormat = new Intl.DateTimeFormat(i18n.language, {
    dateStyle: "medium",
    timeStyle: "short",
  });
  const tapes = tapesOf(list.tapes, raccolta);
  const scopes: (string | null)[] = [null, "", ...list.raccolte];

  return (
    <section className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
      <div className="mx-auto flex w-full max-w-[60rem] flex-col px-10 pb-12">
        <DocumentHeader
          meta={t("library.destination", {
            // Tutta la Libreria non è una cartella: le cose nuove vanno nella radice.
            raccolta: raccoltaLabel(raccolta ?? "", t),
          })}
          title={t("library.title")}
        />
        <div
          aria-label={t("library.raccolte")}
          className="flex flex-wrap items-center gap-2"
          role="toolbar"
        >
          {scopes.map((scope) => (
            <ScopePill
              key={scope ?? ".all"}
              onChoose={onRaccolta}
              scope={scope}
              selected={scope === raccolta}
            />
          ))}
          {naming === "" ? (
            <NameInput
              className="h-8 w-48 rounded-full"
              initial=""
              label={t("library.newRaccoltaName")}
              onCancel={cancelName}
              onSubmit={submitName}
              taken={list.raccolte}
            />
          ) : (
            <button
              className={`${PILL} border-dashed text-muted-foreground`}
              onClick={startCreate}
              type="button"
            >
              <Plus />
              {t("library.newRaccolta")}
            </button>
          )}
        </div>
        {raccolta ? (
          <div className="mt-3 flex items-center gap-1">
            {naming === raccolta ? (
              <NameInput
                className="h-8 w-56"
                initial={raccolta}
                label={t("library.renameRaccolta")}
                onCancel={cancelName}
                onSubmit={submitName}
                taken={list.raccolte.filter((r) => r !== raccolta)}
              />
            ) : (
              <Button
                className="text-muted-foreground"
                onClick={startRename}
                size="sm"
                variant="ghost"
              >
                <FolderPen />
                {t("library.renameRaccolta")}
              </Button>
            )}
            <Button
              className="text-muted-foreground"
              onClick={remove}
              size="sm"
              variant="ghost"
            >
              <FolderX />
              {t("library.deleteRaccolta")}
            </Button>
          </div>
        ) : null}
        <h2 className="mt-8 font-medium">{raccoltaLabel(raccolta, t)}</h2>
        <div className="mt-3 flex items-center gap-4 border-b pb-2 text-muted-foreground text-sm">
          <SortHeader
            className="flex-1"
            column="title"
            label={t("library.sortTitle")}
            onSort={sort}
            order={order}
          />
          {raccolta === null ? (
            <span className="w-32 shrink-0">{t("library.raccolta")}</span>
          ) : null}
          <SortHeader
            className="w-40"
            column="date"
            label={t("library.sortDate")}
            onSort={sort}
            order={order}
          />
          <SortHeader
            className="w-16 justify-end"
            column="duration"
            label={t("library.sortDuration")}
            onSort={sort}
            order={order}
          />
          <span className="w-44 shrink-0" />
        </div>
        {tapes.length === 0 ? (
          <p className="py-10 text-muted-foreground">{t("library.empty")}</p>
        ) : (
          <ul className="flex flex-col divide-y">
            {sortTapes(tapes, order).map((tape) => (
              <TapeRow
                dateFormat={dateFormat}
                key={tape.path}
                onMove={onMove}
                onOpen={onOpen}
                onTrash={onTrash}
                raccolte={list.raccolte}
                showRaccolta={raccolta === null}
                tape={tape}
              />
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}

/** Una pillola delle Raccolte. */
function ScopePill({
  onChoose,
  scope,
  selected,
}: {
  onChoose: (raccolta: string | null) => void;
  scope: string | null;
  selected: boolean;
}) {
  const { t } = useTranslation();
  const choose = useCallback(() => onChoose(scope), [onChoose, scope]);
  return (
    <button
      aria-pressed={selected}
      className={PILL}
      onClick={choose}
      type="button"
    >
      {raccoltaLabel(scope, t)}
    </button>
  );
}

/** L'intestazione di una colonna: un clic ordina per lei, il secondo inverte il verso. */
function SortHeader({
  className,
  column,
  label,
  onSort,
  order,
}: {
  className: string;
  column: TapeColumn;
  label: string;
  onSort: (column: TapeColumn) => void;
  order: TapeOrder;
}) {
  const { t } = useTranslation();
  const sort = useCallback(() => onSort(column), [column, onSort]);
  const active = order.column === column;
  let arrow: ReactNode = null;
  if (active) {
    arrow = order.descending ? <ArrowDown /> : <ArrowUp />;
  }
  return (
    <button
      // La colonna scelta dice anche il verso, che la freccia mostra solo a chi vede.
      aria-label={
        active
          ? `${label}, ${t(order.descending ? "library.descending" : "library.ascending")}`
          : label
      }
      aria-pressed={active}
      className={`flex shrink-0 items-center gap-1 rounded-md transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-pressed:text-foreground [&_svg]:size-3.5 ${className}`}
      onClick={sort}
      type="button"
    >
      {label}
      {arrow}
    </button>
  );
}

function TapeRow({
  tape,
  dateFormat,
  onMove,
  onOpen,
  onTrash,
  raccolte,
  showRaccolta,
}: {
  tape: TapeEntry;
  dateFormat: Intl.DateTimeFormat;
  onMove: (path: string, raccolta: string) => void;
  onOpen: (path: string) => void;
  onTrash: (tape: { path: string; titolo: string }) => void;
  raccolte: string[];
  showRaccolta: boolean;
}) {
  const { t } = useTranslation();
  const open = useCallback(() => onOpen(tape.path), [tape.path, onOpen]);
  const move = useCallback(
    (raccolta: string) => onMove(tape.path, raccolta),
    [tape.path, onMove]
  );
  const trash = useCallback(() => onTrash(tape), [tape, onTrash]);
  return (
    <li className="group flex items-center gap-4 py-2.5">
      <button
        className="flex min-w-0 flex-1 items-center gap-3 rounded-md py-1 text-left focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
        onClick={open}
        title={tape.path}
        type="button"
      >
        <AudioLines
          aria-hidden
          className="size-4 shrink-0 text-muted-foreground"
        />
        <span className="truncate decoration-muted-foreground/50 underline-offset-4 group-hover:underline">
          {tape.titolo}
        </span>
      </button>
      {showRaccolta ? (
        <span className="w-32 shrink-0 truncate text-muted-foreground text-sm">
          {tape.raccolta ?? t("library.none")}
        </span>
      ) : null}
      <span className="w-40 shrink-0 text-muted-foreground text-sm tabular-nums">
        {dateFormat.format(new Date(tape.creato))}
      </span>
      <span className="w-16 shrink-0 text-right text-muted-foreground text-sm tabular-nums">
        {tape.durataMs === null ? "" : elapsedText(tape.durataMs)}
      </span>
      {/* Le azioni della riga compaiono passandoci sopra o con il focus: pochi comandi in vista. */}
      <span className="flex w-44 shrink-0 items-center justify-end gap-1 opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 group-hover:opacity-100">
        <MoveSelect
          className="min-w-0 flex-1"
          current={tape.raccolta ?? ""}
          label={t("library.moveTo")}
          onMove={move}
          raccolte={raccolte}
        />
        <Button
          aria-label={t("library.delete")}
          className="text-muted-foreground hover:text-destructive"
          onClick={trash}
          size="icon-sm"
          title={t("library.delete")}
          variant="ghost"
        >
          <Trash2 />
        </Button>
      </span>
    </li>
  );
}
