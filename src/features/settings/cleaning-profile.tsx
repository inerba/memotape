import { useCallback, useId } from "react";
import { useTranslation } from "react-i18next";
import type { AppError, Sensibilita } from "@/bindings";
import { FieldHelp } from "@/components/field-help";
import { Segmented } from "@/components/segmented";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import { SaveTick, SettingFeedback } from "./setting-feedback";
import { SWITCH_CLASS } from "./setting-switch";
import { useSettings } from "./settings-context";

const LEVELS = ["spento", "sensibile", "bilanciato", "selettivo"] as const;

/** Il nome di ogni Profilo audio e la spiegazione della sua pulizia. */
const PROFILES = {
  audioFileMisto: {
    description: "settings.cleaning.description",
    legend: "settings.cleaning.profile",
  },
  audioMicrofono: {
    description: "settings.cleaning.recordingDescription",
    legend: "settings.recording.inputs.mic",
  },
  audioSistema: {
    description: "settings.cleaning.recordingDescription",
    legend: "settings.recording.inputs.system",
  },
} as const;

/** Le classi del gruppo e della Sensibilità per ogni disposizione. */
const LAYOUTS = {
  bar: {
    fieldset: "flex flex-wrap items-center gap-x-5 gap-y-2",
    sensitivity: "flex-wrap items-center gap-x-2 gap-y-1",
    size: "sm",
  },
  menu: {
    fieldset: "flex flex-col gap-1",
    sensitivity: "flex-col gap-1.5",
    size: "md",
  },
  settings: {
    fieldset: "flex flex-col gap-4",
    sensitivity: "flex-col gap-1.5",
    size: "lg",
  },
} as const;

/**
 * Filtra rumore e Sensibilità di un Profilo audio, che salvano subito. `layout`: in Impostazioni
 * uno sotto l'altro; nella barra della Registrazione su una riga; nel menu di Nuova registrazione
 * come righe con l'interruttore a destra e la Sensibilità su tutta la larghezza.
 */
export function CleaningProfile({
  name,
  onError,
  layout = "settings",
  disabled = false,
  bypass = false,
  hideLegend = false,
  inlineFeedback = true,
}: {
  name: "audioMicrofono" | "audioSistema" | "audioFileMisto";
  onError: (error: AppError | null) => void;
  layout?: "settings" | "bar" | "menu";
  disabled?: boolean;
  bypass?: boolean;
  hideLegend?: boolean;
  inlineFeedback?: boolean;
}) {
  const { t } = useTranslation();
  const { settings, save } = useSettings();
  const choose = useCallback(
    async (pulizia: boolean) => {
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
    async (sensibilita: Sensibilita) => {
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
  const { description, legend } = PROFILES[name];
  const bar = layout === "bar";
  const menu = layout === "menu";
  // Fuori dal menu l'interruttore precede l'etichetta e ha la sua spiegazione.
  const inline = !menu;
  const enable = t("settings.cleaning.enable");
  const sensitivityLabel = t("settings.protection.label");
  const cleaningSwitch = (
    <Switch
      aria-describedby={`${id}-description`}
      checked={settings[name]?.pulizia ?? false}
      className={SWITCH_CLASS}
      data-setting={`${name}.pulizia`}
      disabled={disabled}
      id={id}
      onCheckedChange={choose}
      size={bar ? "sm" : "default"}
    />
  );
  return (
    <fieldset
      className={cn("min-w-0 disabled:opacity-50", LAYOUTS[layout].fieldset)}
      disabled={disabled}
    >
      <legend
        className={
          layout === "settings" && !hideLegend
            ? "mb-2 font-medium text-sm"
            : "sr-only"
        }
      >
        {t(legend)}
      </legend>
      <div className="flex min-w-0 flex-col gap-1">
        <div className={cn("flex items-center gap-1", menu && "min-h-8")}>
          <label
            className={cn(
              "flex w-fit cursor-pointer items-center gap-2 text-sm",
              menu && "w-auto flex-1"
            )}
            htmlFor={id}
          >
            {inline ? cleaningSwitch : null}
            {enable}
          </label>
          {inline ? (
            <FieldHelp label={enable}>{t(description)}</FieldHelp>
          ) : null}
          <SaveTick label={enable} name={`${name}.pulizia`} />
          {menu ? cleaningSwitch : null}
        </div>
        {inlineFeedback ? (
          <SettingFeedback
            disabled={disabled}
            label={enable}
            name={`${name}.pulizia`}
          />
        ) : null}
      </div>
      <div className={cn("flex min-w-0", LAYOUTS[layout].sensitivity)}>
        <div className="flex items-center gap-1">
          <span className="text-sm" id={`${id}-sensitivity`}>
            {sensitivityLabel}
          </span>
          {menu ? null : (
            <FieldHelp label={t("settings.protection.fullLabel")}>
              <p>{t(`settings.protection.descriptions.${sensitivity}`)}</p>
              <p className="mt-2">{t("settings.protection.scope")}</p>
            </FieldHelp>
          )}
          {bar ? null : (
            <SaveTick label={sensitivityLabel} name={`${name}.sensibilita`} />
          )}
        </div>
        <Segmented
          className={layout === "settings" ? "w-fit" : undefined}
          describedBy={`${id}-sensitivity ${id}-tradeoff`}
          disabled={disabled}
          fill={menu}
          label={t("settings.protection.fullLabel")}
          name={`${name}.sensibilita`}
          onChange={chooseSensitivity}
          options={LEVELS.map((level) => ({
            label: t(`settings.protection.levels.${level}`),
            short: t(`settings.protection.short.${level}`),
            value: level,
          }))}
          size={LAYOUTS[layout].size}
          value={sensitivity}
        />
        {bar ? (
          <SaveTick label={sensitivityLabel} name={`${name}.sensibilita`} />
        ) : null}
        {inlineFeedback ? (
          <SettingFeedback
            className={bar ? "basis-full" : undefined}
            disabled={disabled}
            label={sensitivityLabel}
            name={`${name}.sensibilita`}
          />
        ) : null}
      </div>
      <span className="sr-only" id={`${id}-description`}>
        {t(description)}
      </span>
      <span className="sr-only" id={`${id}-tradeoff`}>
        {t(`settings.protection.descriptions.${sensitivity}`)}{" "}
        {t("settings.protection.scope")}
      </span>
      {bypass ? (
        <p className="basis-full text-destructive text-xs" role="status">
          {t("recording.cleaningBypass")}
        </p>
      ) : null}
    </fieldset>
  );
}
