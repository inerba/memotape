import { type ChangeEvent, useCallback } from "react";
import type { AppError } from "@/bindings";
import { NativeSelect } from "@/components/native-select";
import { GUADAGNI, guadagnoText } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { cn } from "@/lib/utils";

/**
 * La tendina del Guadagno di un Ingresso, che salva subito: in Impostazioni e accanto al suo
 * indicatore durante la Registrazione, dove Rust lo applica all'audio da quel momento.
 */
export function GuadagnoSelect({
  className,
  name,
  onError,
  ...props
}: {
  "aria-label"?: string;
  className?: string;
  disabled?: boolean;
  id?: string;
  name: "guadagnoMicrofono" | "guadagnoSistema";
  onError: (error: AppError) => void;
}) {
  const { save, settings } = useSettings();
  const choose = useCallback(
    async (e: ChangeEvent<HTMLSelectElement>) => {
      const error = await save({ ...settings, [name]: Number(e.target.value) });
      if (error) {
        onError(error);
      }
    },
    [name, onError, save, settings]
  );
  return (
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
  );
}
