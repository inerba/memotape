import { useCallback, useId } from "react";
import type { AppError } from "@/bindings";
import { FieldHelp } from "@/components/field-help";
import { Switch } from "@/components/ui/switch";
import type { ParlantiRegistrazione } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { cn } from "@/lib/utils";
import { SaveTick, SettingFeedback } from "./setting-feedback";

/** L'interruttore acceso è salvia, come Segui l'audio; spento resta il filo dei campi. */
export const SWITCH_CLASS = "data-[state=checked]:bg-play";

/**
 * Un interruttore che salva subito un'impostazione e mostra l'esito accanto al campo. `end` mette
 * l'interruttore a destra dell'etichetta, come nelle righe dei menu.
 */
export function SettingSwitch({
  description,
  disabled,
  end = false,
  label,
  name,
  note,
  onError,
  inlineFeedback = true,
}: {
  description?: string;
  disabled?: boolean;
  end?: boolean;
  label: string;
  name:
    | "assistenti"
    | "parlantiFile"
    | "trascrizioneDalVivo"
    | ParlantiRegistrazione;
  note?: string;
  onError: (error: AppError) => void;
  inlineFeedback?: boolean;
}) {
  const id = useId();
  const { save, settings } = useSettings();
  const change = useCallback(
    async (checked: boolean) => {
      const error = await save(
        (current) => ({
          ...current,
          [name]: checked,
        }),
        name
      );
      if (error && !inlineFeedback) {
        onError(error);
      }
    },
    [inlineFeedback, name, onError, save]
  );
  const start = !end;
  const control = (
    <Switch
      aria-describedby={description ? `${id}-description` : undefined}
      checked={settings[name] ?? false}
      className={SWITCH_CLASS}
      disabled={disabled}
      id={id}
      onCheckedChange={change}
    />
  );
  return (
    <div className="flex min-w-0 flex-col gap-1">
      <div className="flex items-center gap-1">
        <label
          className={cn(
            "flex min-w-0 cursor-pointer items-center gap-2 text-sm has-[:disabled]:cursor-default has-[:disabled]:opacity-50",
            end && "flex-1"
          )}
          htmlFor={id}
        >
          {start ? control : null}
          <span className={cn("min-w-0", end && "flex-1")}>{label}</span>
        </label>
        {description || note ? (
          <FieldHelp label={label}>
            {description}
            {description && note ? " " : null}
            {note}
          </FieldHelp>
        ) : null}
        <SaveTick label={label} name={name} />
        {end ? control : null}
        {description ? (
          <span className="sr-only" id={`${id}-description`}>
            {description}
          </span>
        ) : null}
      </div>
      {inlineFeedback ? (
        <SettingFeedback disabled={disabled} label={label} name={name} />
      ) : null}
    </div>
  );
}
