import {
  ArrowDown,
  ArrowUp,
  FolderPen,
  FolderX,
  Plus,
  Trash2,
} from "lucide-react";
import {
  type KeyboardEvent,
  type ReactNode,
  useCallback,
  useMemo,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import {
  type AppError,
  commands,
  type LibraryList,
  type TapeEntry,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  azioniDelTape,
  nextOrder,
  orderOf,
  raccoltaLabel,
  rigaDopo,
  sortTapes,
  type TapeColumn,
  type TapeOrder,
  tapesOf,
  versoDellOrdine,
} from "@/features/library/library";
import { NameInput } from "@/features/library/name-input";
import { recentTitle } from "@/features/library/recent-tapes";
import {
  contextMenu,
  TapeContextMenu,
  type TapeOperations,
  tapeKeys,
  useFocusBack,
} from "@/features/library/tape-context-menu";
import { DocumentHeader, siblingTitles } from "@/features/library/tape-header";
import { elapsedText } from "@/features/recording/recording";
import { folderOf } from "@/features/source/file-name";

const PILL =
  "inline-flex h-8 items-center gap-1.5 rounded-full border px-3 text-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground [&_svg]:size-3.5";

/** Le righe di un salto di Pagina giù o su. ponytail: fisse, non misurate sull'altezza in vista. */
const PAGINA = 10;

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
  onMoved,
  onRaccolta,
  operations,
  raccolta,
}: {
  list: LibraryList;
  onError: (error: AppError) => void;
  /** Una Raccolta rinominata: la cartella vecchia e quella nuova. */
  onMoved: (from: string, to: string) => void;
  onRaccolta: (raccolta: string | null) => void;
  operations: TapeOperations;
  /** `null` Tutta la Libreria, `""` Senza raccolta. */
  raccolta: string | null;
}) {
  const { i18n, t } = useTranslation();
  const [order, setOrder] = useState(savedOrder);
  // La riga nel Tab: la tabella è un solo elemento, le frecce passano da una riga all'altra.
  const [active, setActive] = useState(0);
  // `recentTitle` scorre tutta la Libreria: una volta per Tape, non a ogni confronto dell'ordinamento.
  const shown = useMemo(() => {
    const titles = new Map(
      list.tapes.map((tape) => [tape.path, recentTitle(tape, list.tapes, t)])
    );
    return (tape: TapeEntry) => titles.get(tape.path) ?? tape.titolo;
  }, [list.tapes, t]);
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
      <div className="@container flex w-full flex-col px-10 pb-12">
        <DocumentHeader title={t("library.title")} />
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
        {/* La pillola scelta dice già cosa si vede: qui solo dove finiscono le cose nuove. */}
        <p className="mt-8 text-muted-foreground text-sm">
          {t("library.destination", {
            // Tutta la Libreria non è una cartella: le cose nuove vanno nella radice.
            raccolta: raccoltaLabel(raccolta ?? "", t),
          })}
        </p>
        <table
          aria-label={t("library.title")}
          // Bordi separati: con quelli uniti Chromium non disegna il contorno della riga col focus.
          className="mt-3 w-full table-fixed border-separate border-spacing-0 text-sm"
        >
          {/* Il titolo prende la larghezza che resta; nelle finestre strette la Raccolta cede. */}
          <colgroup>
            <col />
            {raccolta === null ? (
              <col className="@2xl:table-column hidden w-36" />
            ) : null}
            <col className="w-40" />
            <col className="w-20" />
            <col className="w-20" />
          </colgroup>
          <thead className="text-muted-foreground">
            <tr className="[&>th]:border-b">
              <th className="pb-2 pl-2 text-left font-normal">
                <SortHeader
                  column="title"
                  label={t("library.sortTitle")}
                  onSort={sort}
                  order={order}
                />
              </th>
              {raccolta === null ? (
                <th className="@2xl:table-cell hidden pb-2 pl-4 text-left font-normal">
                  {t("library.raccolta")}
                </th>
              ) : null}
              <th className="pb-2 pl-4 text-left font-normal">
                <SortHeader
                  column="date"
                  label={t("library.sortDate")}
                  onSort={sort}
                  order={order}
                />
              </th>
              <th className="pb-2 pl-4 text-right font-normal">
                <SortHeader
                  column="duration"
                  label={t("library.sortDuration")}
                  onSort={sort}
                  order={order}
                />
              </th>
              <th>
                <span className="sr-only">{t("library.more")}</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {sortTapes(tapes, order, shown).map((tape, index) => (
              <TapeRow
                dateFormat={dateFormat}
                index={index}
                inTab={index === Math.min(active, tapes.length - 1)}
                key={tape.path}
                library={list}
                onFocus={setActive}
                operations={operations}
                showRaccolta={raccolta === null}
                tape={tape}
                title={shown(tape)}
              />
            ))}
          </tbody>
        </table>
        {tapes.length === 0 ? (
          <p className="py-10 text-muted-foreground">{t("library.empty")}</p>
        ) : null}
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
  className = "",
  column,
  label,
  onSort,
  order,
}: {
  className?: string;
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
      // La colonna scelta dice anche il suo verso («Data, dalla più recente»), che la freccia
      // mostra solo a chi vede.
      aria-label={active ? `${label}, ${t(versoDellOrdine(order))}` : label}
      aria-pressed={active}
      className={`-my-0.5 inline-flex min-h-6 shrink-0 items-center gap-1 rounded-md transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-pressed:text-foreground [&_svg]:size-3.5 ${className}`}
      onClick={sort}
      type="button"
    >
      {label}
      {arrow}
    </button>
  );
}

/**
 * Una riga della tabella: il focus è sulla riga (Invio apre, F2 rinomina, Canc chiede il Cestino,
 * il tasto Menu apre il menu); i pulsanti dentro restano fuori dal Tab.
 */
function TapeRow({
  dateFormat,
  index,
  inTab,
  library,
  onFocus,
  operations,
  showRaccolta,
  tape,
  title,
}: {
  dateFormat: Intl.DateTimeFormat;
  index: number;
  /** La riga che il Tab raggiunge. */
  inTab: boolean;
  library: LibraryList;
  onFocus: (index: number) => void;
  operations: TapeOperations;
  showRaccolta: boolean;
  tape: TapeEntry;
  /** Il titolo mostrato: compatto per le Registrazioni con il nome automatico. */
  title: string;
}) {
  const { t } = useTranslation();
  const [renaming, setRenaming] = useState(false);
  const row = useFocusBack<HTMLTableRowElement>(renaming);
  const { modificabile } = azioniDelTape(
    tape,
    library.raccolte,
    operations.lavorato
  );
  const { onOpen, onRename, onTrash } = operations;
  const menu = `tape-${index}`;
  const open = useCallback(() => onOpen(tape.path), [tape.path, onOpen]);
  const startRename = useCallback(() => setRenaming(true), []);
  const cancelRename = useCallback(() => setRenaming(false), []);
  const submitRename = useCallback(
    (titolo: string) => {
      setRenaming(false);
      onRename(tape.path, titolo);
    },
    [onRename, tape.path]
  );
  const trash = useCallback(() => onTrash(tape), [tape, onTrash]);
  const focused = useCallback(() => onFocus(index), [index, onFocus]);
  // Le frecce, Inizio, Fine e Pagina su e giù passano il focus a un'altra riga.
  const keyDown = useCallback(
    (e: KeyboardEvent<HTMLTableRowElement>) => {
      const tr = e.currentTarget;
      const rows = (tr.parentElement as HTMLTableSectionElement | null)?.rows;
      const next =
        e.target === tr && rows
          ? rigaDopo(e.key, tr.sectionRowIndex, rows.length, PAGINA)
          : null;
      if (next !== null) {
        e.preventDefault();
        rows?.[next]?.focus();
        return;
      }
      tapeKeys(
        modificabile
          ? { apriTape: open, cestinaTape: trash, rinominaTape: startRename }
          : { apriTape: open }
      )(e);
    },
    [modificabile, open, startRename, trash]
  );
  return (
    <tr
      className="group transition-colors hover:bg-accent/50 focus-visible:bg-accent/50 focus-visible:outline-2 focus-visible:outline-ring focus-visible:-outline-offset-2 has-[:popover-open]:bg-accent/50 [&>td]:border-b"
      onContextMenu={contextMenu(menu)}
      onFocus={focused}
      onKeyDown={keyDown}
      ref={row}
      tabIndex={inTab ? 0 : -1}
    >
      <td className="py-2.5 pr-2 pl-2">
        {renaming ? (
          <NameInput
            initial={tape.titolo}
            label={t("library.renameName")}
            onCancel={cancelRename}
            onSubmit={submitRename}
            taken={siblingTitles(library, tape.path)}
          />
        ) : (
          <button
            className="block max-w-full truncate rounded-md py-1 text-left decoration-muted-foreground/50 underline-offset-4 hover:underline"
            onClick={open}
            tabIndex={-1}
            title={tape.path}
            type="button"
          >
            {title}
          </button>
        )}
      </td>
      {showRaccolta ? (
        <td className="@2xl:table-cell hidden truncate pl-4 text-muted-foreground">
          {tape.raccolta ?? t("library.none")}
        </td>
      ) : null}
      <td className="truncate pl-4 text-muted-foreground tabular-nums">
        {dateFormat.format(new Date(tape.creato))}
      </td>
      <td className="pl-4 text-right text-muted-foreground tabular-nums">
        {tape.durataMs === null ? "" : elapsedText(tape.durataMs)}
      </td>
      {/* Le azioni compaiono passandoci sopra o con il focus sulla riga: pochi comandi in vista. */}
      <td>
        <span className="flex items-center justify-end opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 group-hover:opacity-100 group-has-[:popover-open]:opacity-100">
          <Button
            aria-label={t("library.deleteNamed", { titolo: title })}
            className="text-muted-foreground hover:text-destructive"
            disabled={!modificabile}
            onClick={trash}
            size="icon-sm"
            tabIndex={-1}
            title={t("library.deleteNamed", { titolo: title })}
            variant="ghost"
          >
            <Trash2 />
          </Button>
          <TapeContextMenu
            id={menu}
            onRename={startRename}
            operations={operations}
            raccolte={library.raccolte}
            tape={tape}
            title={title}
          />
        </span>
      </td>
    </tr>
  );
}
