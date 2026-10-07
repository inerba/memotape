import { type LucideIcon, Mic, Speaker } from "lucide-react";
import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { AppError } from "@/bindings";
import { Switch } from "@/components/ui/switch";
import { type RecordingInput, withInput } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { SettingFeedback } from "./setting-feedback";
import { SWITCH_CLASS } from "./setting-switch";

const INPUTS = [
  { icon: Mic, input: "mic" },
  { icon: Speaker, input: "system" },
] as const satisfies { icon: LucideIcon; input: RecordingInput }[];

/**
 * "Registra da": un interruttore con l'icona per Microfono e Audio di sistema, in riquadri
 * affiancati, che salvano subito la sorgente di registrazione.
 */
export function RecordingInputs({
  onError,
  inlineFeedback = true,
}: {
  onError: (error: AppError) => void;
  inlineFeedback?: boolean;
}) {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  const toggle = useCallback(
    async (input: RecordingInput, on: boolean) => {
      const error = await save(
        (current) => ({
          ...current,
          recordingSource: withInput(current.recordingSource, input, on),
        }),
        "recordingSource"
      );
      if (error && !inlineFeedback) {
        onError(error);
      }
    },
    [inlineFeedback, onError, save]
  );
  return (
    <div className="flex min-w-0 flex-col gap-1">
      <div className="grid grid-cols-2 gap-4">
        {INPUTS.map(({ icon, input }) => (
          <InputChoice
            checked={
              settings.recordingSource === "both" ||
              settings.recordingSource === input
            }
            icon={icon}
            input={input}
            key={input}
            onToggle={toggle}
          />
        ))}
      </div>
      {inlineFeedback ? (
        <SettingFeedback
          label={t("settings.recording.input")}
          name="recordingSource"
        />
      ) : null}
    </div>
  );
}

function InputChoice({
  checked,
  icon: Icon,
  input,
  onToggle,
}: {
  checked: boolean;
  icon: LucideIcon;
  input: RecordingInput;
  onToggle: (input: RecordingInput, on: boolean) => void;
}) {
  const { t } = useTranslation();
  const id = `settings-input-${input}`;
  const toggle = useCallback(
    (state: boolean) => onToggle(input, state),
    [input, onToggle]
  );
  const label = t(`settings.recording.inputs.${input}`);
  return (
    <label
      className="flex h-14 cursor-pointer items-center gap-3 rounded-lg border px-3 text-sm transition-colors hover:bg-accent has-data-[state=checked]:border-foreground/60"
      htmlFor={id}
    >
      <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-accent text-foreground/80">
        <Icon className="size-4" />
      </span>
      <span className="min-w-0 flex-1 truncate font-medium">{label}</span>
      <Switch
        checked={checked}
        className={SWITCH_CLASS}
        id={id}
        onCheckedChange={toggle}
      />
    </label>
  );
}
