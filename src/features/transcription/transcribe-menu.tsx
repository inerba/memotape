import { Captions } from "lucide-react";
import { type ChangeEvent, useCallback, useId } from "react";
import { useTranslation } from "react-i18next";
import type { AppError } from "@/bindings";
import { NativeSelect } from "@/components/native-select";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { useModels } from "@/features/models/use-models";
import { CleaningProfile } from "@/features/settings/cleaning-profile";
import { SettingSwitch } from "@/features/settings/setting-switch";
import {
  speechLanguageChoice,
  speechLanguageName,
} from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";

/**
 * La Lingua del parlato scelta e `nameOf` per i nomi delle lingue ("Automatica" o il nome). `ignoredBy` è il nome del modello scelto se non usa la lingua (Parakeet).
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
  return { ...choice, ignoredBy, nameOf };
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
    <div className="flex flex-col gap-3">
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

/** Trascrivi… per una Sorgente ancora senza testo: apre `TranscribeDialog`. */
export function TranscribeButton({
  disabled,
  onClick,
}: {
  disabled: boolean;
  onClick: () => void;
}) {
  const { t } = useTranslation();
  return (
    <Button className="h-10 px-4" disabled={disabled} onClick={onClick}>
      <Captions />
      {t("transcription.choose")}
    </Button>
  );
}

/**
 * Cosa si trascrive: un file da importare, un Tape senza testo, o un Tape il cui testo si sostituisce
 * (`manual`: con correzioni a mano).
 */
export type TranscribeTarget =
  | { kind: "file"; name: string }
  | { kind: "tape" }
  | { kind: "replace"; manual: boolean };

/**
 * Le scelte prima di ogni Trascrizione, sempre in vista. Un file importato si può pulire e
 * filtrare (Filtra rumore e Sensibilità); un Tape si trascrive con l'audio salvato così com'è
 * (ADR-0029), quindi offre solo modello, Lingua del parlato e Riconosci i parlanti.
 */
export function TranscribeDialog({
  onError,
  onOpenChange,
  onTranscribe,
  open,
  target,
}: {
  onError: (error: AppError) => void;
  onOpenChange: (open: boolean) => void;
  onTranscribe: () => void;
  open: boolean;
  target: TranscribeTarget;
}) {
  const { t } = useTranslation();
  const replace = target.kind === "replace";
  let title = t("transcription.dialog.tapeTitle");
  let description = t("transcription.dialog.tapeDescription");
  if (target.kind === "file") {
    title = t("transcription.dialog.fileTitle", { name: target.name });
    description = t("transcription.dialog.fileDescription");
  } else if (replace) {
    title = t("transcription.replace.title");
    description = t(
      target.manual
        ? "transcription.replace.manualDescription"
        : "transcription.replace.description"
    );
  }
  const showError = useCallback(
    (error: AppError | null) => {
      if (error) {
        onError(error);
      }
    },
    [onError]
  );
  return (
    <AlertDialog onOpenChange={onOpenChange} open={open}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>{description}</AlertDialogDescription>
        </AlertDialogHeader>
        <div className="flex flex-col gap-5">
          <TranscribeOptions onError={onError} />
          {target.kind === "file" ? (
            <CleaningProfile
              hideLegend
              name="audioFileMisto"
              onError={showError}
            />
          ) : null}
        </div>
        <AlertDialogFooter>
          <AlertDialogCancel>
            {t(replace ? "transcription.replace.keep" : "transcription.cancel")}
          </AlertDialogCancel>
          <AlertDialogAction onClick={onTranscribe}>
            {t(
              replace ? "transcription.replace.confirm" : "transcription.start"
            )}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
