import { FolderOpen, Trash2 } from "lucide-react";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import type { LibraryList } from "@/bindings";
import { Button } from "@/components/ui/button";
import { MoveSelect } from "@/features/library/move-select";
import { NameInput } from "@/features/library/name-input";
import { fileName, folderOf } from "@/features/source/file-name";

const EXTENSION = /\.bino$/i;

/**
 * Il titolo del Bino `path`, che un clic rinomina, con Mostra in Esplora file, Sposta in… ed
 * Elimina; per un Bino fuori dalla Libreria, Aggiungi alla Libreria….
 */
export function BinoHeader({
  disabled,
  library,
  onMove,
  onRename,
  onReveal,
  onTrash,
  path,
}: {
  /** Ci lavora l'Attività in corso: niente rinomina, spostamento né Cestino. */
  disabled: boolean;
  library: LibraryList;
  onMove: (path: string, raccolta: string) => void;
  onRename: (path: string, titolo: string) => void;
  onReveal: (path: string) => void;
  onTrash: (bino: { path: string; titolo: string }) => void;
  path: string;
}) {
  const { t } = useTranslation();
  const [renaming, setRenaming] = useState(false);
  const lower = path.toLowerCase();
  const entry = library.bini.find((b) => b.path.toLowerCase() === lower);
  const titolo = entry?.titolo ?? fileName(path).replace(EXTENSION, "");
  // `null`: fuori dalla Libreria.
  const raccolta = entry ? (entry.raccolta ?? "") : null;
  const folder = folderOf(lower);
  const siblings = library.bini
    .filter(
      (b) =>
        b.path.toLowerCase() !== lower &&
        folderOf(b.path.toLowerCase()) === folder
    )
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
  const move = useCallback((to: string) => onMove(path, to), [onMove, path]);
  const reveal = useCallback(() => onReveal(path), [onReveal, path]);
  const trash = useCallback(
    () => onTrash({ path, titolo }),
    [onTrash, path, titolo]
  );

  return (
    <section className="flex items-center gap-2">
      <div className="min-w-0 flex-1">
        {renaming ? (
          <NameInput
            initial={titolo}
            label={t("library.renameName")}
            onCancel={cancel}
            onSubmit={submit}
            taken={siblings}
          />
        ) : (
          <button
            className="block max-w-full cursor-text truncate rounded-sm text-left font-medium text-lg hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-default disabled:no-underline"
            disabled={disabled}
            onClick={start}
            title={t("library.rename")}
            type="button"
          >
            {titolo}
          </button>
        )}
      </div>
      <Button
        aria-label={t("library.reveal")}
        onClick={reveal}
        size="icon"
        title={t("library.reveal")}
        variant="ghost"
      >
        <FolderOpen />
      </Button>
      <MoveSelect
        current={raccolta ?? undefined}
        disabled={disabled}
        label={
          raccolta === null ? t("library.addToLibrary") : t("library.moveTo")
        }
        onMove={move}
        raccolte={library.raccolte}
      />
      {raccolta === null ? null : (
        <Button
          aria-label={t("library.delete")}
          disabled={disabled}
          onClick={trash}
          size="icon"
          title={t("library.delete")}
          variant="ghost"
        >
          <Trash2 />
        </Button>
      )}
    </section>
  );
}
