import { FolderPen, FolderX, Library, ListIcon, Settings } from "lucide-react";
import { type ChangeEvent, type ReactNode, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";
import {
  type AppError,
  type BinoEntry,
  commands,
  type LibraryList,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import { biniOf, clockText, groupByDate } from "@/features/library/library";
import { SELECT } from "@/features/library/move-select";
import { NameInput } from "@/features/library/name-input";
import { elapsedText } from "@/features/recording/recording";
import { folderOf } from "@/features/source/file-name";

/** I valori del selettore che non sono Raccolte: nessuna Raccolta inizia con un punto. */
const ALL = ".all";
const NEW = ".new";

/**
 * La barra laterale: `actions` (Registra, Apri file) in cima, l'Attività in corso, il selettore
 * della Raccolta con le sue operazioni, i Bini della Raccolta per data e in fondo l'elenco completo
 * e Impostazioni.
 */
export function Sidebar({
  actions,
  activity,
  list,
  onActivity,
  onError,
  onMoved,
  onOpen,
  onRaccolta,
  onShowAll,
  raccolta,
  selected,
}: {
  actions: ReactNode;
  /** Lo stato in breve dell'Attività in corso, se c'è. */
  activity: string | null;
  list: LibraryList;
  onActivity: () => void;
  onError: (error: AppError) => void;
  /** Una Raccolta rinominata: la cartella vecchia e quella nuova. */
  onMoved: (from: string, to: string) => void;
  onOpen: (path: string) => void;
  onRaccolta: (raccolta: string | null) => void;
  onShowAll: () => void;
  /** `null` Tutta la Libreria, `""` Senza raccolta. */
  raccolta: string | null;
  /** Il Bino aperto. */
  selected: string | null;
}) {
  const { i18n, t } = useTranslation();
  // La Raccolta che si sta creando (`""`) o rinominando.
  const [naming, setNaming] = useState<string | null>(null);

  const choose = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) => {
      const { value } = e.target;
      if (value === NEW) {
        setNaming("");
      } else {
        onRaccolta(value === ALL ? null : value);
      }
    },
    [onRaccolta]
  );

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
      // Il Bino aperto in quella Raccolta resta aperto, nella cartella nuova.
      onMoved(`${folderOf(result.data)}\\${renaming}`, result.data);
      onRaccolta(nome);
    },
    [naming, onError, onMoved, onRaccolta]
  );

  const cancelName = useCallback(() => setNaming(null), []);

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
  const groups = groupByDate(biniOf(list.bini, raccolta), new Date());

  return (
    <aside
      aria-label={t("library.title")}
      className="flex w-72 shrink-0 flex-col gap-3 border-r bg-muted/30 p-3"
    >
      <div className="flex flex-col gap-2">{actions}</div>
      {activity ? (
        <Button
          className="justify-start"
          onClick={onActivity}
          title={t("library.activity")}
          variant="secondary"
        >
          <span className="size-2 shrink-0 animate-pulse rounded-full bg-destructive" />
          <span className="truncate">{activity}</span>
        </Button>
      ) : null}
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-2">
          <Library aria-hidden className="size-4 shrink-0" />
          <select
            aria-label={t("library.raccolta")}
            className={`${SELECT} flex-1`}
            onChange={choose}
            value={raccolta ?? ALL}
          >
            <option value={ALL}>{t("library.all")}</option>
            <option value="">{t("library.none")}</option>
            {list.raccolte.length > 0 ? (
              <optgroup label={t("library.raccolte")}>
                {list.raccolte.map((r) => (
                  <option key={r} value={r}>
                    {r}
                  </option>
                ))}
              </optgroup>
            ) : null}
            <option value={NEW}>{t("library.newRaccolta")}</option>
          </select>
          {raccolta ? (
            <>
              <Button
                aria-label={t("library.renameRaccolta")}
                onClick={startRename}
                size="icon"
                title={t("library.renameRaccolta")}
                variant="ghost"
              >
                <FolderPen />
              </Button>
              <Button
                aria-label={t("library.deleteRaccolta")}
                onClick={remove}
                size="icon"
                title={t("library.deleteRaccolta")}
                variant="ghost"
              >
                <FolderX />
              </Button>
            </>
          ) : null}
        </div>
        {naming === null ? null : (
          <NameInput
            initial={naming}
            label={
              naming
                ? t("library.renameRaccolta")
                : t("library.newRaccoltaName")
            }
            onCancel={cancelName}
            onSubmit={submitName}
            taken={list.raccolte.filter((r) => r !== naming)}
          />
        )}
      </div>
      <nav className="-mx-1 min-h-0 flex-1 overflow-y-auto px-1">
        {groups.length === 0 ? (
          <p className="px-2 text-muted-foreground text-sm">
            {t("library.empty")}
          </p>
        ) : null}
        {groups.map((group) => (
          <section className="mb-3" key={group.key}>
            <h2 className="px-2 pb-1 font-medium text-muted-foreground text-xs first-letter:uppercase">
              {groupLabel(group.key)}
            </h2>
            <ul>
              {group.bini.map((bino) => (
                <li key={bino.path}>
                  <BinoItem
                    bino={bino}
                    onOpen={onOpen}
                    selected={bino.path === selected}
                  />
                </li>
              ))}
            </ul>
          </section>
        ))}
      </nav>
      <div className="flex flex-col gap-1 border-t pt-2">
        <Button className="justify-start" onClick={onShowAll} variant="ghost">
          <ListIcon />
          {t("library.showAll")}
        </Button>
        <Button asChild className="justify-start" variant="ghost">
          <Link to="/settings">
            <Settings />
            {t("settings.open")}
          </Link>
        </Button>
      </div>
    </aside>
  );
}

function BinoItem({
  bino,
  onOpen,
  selected,
}: {
  bino: BinoEntry;
  onOpen: (path: string) => void;
  selected: boolean;
}) {
  const open = useCallback(() => onOpen(bino.path), [bino.path, onOpen]);
  return (
    <button
      aria-current={selected ? "page" : undefined}
      className="flex w-full items-baseline gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring aria-[current=page]:bg-accent aria-[current=page]:font-medium"
      onClick={open}
      title={bino.path}
      type="button"
    >
      <span className="min-w-0 flex-1 truncate">{bino.titolo}</span>
      <span className="shrink-0 text-muted-foreground text-xs tabular-nums">
        {clockText(bino.creato)}
        {bino.durataMs === null ? "" : ` · ${elapsedText(bino.durataMs)}`}
      </span>
    </button>
  );
}
