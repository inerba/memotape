import { Captions, ChevronDown } from "lucide-react";
import { type ChangeEvent, useCallback, useId } from "react";
import { useTranslation } from "react-i18next";
import type { AppError } from "@/bindings";
import { NativeSelect } from "@/components/native-select";
import { PopoverMenu } from "@/components/popover-menu";
import { Button } from "@/components/ui/button";
import { useModels } from "@/features/models/use-models";
import { SettingSwitch } from "@/features/settings/setting-switch";
import {
  speechLanguageChoice,
  speechLanguageName,
} from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";

/**
 * La Lingua del parlato scelta, come si mostra: "Automatica" o il nome della lingua, e `nameOf` per
 * i nomi delle altre. `ignoredBy` è il nome del modello scelto se non usa la lingua (Parakeet).
 */
function useSpeechLanguage() {
  const { i18n, t } = useTranslation();
  const { settings } = useSettings();
  const models = useModels();
  const model = models.find((m) => m.id === settings.model);
  const choice = speechLanguageChoice(
    model?.languages ?? null,
    settings.speechLanguage,
    i18n.language
  );
  const nameOf = (code: string) =>
    code === "auto"
      ? t("speechLanguage.auto")
      : speechLanguageName(code, i18n.language);
  const ignoredBy =
    model && !model.acceptsLanguage && choice.value !== "auto"
      ? model.name
      : null;
  return { ...choice, ignoredBy, name: nameOf(choice.value), nameOf };
}

/** Le scelte della prossima Trascrizione: modello, Lingua del parlato e Riconosci i parlanti. */
export function TranscribeOptions({
  onError,
}: {
  onError: (error: AppError) => void;
}) {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  const models = useModels();
  const {
    ignoredBy,
    nameOf,
    options: languages,
    value: language,
  } = useSpeechLanguage();
  const modelId = useId();
  const languageId = useId();
  // Quelli scaricati, più quello scelto anche se non lo è: Trascrivi dirà che manca.
  const transcribers = models.filter(
    (m) =>
      m.kind === "trascrizione" &&
      (m.state.state === "downloaded" || m.id === settings.model)
  );

  const choose = useCallback(
    async (change: Partial<typeof settings>) => {
      const error = await save((current) => ({ ...current, ...change }));
      if (error) {
        onError(error);
      }
    },
    [onError, save]
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

  return (
    <div className="flex flex-col gap-3 px-2 py-2">
      <div className="flex flex-col gap-1.5">
        <label className="text-muted-foreground text-xs" htmlFor={modelId}>
          {t("models.choose")}
        </label>
        <NativeSelect
          id={modelId}
          onChange={chooseModel}
          value={settings.model}
        >
          {transcribers.map((m) => (
            <option key={m.id} value={m.id}>
              {m.name}
            </option>
          ))}
        </NativeSelect>
      </div>
      <div className="flex flex-col gap-1.5">
        <label className="text-muted-foreground text-xs" htmlFor={languageId}>
          {t("speechLanguage.label")}
        </label>
        <NativeSelect
          id={languageId}
          onChange={chooseLanguage}
          value={language}
        >
          <option value="auto">{t("speechLanguage.auto")}</option>
          {languages.map((l) => (
            <option key={l} value={l}>
              {nameOf(l)}
            </option>
          ))}
        </NativeSelect>
        {ignoredBy ? (
          <p className="max-w-64 text-muted-foreground text-xs">
            {t("speechLanguage.notHonored", { model: ignoredBy })}
          </p>
        ) : null}
      </div>
      <SettingSwitch
        description={
          settings.diarizer === "nemotron3"
            ? t("transcription.nemotron3Note")
            : undefined
        }
        label={t("transcription.parlanti")}
        name="parlantiFile"
        note={
          settings.diarizer === "nemotron3"
            ? undefined
            : t("transcription.parlantiNote", { count: 4 })
        }
        onError={onError}
      />
    </div>
  );
}

/**
 * Trascrivi ▾ per una Sorgente ancora senza testo: il pulsante dice la Lingua del parlato scelta, il
 * menu sceglie modello, Lingua del parlato e Riconosci i parlanti.
 */
export function TranscribeMenu({
  busy,
  canTranscribe,
  onError,
  onTranscribe,
}: {
  busy: boolean;
  canTranscribe: boolean;
  onError: (error: AppError) => void;
  onTranscribe: () => void;
}) {
  const { t } = useTranslation();
  const { name } = useSpeechLanguage();
  return (
    <div className="flex">
      <Button
        className="h-10 rounded-r-none px-4"
        disabled={!canTranscribe}
        onClick={onTranscribe}
      >
        <Captions />
        {t("transcription.startWith", { language: name })}
      </Button>
      <PopoverMenu
        className="h-10 w-9 rounded-l-none border-primary-foreground/15 border-l"
        disabled={busy}
        icon={<ChevronDown />}
        id="transcribe"
        label={t("transcription.options")}
        variant="default"
      >
        <TranscribeOptions onError={onError} />
      </PopoverMenu>
    </div>
  );
}
