import { type ReactNode, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands } from "@/bindings";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

/** La risposta di un comando che restituisce il percorso nuovo del Tape. */
type Moved =
  | { status: "ok"; data: string }
  | { status: "error"; error: AppError };

interface Trashing {
  path: string;
  titolo: string;
}

/**
 * Rinomina, Sposta in…, Mostra in Esplora file ed Elimina (con la conferma in `dialog`) di un Tape.
 * `onMoved` riceve il percorso vecchio e quello nuovo, `onTrashed` quello del Tape nel Cestino; gli
 * errori vanno a `onError`.
 */
export function useTapeOperations({
  onError,
  onMoved,
  onTrashed,
}: {
  onError: (error: AppError) => void;
  onMoved: (from: string, to: string) => void;
  onTrashed: (path: string) => void;
}): {
  dialog: ReactNode;
  moveTape: (path: string, raccolta: string) => void;
  renameTape: (path: string, titolo: string) => void;
  requestTrash: (tape: Trashing) => void;
  reveal: (path: string) => void;
} {
  const { t } = useTranslation();
  const [trashing, setTrashing] = useState<Trashing | null>(null);

  const moved = useCallback(
    async (path: string, result: Promise<Moved>) => {
      const done = await result;
      if (done.status === "error") {
        onError(done.error);
      } else {
        onMoved(path, done.data);
      }
    },
    [onError, onMoved]
  );
  const renameTape = useCallback(
    (path: string, titolo: string) =>
      moved(path, commands.renameTape(path, titolo)),
    [moved]
  );
  const moveTape = useCallback(
    (path: string, raccolta: string) =>
      moved(path, commands.moveTape(path, raccolta)),
    [moved]
  );
  const reveal = useCallback(
    async (path: string) => {
      const result = await commands.openSource(path);
      if (result.status === "error") {
        onError(result.error);
      }
    },
    [onError]
  );
  const close = useCallback((opened: boolean) => {
    if (!opened) {
      setTrashing(null);
    }
  }, []);
  const trash = useCallback(async () => {
    if (!trashing) {
      return;
    }
    const result = await commands.trashTape(trashing.path);
    if (result.status === "error") {
      onError(result.error);
    } else {
      onTrashed(trashing.path);
    }
  }, [onError, onTrashed, trashing]);

  const dialog = (
    <AlertDialog onOpenChange={close} open={trashing !== null}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("library.deleteTitle")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t("library.deleteDescription", { titolo: trashing?.titolo })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t("library.deleteKeep")}</AlertDialogCancel>
          <AlertDialogAction onClick={trash}>
            {t("library.deleteConfirm")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
  return { dialog, moveTape, renameTape, requestTrash: setTrashing, reveal };
}
