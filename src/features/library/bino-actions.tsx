import { Ellipsis, FileDown, FolderOpen, Trash2 } from "lucide-react";
import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands, type LibraryList } from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { Button } from "@/components/ui/button";
import { entryOf } from "@/features/library/bino-header";
import { MoveSelect } from "@/features/library/move-select";
import { fileName } from "@/features/source/file-name";

const EXTENSION = /\.bino$/i;

function closeMenu() {
  document.getElementById("menu-more")?.hidePopover();
}

/**
 * Esporta Markdown…, Mostra in Esplora file e "…" con Sposta in… ed Elimina; per un Bino fuori dalla
 * Libreria, Aggiungi alla Libreria….
 */
export function BinoActions({
  disabled,
  library,
  onError,
  onExported,
  onMove,
  onReveal,
  onTrash,
  path,
}: {
  /** Ci lavora l'Attività in corso: niente spostamento né Cestino. */
  disabled: boolean;
  library: LibraryList;
  onError: (error: AppError) => void;
  onExported: (path: string) => void;
  onMove: (path: string, raccolta: string) => void;
  onReveal: (path: string) => void;
  onTrash: (bino: { path: string; titolo: string }) => void;
  path: string;
}) {
  const { t } = useTranslation();
  const entry = entryOf(library, path);
  // `null`: fuori dalla Libreria.
  const raccolta = entry ? (entry.raccolta ?? "") : null;

  const exportMarkdown = useCallback(async () => {
    const result = await commands.exportMarkdown(path);
    if (result.status === "error") {
      onError(result.error);
    } else if (result.data) {
      onExported(result.data);
    }
  }, [onError, onExported, path]);
  const reveal = useCallback(() => onReveal(path), [onReveal, path]);
  const move = useCallback(
    (to: string) => {
      closeMenu();
      onMove(path, to);
    },
    [onMove, path]
  );
  const trash = useCallback(() => {
    closeMenu();
    onTrash({
      path,
      titolo: entry?.titolo ?? fileName(path).replace(EXTENSION, ""),
    });
  }, [entry, onTrash, path]);

  return (
    <>
      <Button onClick={exportMarkdown} variant="outline">
        <FileDown />
        {t("transcription.export")}
      </Button>
      <Button
        aria-label={t("library.reveal")}
        onClick={reveal}
        size="icon"
        title={t("library.reveal")}
        variant="ghost"
      >
        <FolderOpen />
      </Button>
      <PopoverMenu
        disabled={disabled}
        icon={<Ellipsis />}
        id="more"
        label={t("library.more")}
      >
        <MoveSelect
          current={raccolta ?? undefined}
          label={
            raccolta === null ? t("library.addToLibrary") : t("library.moveTo")
          }
          onMove={move}
          raccolte={library.raccolte}
        />
        {raccolta === null ? null : (
          <Button onClick={trash} variant="outline">
            <Trash2 />
            {t("library.delete")}
          </Button>
        )}
      </PopoverMenu>
    </>
  );
}
