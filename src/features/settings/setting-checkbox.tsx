import { type ChangeEvent, useCallback } from "react";
import type { AppError } from "@/bindings";
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
  name: "parlantiFile" | "trascrizioneDalVivo" | ParlantiRegistrazione;
  note?: string;
  onError: (error: AppError) => void;
}) {
  const { save, settings } = useSettings();
  const change = useCallback(
    async (e: ChangeEvent<HTMLInputElement>) => {
      const error = await save({ ...settings, [name]: e.target.checked });
      if (error) {
        onError(error);
      }
    },
    [name, onError, save, settings]
  );
  return (
    <label
      className="flex shrink-0 items-center gap-2 text-sm has-[:disabled]:opacity-50"
      title={note}
    >
      <input
        checked={settings[name] ?? false}
        className="size-4 accent-primary"
        disabled={disabled}
        onChange={change}
        type="checkbox"
      />
      {label}
    </label>
  );
}
