import { type ChangeEvent, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { AppError } from "@/bindings";
import { NativeSelect } from "@/components/native-select";
import { GUADAGNI, guadagnoText } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { cn } from "@/lib/utils";
import { SaveTick, SettingFeedback } from "./setting-feedback";

/**
 * La tendina del Guadagno di un Ingresso, che salva subito: in Impostazioni e accanto al suo
 * indicatore durante la Registrazione, dove Rust lo applica all'audio da quel momento.
 */
export function GuadagnoSelect({
  className,
  name,
  onError,
  inlineFeedback = true,
  ...props
}: {
  "aria-describedby"?: string;
  "aria-label"?: string;
  className?: string;
  disabled?: boolean;
  id?: string;
  name: "guadagnoMicrofono" | "guadagnoSistema";
  onError: (error: AppError) => void;
  inlineFeedback?: boolean;
}) {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  const choose = useCallback(
    async (e: ChangeEvent<HTMLSelectElement>) => {
      const value = Number(e.target.value);
      const error = await save(
        (current) => ({ ...current, [name]: value }),
        name
      );
      if (error && !inlineFeedback) {
        onError(error);
      }
    },
    [inlineFeedback, name, onError, save]
  );
  return (
    <div className="flex min-w-0 flex-col gap-1">
      <div className="flex min-w-0 items-center gap-1">
        <NativeSelect
          className={cn("tabular-nums", className)}
          onChange={choose}
          value={settings[name] ?? 0}
          {...props}
        >
          {GUADAGNI.map((db) => (
            <option key={db} value={db}>
              {guadagnoText(db)}
            </option>
          ))}
        </NativeSelect>
        <SaveTick
          label={props["aria-label"] ?? t("settings.recording.guadagno")}
          name={name}
        />
      </div>
      {inlineFeedback ? (
        <SettingFeedback
          disabled={props.disabled}
          label={props["aria-label"] ?? t("settings.recording.guadagno")}
          name={name}
        />
      ) : null}
    </div>
  );
}
