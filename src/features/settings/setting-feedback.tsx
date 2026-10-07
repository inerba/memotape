import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { errorText } from "@/features/status/status";
import { cn } from "@/lib/utils";
import { useSettings } from "./settings-context";
import type { SettingField } from "./settings-writer";

/** Un esito per campo: le risposte vecchie non sostituiscono la scelta più recente. */
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
  const { feedback, retry, clearFeedback } = useSettings();
  const state = feedback[name];
  useEffect(() => {
    if (state?.status !== "saved") {
      return;
    }
    const timer = setTimeout(() => clearFeedback(name, state.revision), 2000);
    return () => clearTimeout(timer);
  }, [clearFeedback, name, state]);
  const again = useCallback(() => {
    retry(name);
  }, [name, retry]);
  if (!state) {
    return null;
  }
  if (state.status === "error") {
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
  return (
    <p className={cn("text-muted-foreground text-xs", className)} role="status">
      <span className="sr-only">{label}: </span>
      {t(
        state.status === "saving"
          ? "settings.save.saving"
          : "settings.save.saved"
      )}
    </p>
  );
}
