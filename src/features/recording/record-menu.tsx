import { ChevronDown, Circle } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AppError } from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { Button } from "@/components/ui/button";
import { SettingCheckbox } from "@/features/settings/setting-checkbox";
import {
  PARLANTI_LABELS,
  parlantiRegistrazione,
} from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";

/**
 * Registra ▾: il menu sceglie Trascrivi dal vivo e Riconosci i parlanti della Registrazione, che vale
 * solo dal vivo.
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
    <div className="flex flex-1 items-center gap-1">
      <Button
        className="flex-1"
        disabled={disabled}
        onClick={onRecord}
        variant="outline"
      >
        <Circle className="fill-destructive text-destructive" />
        {t("recording.start")}
      </Button>
      <PopoverMenu
        disabled={disabled}
        icon={<ChevronDown />}
        id="record"
        label={t("recording.options")}
      >
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
      </PopoverMenu>
    </div>
  );
}
