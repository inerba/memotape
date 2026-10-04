import { ChevronDown } from "lucide-react";
import { type ChangeEvent, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { AppError } from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { Button } from "@/components/ui/button";
import { SELECT } from "@/features/library/move-select";
import { useModels } from "@/features/models/use-models";
import { SettingCheckbox } from "@/features/settings/setting-checkbox";
import { speechLanguageChoice } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";

/**
 * Trascrivi ▾: il pulsante dice la Lingua del parlato scelta, il menu sceglie modello, Lingua del
 * parlato e Riconosci i parlanti. Durante la Trascrizione c'è Annulla.
 */
export function TranscribeMenu({
  busy,
  canTranscribe,
  cancelling,
  onCancel,
  onError,
  onTranscribe,
  running,
}: {
  busy: boolean;
  canTranscribe: boolean;
  cancelling: boolean;
  /** `null` se non c'è niente da annullare. */
  onCancel: (() => void) | null;
  onError: (error: AppError) => void;
  onTranscribe: () => void;
  running: boolean;
}) {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  const models = useModels();
  const modelLanguages =
    models.find((m) => m.id === settings.model)?.languages ?? null;
  const { options: languages, value: language } = speechLanguageChoice(
    modelLanguages,
    settings.speechLanguage
  );
  // Quelli scaricati, più quello scelto anche se non lo è: Trascrivi dirà che manca.
  const transcribers = models.filter(
    (m) =>
      m.kind === "trascrizione" &&
      (m.state.state === "downloaded" || m.id === settings.model)
  );

  const choose = useCallback(
    async (change: Partial<typeof settings>) => {
      const error = await save({ ...settings, ...change });
      if (error) {
        onError(error);
      }
    },
    [onError, save, settings]
  );
  const chooseModel = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) => choose({ model: e.target.value }),
    [choose]
  );
  const chooseLanguage = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({
        speechLanguage: languages.find((l) => l === e.target.value) ?? "auto",
      }),
    [choose, languages]
  );

  const languageName =
    language === "auto"
      ? t("speechLanguage.auto")
      : t(`speechLanguage.languages.${language}`);
  return (
    <div className="flex items-center gap-1">
      <Button disabled={!canTranscribe} onClick={onTranscribe}>
        {running
          ? t("transcription.running")
          : t("transcription.startWith", { language: languageName })}
      </Button>
      <PopoverMenu
        disabled={busy}
        icon={<ChevronDown />}
        id="transcribe"
        label={t("transcription.options")}
      >
        <label className="flex flex-col gap-1">
          {t("models.choose")}
          <select
            className={SELECT}
            onChange={chooseModel}
            value={settings.model}
          >
            {transcribers.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1">
          {t("speechLanguage.label")}
          <select className={SELECT} onChange={chooseLanguage} value={language}>
            <option value="auto">{t("speechLanguage.auto")}</option>
            {languages.map((l) => (
              <option key={l} value={l}>
                {t(`speechLanguage.languages.${l}`)}
              </option>
            ))}
          </select>
        </label>
        <SettingCheckbox
          label={t("transcription.parlanti")}
          name="parlantiFile"
          note={t("transcription.parlantiNote")}
          onError={onError}
        />
      </PopoverMenu>
      {onCancel ? (
        <Button disabled={cancelling} onClick={onCancel} variant="outline">
          {cancelling
            ? t("transcription.cancelling")
            : t("transcription.cancel")}
        </Button>
      ) : null}
    </div>
  );
}
