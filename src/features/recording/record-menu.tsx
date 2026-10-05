import { ChevronDown, Mic } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AppError } from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { Button } from "@/components/ui/button";
import { RecordingInputs } from "@/features/settings/recording-inputs";
import { SettingCheckbox } from "@/features/settings/setting-checkbox";
import {
  PARLANTI_LABELS,
  parlantiRegistrazione,
} from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";

/**
 * Nuova registrazione ▾, l'azione principale della barra laterale: il menu sceglie da cosa registrare,
 * Trascrivi dal vivo e Riconosci i parlanti della Registrazione, che vale solo dal vivo.
 */
export function RecordMenu({
  disabled,
  onError,
  onRecord,
}: {
  disabled: boolean;
  onError: (error: AppError) => void;
  onRecord: () => void;
}) {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const live = settings.trascrizioneDalVivo ?? false;
  return (
    <div className="flex">
      <Button
        className="h-10 flex-1 justify-start gap-2.5 rounded-r-none pl-3.5 text-[0.9375rem]"
        disabled={disabled}
        onClick={onRecord}
      >
        <Mic />
        {t("sidebar.newRecording")}
      </Button>
      <PopoverMenu
        className="h-10 w-9 rounded-l-none border-primary-foreground/15 border-l"
        disabled={disabled}
        icon={<ChevronDown />}
        id="record"
        label={t("recording.options")}
        variant="default"
      >
        <div className="flex flex-col gap-3 p-1.5">
          <fieldset className="flex flex-col">
            <legend className="mb-2 text-muted-foreground text-xs">
              {t("settings.recording.input")}
            </legend>
            <RecordingInputs compact onError={onError} />
          </fieldset>
          <SettingCheckbox
            label={t("recording.live")}
            name="trascrizioneDalVivo"
            onError={onError}
          />
          <fieldset className="flex flex-col gap-2" disabled={!live}>
            <legend className={live ? "mb-2" : "mb-2 opacity-50"}>
              {t("settings.recording.parlanti")}
            </legend>
            {parlantiRegistrazione(settings).map((name) => (
              <SettingCheckbox
                disabled={!live}
                key={name}
                label={t(PARLANTI_LABELS[name])}
                name={name}
                onError={onError}
              />
            ))}
          </fieldset>
        </div>
      </PopoverMenu>
    </div>
  );
}
