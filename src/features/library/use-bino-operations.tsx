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

/** La risposta di un comando che restituisce il percorso nuovo del Bino. */
type Moved =
  | { status: "ok"; data: string }
  | { status: "error"; error: AppError };

interface Trashing {
  path: string;
  titolo: string;
}

/**
 * Rinomina, Sposta in…, Mostra in Esplora file ed Elimina (con la conferma in `dialog`) di un Bino.
 * `onMoved` riceve il percorso vecchio e quello nuovo, `onTrashed` quello del Bino nel Cestino; gli
 * errori vanno a `onError`.
 */
export function useBinoOperations({
  onError,
  onMoved,
  onTrashed,
}: {
  onError: (error: AppError) => void;
  onMoved: (from: string, to: string) => void;
  onTrashed: (path: string) => void;
}): {
  dialog: ReactNode;
  moveBino: (path: string, raccolta: string) => void;
  renameBino: (path: string, titolo: string) => void;
  requestTrash: (bino: Trashing) => void;
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
  const renameBino = useCallback(
    (path: string, titolo: string) =>
      moved(path, commands.renameBino(path, titolo)),
    [moved]
  );
  const moveBino = useCallback(
    (path: string, raccolta: string) =>
      moved(path, commands.moveBino(path, raccolta)),
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
    const result = await commands.trashBino(trashing.path);
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
  return { dialog, moveBino, renameBino, requestTrash: setTrashing, reveal };
}
