import { useCallback, useId } from "react";
import type { AppError } from "@/bindings";
import { FieldHelp } from "@/components/field-help";
import { Checkbox } from "@/components/ui/checkbox";
import type { ParlantiRegistrazione } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { SettingFeedback } from "./setting-feedback";

/** Una casella che salva subito un'impostazione e mostra l'esito accanto al campo. */
export function SettingCheckbox({
  description,
  disabled,
  label,
  name,
  note,
  onError,
  inlineFeedback = true,
}: {
  description?: string;
  disabled?: boolean;
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
    async (checked: boolean | "indeterminate") => {
      const error = await save(
        (current) => ({
          ...current,
          [name]: checked === true,
        }),
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
      <div className="flex items-center gap-1">
        <label
          className="flex min-w-0 cursor-pointer items-center gap-2 text-sm has-[:disabled]:cursor-default has-[:disabled]:opacity-50"
          htmlFor={id}
        >
          <Checkbox
            aria-describedby={description ? `${id}-description` : undefined}
            checked={settings[name] ?? false}
            disabled={disabled}
            id={id}
            onCheckedChange={change}
          />
          {label}
        </label>
        {description || note ? (
          <FieldHelp label={label}>
            {description}
            {description && note ? " " : null}
            {note}
          </FieldHelp>
        ) : null}
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
