import {
  ChevronRight,
  Copy,
  Ellipsis,
  FolderInput,
  FolderOpen,
  FolderSearch,
  Pencil,
  Trash2,
} from "lucide-react";
import {
  type KeyboardEvent,
  type MouseEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import type { TapeEntry } from "@/bindings";
import { closeMenu, openMenuAt, PopoverMenu } from "@/components/popover-menu";
import { azioniDelTape, raccoltaLabel } from "@/features/library/library";
import { MENU_ITEM, MenuItem } from "@/features/library/tape-actions";
import {
  type Azione,
  nomeTasti,
  tastoDellaTabella,
} from "@/features/shortcuts/shortcuts";

/**
 * Le operazioni su un Tape di un elenco (Libreria, Recenti), comuni a menu e tasti: ognuna riceve
 * il Tape, e Sposta in e Rinomina anche la Raccolta o il titolo nuovo.
 */
export interface TapeOperations {
  onCopy: (tape: TapeEntry) => void;
  onMove: (tape: TapeEntry, raccolta: string) => void;
  onOpen: (tape: TapeEntry) => void;
  onRename: (tape: TapeEntry, titolo: string) => void;
  onReveal: (tape: TapeEntry) => void;
  onTrash: (tape: TapeEntry) => void;
  /** Il Tape su cui lavora l'Attività in corso (Trascrivi su un Tape): niente Rinomina, Sposta in né Cestino. */
  tapeDellAttivita: string | null;
}

/** Il clic destro, il tasto Menu o Maiusc+F10 aprono il menu `id` lì dove sono. */
export function contextMenu(id: string) {
  return (e: MouseEvent) => {
    // Nel campo della rinomina resta il menu di Windows, con Taglia e Incolla.
    if ((e.target as Element).closest("input")) {
      return;
    }
    e.preventDefault();
    openMenuAt(id, e.clientX, e.clientY);
  };
}

/**
 * I tasti di `tastoDellaTabella` sull'elemento del Tape (non sui campi dentro): esegue l'azione
 * che c'è in `azioni`.
 */
export function tapeKeys(azioni: Partial<Record<Azione, () => void>>) {
  return (e: KeyboardEvent) => {
    if (e.target !== e.currentTarget) {
      return;
    }
    const azione = tastoDellaTabella(e);
    const esegui = azione ? azioni[azione] : undefined;
    if (esegui) {
      e.preventDefault();
      esegui();
    }
  };
}

/**
 * Lo stato e le azioni di una riga di un elenco di Tape (Libreria, Recenti): la rinomina sul posto,
 * Apri e Cestino, e se il Tape si può modificare. `ref` va sull'elemento con il focus, che lo
 * riprende quando la rinomina finisce con Invio o Esc (il campo sparisce e il focus resterebbe sul
 * documento); se si è usciti con un clic altrove, il focus resta lì.
 */
export function useTapeRow<T extends HTMLElement>(
  tape: TapeEntry,
  raccolte: string[],
  operations: TapeOperations
) {
  const [renaming, setRenaming] = useState(false);
  const ref = useRef<T>(null);
  const was = useRef(renaming);
  useEffect(() => {
    const active = document.activeElement;
    if (
      was.current &&
      !renaming &&
      (active === null || active === document.body)
    ) {
      ref.current?.focus();
    }
    was.current = renaming;
  }, [renaming]);
  const { modificabile } = azioniDelTape(
    tape,
    raccolte,
    operations.tapeDellAttivita
  );
  const { onOpen, onRename, onTrash } = operations;
  const open = useCallback(() => onOpen(tape), [tape, onOpen]);
  const startRename = useCallback(() => setRenaming(true), []);
  const cancelRename = useCallback(() => setRenaming(false), []);
  const submitRename = useCallback(
    (titolo: string) => {
      setRenaming(false);
      onRename(tape, titolo);
    },
    [onRename, tape]
  );
  const trash = useCallback(() => onTrash(tape), [tape, onTrash]);
  return {
    cancelRename,
    modificabile,
    open,
    ref,
    renaming,
    startRename,
    submitRename,
    trash,
  };
}

/**
 * Il menu di un Tape di un elenco: «…» lo apre sotto di sé, `contextMenu(id)` nel punto del clic.
 * Apri, Rinomina (`onRename`: il campo è di chi mostra il titolo), Sposta in ▸, Mostra in Esplora
 * file, Copia testo e Sposta nel Cestino, con la conferma.
 */
export function TapeContextMenu({
  className = "",
  id,
  onRename,
  operations,
  raccolte,
  tape,
  title,
}: {
  className?: string;
  id: string;
  onRename: () => void;
  operations: TapeOperations;
  raccolte: string[];
  tape: TapeEntry;
  /** Il titolo mostrato. */
  title: string;
}) {
  const { t } = useTranslation();
  const { destinazioni, modificabile } = azioniDelTape(
    tape,
    raccolte,
    operations.tapeDellAttivita
  );
  // Ogni voce chiude il menu, poi agisce sul Tape. Il contenuto è montato solo da aperto (`lazy`).
  const voce = (azione: (tape: TapeEntry) => void) => () => {
    closeMenu(id);
    azione(tape);
  };
  const { onMove } = operations;
  const move = useCallback(
    (raccolta: string) => {
      closeMenu(id);
      onMove(tape, raccolta);
    },
    [id, onMove, tape]
  );

  return (
    <PopoverMenu
      className={`size-8 text-muted-foreground ${className}`}
      icon={<Ellipsis />}
      id={id}
      label={t("library.actionsOf", { titolo: title })}
      lazy
      panelClassName="min-w-56"
      tabIndex={-1}
      variant="ghost"
    >
      <MenuItem
        icon={<FolderOpen />}
        kbd={nomeTasti(t, "apriTape")}
        onClick={voce(operations.onOpen)}
      >
        {t("library.open")}
      </MenuItem>
      <MenuItem
        disabled={!modificabile}
        icon={<Pencil />}
        kbd={nomeTasti(t, "rinominaTape")}
        onClick={voce(onRename)}
      >
        {t("library.renameTape")}
      </MenuItem>
      <PopoverMenu
        className={`${MENU_ITEM} justify-start font-normal has-[>svg]:px-2`}
        disabled={!modificabile || destinazioni.length === 0}
        icon={
          <>
            <FolderInput />
            {t("library.moveInto")}
            <ChevronRight className="ml-auto" />
          </>
        }
        id={`${id}-sposta`}
        label={t("library.moveInto")}
        lazy
        panelClassName="mt-0 -ml-1.5 min-w-48 [position-area:right_span-bottom]"
        size="sm"
        variant="ghost"
      >
        {destinazioni.map((raccolta) => (
          <MoveItem key={raccolta} onMove={move} raccolta={raccolta} />
        ))}
      </PopoverMenu>
      <MenuItem icon={<FolderSearch />} onClick={voce(operations.onReveal)}>
        {t("library.reveal")}
      </MenuItem>
      <MenuItem icon={<Copy />} onClick={voce(operations.onCopy)}>
        {t("transcription.copy")}
      </MenuItem>
      <hr className="my-1" />
      <MenuItem
        className="text-destructive [&_svg]:text-destructive"
        disabled={!modificabile}
        icon={<Trash2 />}
        kbd={nomeTasti(t, "cestinaTape")}
        onClick={voce(operations.onTrash)}
      >
        {t("library.deleteConfirm")}
      </MenuItem>
    </PopoverMenu>
  );
}

function MoveItem({
  onMove,
  raccolta,
}: {
  onMove: (raccolta: string) => void;
  raccolta: string;
}) {
  const { t } = useTranslation();
  const move = useCallback(() => onMove(raccolta), [onMove, raccolta]);
  return (
    <MenuItem icon={<FolderInput />} onClick={move}>
      <span className="truncate">{raccoltaLabel(raccolta, t)}</span>
    </MenuItem>
  );
}
