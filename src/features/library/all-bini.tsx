import { Trash2 } from "lucide-react";
import { type ChangeEvent, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import type { BinoEntry } from "@/bindings";
import { Button } from "@/components/ui/button";
import { type BinoOrder, sortBini } from "@/features/library/library";
import { MoveSelect, SELECT } from "@/features/library/move-select";
import { elapsedText } from "@/features/recording/recording";

/** L'elenco completo dei Bini di una Raccolta, ordinabile, con Sposta in… ed Elimina. */
export function AllBini({
  bini,
  onMove,
  onOpen,
  onTrash,
  raccolte,
  title,
}: {
  bini: BinoEntry[];
  onMove: (path: string, raccolta: string) => void;
  onOpen: (path: string) => void;
  onTrash: (bino: { path: string; titolo: string }) => void;
  raccolte: string[];
  /** Il nome della Raccolta mostrata. */
  title: string;
}) {
  const { i18n, t } = useTranslation();
  const [order, setOrder] = useState<BinoOrder>("date");
  const changeOrder = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      setOrder(e.target.value === "title" ? "title" : "date"),
    []
  );
  const dateFormat = new Intl.DateTimeFormat(i18n.language, {
    dateStyle: "medium",
    timeStyle: "short",
  });
  return (
    <section className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-center gap-3">
        <h1 className="min-w-0 flex-1 truncate font-medium text-lg">{title}</h1>
        <label className="flex items-center gap-2 text-sm">
          {t("library.sortBy")}
          <select className={SELECT} onChange={changeOrder} value={order}>
            <option value="date">{t("library.sortDate")}</option>
            <option value="title">{t("library.sortTitle")}</option>
          </select>
        </label>
      </div>
      {bini.length === 0 ? (
        <p className="text-muted-foreground text-sm">{t("library.empty")}</p>
      ) : (
        <ul className="min-h-0 flex-1 divide-y overflow-y-auto rounded-md border">
          {sortBini(bini, order).map((bino) => (
            <BinoRow
              bino={bino}
              dateFormat={dateFormat}
              key={bino.path}
              onMove={onMove}
              onOpen={onOpen}
              onTrash={onTrash}
              raccolte={raccolte}
            />
          ))}
        </ul>
      )}
    </section>
  );
}

function BinoRow({
  bino,
  dateFormat,
  onMove,
  onOpen,
  onTrash,
  raccolte,
}: {
  bino: BinoEntry;
  dateFormat: Intl.DateTimeFormat;
  onMove: (path: string, raccolta: string) => void;
  onOpen: (path: string) => void;
  onTrash: (bino: { path: string; titolo: string }) => void;
  raccolte: string[];
}) {
  const { t } = useTranslation();
  const open = useCallback(() => onOpen(bino.path), [bino.path, onOpen]);
  const move = useCallback(
    (raccolta: string) => onMove(bino.path, raccolta),
    [bino.path, onMove]
  );
  const trash = useCallback(() => onTrash(bino), [bino, onTrash]);
  return (
    <li className="flex items-center gap-3 px-3 py-2">
      <button
        className="min-w-0 flex-1 truncate text-left text-sm hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        onClick={open}
        title={bino.path}
        type="button"
      >
        {bino.titolo}
      </button>
      <span className="w-28 shrink-0 truncate text-muted-foreground text-xs">
        {bino.raccolta ?? t("library.none")}
      </span>
      <span className="w-40 shrink-0 text-muted-foreground text-xs tabular-nums">
        {dateFormat.format(new Date(bino.creato))}
      </span>
      <span className="w-16 shrink-0 text-right text-muted-foreground text-xs tabular-nums">
        {bino.durataMs === null ? "" : elapsedText(bino.durataMs)}
      </span>
      <MoveSelect
        current={bino.raccolta ?? ""}
        label={t("library.moveTo")}
        onMove={move}
        raccolte={raccolte}
      />
      <Button
        aria-label={t("library.delete")}
        onClick={trash}
        size="icon"
        title={t("library.delete")}
        variant="ghost"
      >
        <Trash2 />
      </Button>
    </li>
  );
}
