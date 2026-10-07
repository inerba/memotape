import { Check } from "lucide-react";
import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { errorText } from "@/features/status/status";
import { cn } from "@/lib/utils";
import { useSettings } from "./settings-context";
import type { SettingField } from "./settings-writer";

/** Quanto dura la spunta: è anche la durata della sua animazione (`animate-save-tick`). */
const TICK_MS = 1600;

/**
 * La spunta salvia accanto al valore appena salvato: compare e svanisce senza spostare nulla,
 * perché occupa sempre il suo posto. Il lettore di schermo sente "Salvato".
 */
export function SaveTick({
  name,
  label,
  className,
}: {
  name: SettingField;
  label: string;
  className?: string;
}) {
  const { t } = useTranslation();
  const { feedback, clearFeedback } = useSettings();
  const state = feedback[name];
  useEffect(() => {
    if (state?.status !== "saved") {
      return;
    }
    const timer = setTimeout(
      () => clearFeedback(name, state.revision),
      TICK_MS
    );
    return () => clearTimeout(timer);
  }, [clearFeedback, name, state]);
  return (
    <span
      className={cn(
        "inline-flex size-4 shrink-0 items-center justify-center",
        className
      )}
      role="status"
    >
      {state?.status === "saved" ? (
        <>
          <Check
            aria-hidden
            className="size-3.5 animate-save-tick text-play motion-reduce:animate-none"
            key={state.revision}
          />
          <span className="sr-only">
            {label}: {t("settings.save.saved")}
          </span>
        </>
      ) : null}
    </span>
  );
}

/**
 * L'errore di salvataggio di un campo, con Riprova; le risposte vecchie non sostituiscono la
 * scelta più recente. Il salvataggio riuscito lo dice `SaveTick`.
 */
export function SettingFeedback({
  name,
  label,
  disabled = false,
  className,
}: {
  name: SettingField;
  label: string;
  disabled?: boolean;
  className?: string;
}) {
  const { t } = useTranslation();
  const { feedback, retry } = useSettings();
  const state = feedback[name];
  const again = useCallback(() => {
    retry(name);
  }, [name, retry]);
  if (state?.status !== "error") {
    return null;
  }
  return (
    <div
      className={cn(
        "flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-xs",
        className
      )}
    >
      <p className="min-w-0 text-destructive leading-relaxed" role="alert">
        {t("settings.save.failed", { field: label })}{" "}
        {errorText(state.error, t)} {t("settings.save.restored")}
      </p>
      <Button
        aria-label={t("settings.save.retryField", { field: label })}
        className="h-7 px-2 text-xs"
        disabled={disabled}
        onClick={again}
        size="sm"
        variant="ghost"
      >
        {t("settings.save.retry")}
      </Button>
    </div>
  );
}
