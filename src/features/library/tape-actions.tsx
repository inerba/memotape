import {
  Ellipsis,
  FileDown,
  FolderInput,
  FolderOpen,
  RefreshCcw,
  Trash2,
  UsersRound,
} from "lucide-react";
import { type ReactNode, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands, type LibraryList } from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { raccoltaLabel } from "@/features/library/library";
import { entryOf, titleOf } from "@/features/library/tape-header";

function closeMenu() {
  document.getElementById("menu-more")?.hidePopover();
}

export const MENU_ITEM =
  "flex h-8 w-full items-center gap-2.5 rounded-md px-2 text-left text-sm transition-colors hover:bg-accent focus-visible:bg-accent focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0 [&_svg]:text-muted-foreground";

/**
 * "…" del Tape aperto, con le azioni meno frequenti: Esporta Markdown…, Mostra in Esplora file,
 * Sposta in… (o Aggiungi alla Libreria… per un Tape da fuori), Trascrivi di nuovo (se `onTranscribe`,
 * apre il dialog delle scelte) ed Elimina.
 */
export function TapeMenu({
  disabled,
  diarized,
  diarizingDisabled,
  library,
  onDiarize,
  onError,
  onExported,
  onMove,
  onReveal,
  onTranscribe,
  onTrash,
  path,
}: {
  /** Ci lavora l'Attività in corso: niente spostamento, Trascrivi né Cestino. */
  disabled: boolean;
  diarized: boolean;
  diarizingDisabled: boolean;
  library: LibraryList;
  onDiarize?: () => void;
  onError: (error: AppError) => void;
  onExported: (path: string) => void;
  onMove: (path: string, raccolta: string) => void;
  onReveal: (path: string) => void;
  /** Trascrivi di nuovo; assente se il Tape non è la Sorgente o un'Attività è in corso. */
  onTranscribe?: () => void;
  onTrash: (tape: { path: string; titolo: string }) => void;
  path: string;
}) {
  const { t } = useTranslation();
  const entry = entryOf(library, path);
  // `null`: fuori dalla Libreria.
  const raccolta = entry ? (entry.raccolta ?? "") : null;
  // Dove si può spostare: Senza raccolta e le Raccolte, tranne dove sta già.
  const targets = ["", ...library.raccolte].filter((r) => r !== raccolta);

  const exportMarkdown = useCallback(async () => {
    closeMenu();
    const result = await commands.exportMarkdown(path);
    if (result.status === "error") {
      onError(result.error);
    } else if (result.data) {
      onExported(result.data);
    }
  }, [onError, onExported, path]);
  const reveal = useCallback(() => {
    closeMenu();
    onReveal(path);
  }, [onReveal, path]);
  const move = useCallback(
    (to: string) => {
      closeMenu();
      onMove(path, to);
    },
    [onMove, path]
  );
  const transcribe = useCallback(() => {
    closeMenu();
    onTranscribe?.();
  }, [onTranscribe]);
  const diarize = useCallback(() => {
    closeMenu();
    onDiarize?.();
  }, [onDiarize]);
  const trash = useCallback(() => {
    closeMenu();
    onTrash({ path, titolo: titleOf(library, path) });
  }, [library, onTrash, path]);

  return (
    <PopoverMenu
      className="size-8"
      icon={<Ellipsis />}
      id="more"
      label={t("library.more")}
      variant="ghost"
    >
      <MenuItem icon={<FileDown />} onClick={exportMarkdown}>
        {t("transcription.export")}
      </MenuItem>
      <MenuItem icon={<FolderOpen />} onClick={reveal}>
        {t("library.reveal")}
      </MenuItem>
      {targets.length > 0 ? (
        <>
          <hr className="my-1" />
          <p className="px-2 pt-1 pb-0.5 text-muted-foreground text-xs">
            {raccolta === null
              ? t("library.addToLibrary")
              : t("library.moveTo")}
          </p>
          {targets.map((target) => (
            <MoveItem
              disabled={disabled}
              key={target}
              onMove={move}
              target={target}
            />
          ))}
        </>
      ) : null}
      {onTranscribe ? (
        <>
          <hr className="my-1" />
          <MenuItem icon={<RefreshCcw />} onClick={transcribe}>
            {t("transcription.again")}
          </MenuItem>
        </>
      ) : null}
      {onDiarize ? (
        <MenuItem
          disabled={diarizingDisabled}
          icon={<UsersRound aria-hidden="true" />}
          onClick={diarize}
        >
          {t(diarized ? "diarization.again" : "diarization.start")}
        </MenuItem>
      ) : null}
      {raccolta === null ? null : (
        <>
          <hr className="my-1" />
          <MenuItem
            className="text-destructive [&_svg]:text-destructive"
            disabled={disabled}
            icon={<Trash2 />}
            onClick={trash}
          >
            {t("library.delete")}
          </MenuItem>
        </>
      )}
    </PopoverMenu>
  );
}

/** Una Raccolta di destinazione di Sposta in…. */
function MoveItem({
  disabled,
  onMove,
  target,
}: {
  disabled: boolean;
  onMove: (raccolta: string) => void;
  target: string;
}) {
  const { t } = useTranslation();
  const move = useCallback(() => onMove(target), [onMove, target]);
  return (
    <MenuItem disabled={disabled} icon={<FolderInput />} onClick={move}>
      <span className="truncate">{raccoltaLabel(target, t)}</span>
    </MenuItem>
  );
}

export function MenuItem({
  children,
  className = "",
  disabled,
  icon,
  kbd,
  onClick,
}: {
  children: ReactNode;
  className?: string;
  disabled?: boolean;
  icon: ReactNode;
  /** Il tasto che fa la stessa cosa, a destra. */
  kbd?: string;
  onClick: () => void;
}) {
  return (
    <button
      className={`${MENU_ITEM} ${className}`}
      disabled={disabled}
      onClick={onClick}
      type="button"
    >
      {icon}
      {children}
      {kbd ? (
        <kbd className="ml-auto pl-4 font-sans text-muted-foreground text-xs">
          {kbd}
        </kbd>
      ) : null}
    </button>
  );
}
