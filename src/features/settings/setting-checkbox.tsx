import { useCallback, useId } from "react";
import type { AppError } from "@/bindings";
import { Checkbox } from "@/components/ui/checkbox";
import type { ParlantiRegistrazione } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";

/** Una casella che salva subito un'impostazione; un errore va a `onError`. */
export function SettingCheckbox({
  disabled,
  label,
  name,
  note,
  onError,
}: {
  disabled?: boolean;
  label: string;
  name:
    | "assistenti"
    | "parlantiFile"
    | "trascrizioneDalVivo"
    | ParlantiRegistrazione;
  note?: string;
  onError: (error: AppError) => void;
}) {
  const id = useId();
  const { save, settings } = useSettings();
  const change = useCallback(
    async (checked: boolean | "indeterminate") => {
      const error = await save({ ...settings, [name]: checked === true });
      if (error) {
        onError(error);
      }
    },
    [name, onError, save, settings]
  );
  return (
    <label
      className="flex shrink-0 cursor-pointer items-center gap-2 text-sm has-[:disabled]:cursor-default has-[:disabled]:opacity-50"
      htmlFor={id}
      title={note}
    >
      <Checkbox
        checked={settings[name] ?? false}
        disabled={disabled}
        id={id}
        onCheckedChange={change}
      />
      {label}
    </label>
  );
}
