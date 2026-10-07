import type { ChangeEvent } from "react";
import { useTranslation } from "react-i18next";
import type { AudioDevice } from "@/bindings";
import { NativeSelect } from "@/components/native-select";
import { cn } from "@/lib/utils";
import { shortDeviceName } from "./devices";

/**
 * Un microfono o un dispositivo di uscita: il predefinito di sistema (`null`) o uno dei rilevati.
 * Un dispositivo salvato ma non collegato resta scelto, come "Non collegato", in mattone.
 * `devices` è `null` finché l'elenco non arriva. `compact` (il menu di Nuova registrazione) mostra
 * i nomi senza il tipo generico di Windows, con il nome intero nel tooltip.
 */
export function DeviceSelect({
  className,
  compact = false,
  devices,
  disabled,
  id,
  label,
  onChange,
  value,
}: {
  className?: string;
  compact?: boolean;
  devices: AudioDevice[] | null;
  disabled?: boolean;
  id?: string;
  label?: string;
  onChange: (e: ChangeEvent<HTMLSelectElement>) => void;
  value: string | null;
}) {
  const { t } = useTranslation();
  const fallback = devices?.find((d) => d.isDefault);
  const missing =
    devices !== null && value !== null && !devices.some((d) => d.id === value);
  const nameOf = (name: string) => (compact ? shortDeviceName(name) : name);
  let defaultText = t("settings.recording.defaultDevice");
  if (fallback) {
    defaultText = compact
      ? t("settings.recording.defaultDeviceShort", {
          name: nameOf(fallback.name),
        })
      : t("settings.recording.defaultDeviceNamed", { name: fallback.name });
  }
  const chosen = devices?.find((d) => d.id === value);
  let title = chosen?.name;
  if (missing) {
    title = t("settings.recording.missingDevice");
  } else if (!chosen && fallback) {
    title = t("settings.recording.defaultDeviceNamed", { name: fallback.name });
  }
  return (
    <NativeSelect
      aria-label={label}
      className={cn(
        "h-9",
        missing && "border-destructive/60 text-destructive",
        className
      )}
      disabled={disabled}
      id={id}
      onChange={onChange}
      title={title}
      value={value ?? ""}
    >
      <option value="">{defaultText}</option>
      {devices?.map((d) => (
        <option key={d.id} title={d.name} value={d.id}>
          {nameOf(d.name)}
        </option>
      ))}
      {missing ? (
        <option value={value ?? ""}>
          {t("settings.recording.missingDevice")}
        </option>
      ) : null}
    </NativeSelect>
  );
}
