import { type ChangeEvent, useCallback, useId } from "react";
import { useTranslation } from "react-i18next";
import type { AppError, Sensibilita } from "@/bindings";
import { FieldHelp } from "@/components/field-help";
import { NativeSelect } from "@/components/native-select";
import { Checkbox } from "@/components/ui/checkbox";
import { SettingFeedback } from "./setting-feedback";
import { useSettings } from "./settings-context";

export function CleaningProfile({
  name,
  onError,
  compact = false,
  disabled = false,
  bypass = false,
  hideLegend = false,
  inlineFeedback = true,
}: {
  name: "audioMicrofono" | "audioSistema" | "audioFileMisto";
  onError: (error: AppError | null) => void;
  compact?: boolean;
  disabled?: boolean;
  bypass?: boolean;
  hideLegend?: boolean;
  inlineFeedback?: boolean;
}) {
  const { t } = useTranslation();
  const { settings, save } = useSettings();
  const choose = useCallback(
    async (checked: boolean | "indeterminate") => {
      const pulizia = checked === true;
      const error = await save(
        (current) => ({
          ...current,
          [name]: { ...current[name], pulizia },
        }),
        `${name}.pulizia`
      );
      if (!inlineFeedback) {
        onError(error);
      }
    },
    [inlineFeedback, name, onError, save]
  );
  const chooseSensitivity = useCallback(
    async (event: ChangeEvent<HTMLSelectElement>) => {
      const sensibilita = event.target.value as Sensibilita;
      const error = await save(
        (current) => ({
          ...current,
          [name]: { ...current[name], sensibilita },
        }),
        `${name}.sensibilita`
      );
      if (!inlineFeedback) {
        onError(error);
      }
    },
    [inlineFeedback, name, onError, save]
  );
  const sensitivity = settings[name]?.sensibilita ?? "bilanciato";
  const id = useId();
  const label =
    name === "audioFileMisto"
      ? "settings.cleaning.profile"
      : `settings.recording.inputs.${name === "audioMicrofono" ? "mic" : "system"}`;
  return (
    <fieldset
      className={
        compact
          ? "flex min-w-0 flex-wrap items-center gap-x-3 gap-y-2 disabled:opacity-50"
          : "flex flex-col gap-2 disabled:opacity-50"
      }
      disabled={disabled}
    >
      <legend
        className={
          compact || hideLegend ? "sr-only" : "mb-2 font-medium text-sm"
        }
      >
        {t(label)}
      </legend>
      <div className="flex min-w-0 flex-col gap-1">
        <div className="flex items-center gap-1">
          <label
            className="flex w-fit cursor-pointer items-center gap-2 text-sm"
            htmlFor={id}
          >
            <Checkbox
              aria-describedby={`${id}-description`}
              checked={settings[name]?.pulizia ?? false}
              disabled={disabled}
              id={id}
              name={`${name}.pulizia`}
              onCheckedChange={choose}
            />
            {t("settings.cleaning.enable")}
          </label>
          <FieldHelp label={t("settings.cleaning.enable")}>
            {t(
              name === "audioFileMisto"
                ? "settings.cleaning.description"
                : "settings.cleaning.recordingDescription"
            )}
          </FieldHelp>
        </div>
        {inlineFeedback ? (
          <SettingFeedback
            disabled={disabled}
            label={t("settings.cleaning.enable")}
            name={`${name}.pulizia`}
          />
        ) : null}
      </div>
      <div
        className={
          compact
            ? "flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1"
            : "flex min-w-0 flex-col gap-2"
        }
      >
        <div className="flex items-center gap-1">
          <label className="font-medium text-sm" htmlFor={`${id}-sensitivity`}>
            {t("settings.protection.label")}
          </label>
          <FieldHelp label={t("settings.protection.label")}>
            <p>{t(`settings.protection.descriptions.${sensitivity}`)}</p>
            <p className="mt-2">{t("settings.protection.scope")}</p>
          </FieldHelp>
        </div>
        <NativeSelect
          aria-describedby={`${id}-scope ${id}-tradeoff`}
          className={compact ? "h-7 text-xs" : "h-9"}
          disabled={disabled}
          id={`${id}-sensitivity`}
          name={`${name}.sensibilita`}
          onChange={chooseSensitivity}
          value={sensitivity}
        >
          {(["spento", "sensibile", "bilanciato", "selettivo"] as const).map(
            (level) => (
              <option key={level} value={level}>
                {t(`settings.protection.levels.${level}`)}
              </option>
            )
          )}
        </NativeSelect>
        {inlineFeedback ? (
          <SettingFeedback
            className={compact ? "basis-full" : undefined}
            disabled={disabled}
            label={t("settings.protection.label")}
            name={`${name}.sensibilita`}
          />
        ) : null}
      </div>
      <span className="sr-only" id={`${id}-description`}>
        {t(
          name === "audioFileMisto"
            ? "settings.cleaning.description"
            : "settings.cleaning.recordingDescription"
        )}
      </span>
      <span className="sr-only" id={`${id}-scope`}>
        {t("settings.protection.scope")}
      </span>
      <span className="sr-only" id={`${id}-tradeoff`}>
        {t(`settings.protection.descriptions.${sensitivity}`)}
      </span>
      {bypass ? (
        <p className="basis-full text-destructive text-xs" role="status">
          {t("recording.cleaningBypass")}
        </p>
      ) : null}
    </fieldset>
  );
}
