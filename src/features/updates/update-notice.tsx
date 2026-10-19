import { CircleArrowDown, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands, events, type UpdateInfo } from "@/bindings";
import { errorText } from "@/features/status/status";

const LINK =
  "-my-0.5 inline-flex min-h-6 items-center whitespace-nowrap rounded-sm font-medium underline underline-offset-4 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:cursor-default disabled:no-underline disabled:opacity-50";

type Fase =
  | { kind: "pronto" }
  | { kind: "download"; percent: number }
  | { kind: "fallito"; error: AppError };

/**
 * L'Aggiornamento disponibile, in fondo alla barra laterale (ADR-0036): Scarica e installa scarica
 * con l'avanzamento e avvia l'installer, che chiude l'app e la riapre aggiornata. Durante
 * un'Attività aspetta; prima di installare passa da `onProteggi` per le bozze delle Note. Se non
 * riesce, Riprova o la pagina della release per scaricare a mano. Resta finché non si chiude.
 */
export function UpdateNotice({
  busy,
  onDismiss,
  onProteggi,
  update,
}: {
  busy: boolean;
  onDismiss: () => void;
  onProteggi: (vai: () => void) => void;
  update: UpdateInfo;
}) {
  const { t } = useTranslation();
  const [fase, setFase] = useState<Fase>({ kind: "pronto" });
  const scaricando = fase.kind === "download";

  useEffect(() => {
    if (!scaricando) {
      return;
    }
    const unlisten = events.updateProgress.listen(({ payload }) =>
      setFase({ kind: "download", percent: payload.percent })
    );
    return () => {
      unlisten.then((stop) => stop());
    };
  }, [scaricando]);

  // Se riesce l'app si chiude: si torna qui solo con un errore.
  const install = useCallback(
    () =>
      onProteggi(async () => {
        setFase({ kind: "download", percent: 0 });
        const result = await commands.installUpdate();
        if (result.status === "error") {
          setFase({ error: result.error, kind: "fallito" });
        }
      }),
    [onProteggi]
  );
  const openPage = useCallback(
    () => commands.openUpdate(update.url),
    [update.url]
  );

  let testo = t("status.updateAvailable", { version: update.version });
  if (fase.kind === "download") {
    testo =
      fase.percent < 100
        ? t("status.updateDownloading", { percent: fase.percent })
        : t("status.updateInstalling");
  } else if (fase.kind === "fallito") {
    testo = errorText(fase.error, t);
  }

  return (
    <div className="mx-3 mb-2 flex items-start gap-2 rounded-lg border border-input bg-card py-2 pr-2 pl-3 text-sm">
      <CircleArrowDown className="mt-[3px] size-4 shrink-0 text-muted-foreground" />
      <div className="flex min-w-0 flex-1 flex-col gap-1 leading-relaxed">
        <p role="status">{testo}</p>
        {scaricando ? null : (
          <p className="flex flex-wrap gap-x-3">
            <button
              className={LINK}
              disabled={busy}
              onClick={install}
              title={busy ? t("status.updateBusy") : undefined}
              type="button"
            >
              {t(
                fase.kind === "fallito"
                  ? "status.updateRetry"
                  : "status.updateInstall"
              )}
            </button>
            <button className={LINK} onClick={openPage} type="button">
              {t(
                fase.kind === "fallito"
                  ? "status.updateManual"
                  : "status.updateNews"
              )}
            </button>
          </p>
        )}
      </div>
      {scaricando ? null : (
        <button
          aria-label={t("app.dismiss")}
          className="flex size-6 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
          onClick={onDismiss}
          title={t("app.dismiss")}
          type="button"
        >
          <X className="size-4" />
        </button>
      )}
    </div>
  );
}
