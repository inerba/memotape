import { type LucideIcon, Mic, Speaker } from "lucide-react";
import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { AppError } from "@/bindings";
import { Checkbox } from "@/components/ui/checkbox";
import { type RecordingInput, withInput } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";

const INPUTS = [
  { icon: Mic, input: "mic" },
  { icon: Speaker, input: "system" },
] as const satisfies { icon: LucideIcon; input: RecordingInput }[];

/**
 * "Registra da": una casella con l'icona per Microfono e Audio di sistema, che salvano subito la
 * sorgente di registrazione. In Impostazioni sono riquadri affiancati, nel menu di Nuova
 * registrazione (`compact`) righe come le altre caselle.
 */
export function RecordingInputs({
  compact = false,
  onError,
}: {
  compact?: boolean;
  onError: (error: AppError) => void;
}) {
  const { save, settings } = useSettings();
  const toggle = useCallback(
    async (input: RecordingInput, on: boolean) => {
      const recordingSource = withInput(settings.recordingSource, input, on);
      if (recordingSource === settings.recordingSource) {
        return;
      }
      const error = await save({ ...settings, recordingSource });
      if (error) {
        onError(error);
      }
    },
    [onError, save, settings]
  );
  return (
    <div className={compact ? "flex flex-col gap-2" : "grid grid-cols-2 gap-4"}>
      {INPUTS.map(({ icon, input }) => (
        <InputChoice
          checked={
            settings.recordingSource === "both" ||
            settings.recordingSource === input
          }
          compact={compact}
          icon={icon}
          input={input}
          key={input}
          onToggle={toggle}
        />
      ))}
    </div>
  );
}

function InputChoice({
  checked,
  compact,
  icon: Icon,
  input,
  onToggle,
}: {
  checked: boolean;
  compact: boolean;
  icon: LucideIcon;
  input: RecordingInput;
  onToggle: (input: RecordingInput, on: boolean) => void;
}) {
  const { t } = useTranslation();
  const id = `${compact ? "menu" : "settings"}-input-${input}`;
  const toggle = useCallback(
    (state: boolean | "indeterminate") => onToggle(input, state === true),
    [input, onToggle]
  );
  const label = t(`settings.recording.inputs.${input}`);
  if (compact) {
    return (
      <label
        className="flex cursor-pointer items-center gap-2 text-sm"
        htmlFor={id}
      >
        <Checkbox checked={checked} id={id} onCheckedChange={toggle} />
        <Icon className="size-4 shrink-0 text-muted-foreground" />
        {label}
      </label>
    );
  }
  return (
    <label
      className="flex h-14 cursor-pointer items-center gap-3 rounded-lg border px-3 text-sm transition-colors hover:bg-accent has-data-[state=checked]:border-foreground/60"
      htmlFor={id}
    >
      <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-accent text-foreground/80">
        <Icon className="size-4" />
      </span>
      <span className="min-w-0 flex-1 truncate font-medium">{label}</span>
      <Checkbox checked={checked} id={id} onCheckedChange={toggle} />
    </label>
  );
}
