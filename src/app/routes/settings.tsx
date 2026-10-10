import { zodResolver } from "@hookform/resolvers/zod";
import {
  ArrowLeft,
  Captions,
  ChevronDown,
  Info,
  Keyboard,
  type LucideIcon,
  Mic,
  Plug,
  SlidersHorizontal,
} from "lucide-react";
import {
  type ChangeEvent,
  type KeyboardEvent,
  type MouseEvent,
  type ReactNode,
  useCallback,
  useEffect,
  useState,
} from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { Link, useSearchParams } from "react-router";
import {
  type AppError,
  type AudioDevice,
  type CopiaCome,
  commands,
  type Settings,
  type Tema,
  type VistaTrascrizione,
} from "@/bindings";
import { FieldHelp } from "@/components/field-help";
import { NativeSelect } from "@/components/native-select";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { WINDOW_CONTROLS_PADDING } from "@/components/window-controls";
import { About } from "@/features/about/about";
import { ModelList } from "@/features/models/model-list";
import { AssistantsSection } from "@/features/settings/assistants-section";
import { CleaningProfile } from "@/features/settings/cleaning-profile";
import { DeviceSelect } from "@/features/settings/device-select";
import { GuadagnoSelect } from "@/features/settings/guadagno-select";
import { RecordingInputs } from "@/features/settings/recording-inputs";
import {
  SaveTick,
  SettingFeedback,
} from "@/features/settings/setting-feedback";
import { SettingSwitch } from "@/features/settings/setting-switch";
import {
  BITRATES_KBPS,
  LANGUAGE_NAMES,
  LANGUAGES,
  PARLANTI_LABELS,
  parlantiRegistrazione,
  SAMPLE_RATES,
  settingsSchema,
} from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import type { SettingField } from "@/features/settings/settings-writer";
import { VocabolarioList } from "@/features/settings/vocabolario-list";
import { ariaTasti, conTasti } from "@/features/shortcuts/shortcuts";
import { useApriPannello } from "@/features/shortcuts/shortcuts-provider";
import { errorText } from "@/features/status/status";

const COPY_FORMATS: CopiaCome[] = ["testo", "markdown"];
const TEMI: Tema[] = ["sistema", "chiaro", "scuro"];
const VISTE: VistaTrascrizione[] = ["copione", "intervista", "nastro"];

/** Le sezioni, una per volta: `?sezione=` le sceglie (l'avviso dei modelli apre Trascrizione). */
const SECTIONS = [
  { icon: Mic, key: "registrazione", text: "settings.recording" },
  { icon: Captions, key: "trascrizione", text: "settings.transcription" },
  { icon: SlidersHorizontal, key: "generale", text: "settings.general" },
  { icon: Plug, key: "assistenti", text: "settings.assistants" },
  { icon: Info, key: "informazioni", text: "about" },
] as const satisfies { icon: LucideIcon; key: string; text: string }[];

export function SettingsPage() {
  const { i18n, t } = useTranslation();
  const { save, settings } = useSettings();
  const apriPannello = useApriPannello();
  const [error, setError] = useState<AppError | null>(null);
  // `null` finché `list_microphones` e `list_output_devices` non rispondono.
  const [microphones, setMicrophones] = useState<AudioDevice[] | null>(null);
  const [outputs, setOutputs] = useState<AudioDevice[] | null>(null);
  const [folder, setFolder] = useState("");
  const [params, setParams] = useSearchParams();
  const section =
    SECTIONS.find((s) => s.key === params.get("sezione")) ?? SECTIONS[0];
  const show = useCallback(
    (e: MouseEvent<HTMLButtonElement>) =>
      setParams({ sezione: e.currentTarget.value }, { replace: true }),
    [setParams]
  );
  // Ogni scelta si salva subito: non c'è un pulsante Salva.
  const form = useForm<Settings>({
    resolver: zodResolver(settingsSchema),
    values: settings,
  });
  const { handleSubmit, reset } = form;
  const choose = useCallback(
    (patch: Partial<Settings>) => {
      reset({ ...settings, ...patch });
      handleSubmit(async () => {
        await save(
          (current) => ({ ...current, ...patch }),
          Object.keys(patch)[0] as keyof Settings
        );
        // SettingsProvider ripristina il valore confermato e riapplica le altre scelte pendenti.
      })();
    },
    [handleSubmit, reset, save, settings]
  );
  const chooseOutput = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({ outputDevice: e.target.value || null }),
    [choose]
  );
  const chooseMicrophone = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({ microphone: e.target.value || null }),
    [choose]
  );
  const chooseBitrate = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({ bitrateKbps: Number(e.target.value) }),
    [choose]
  );
  const chooseChannels = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({ channels: e.target.value === "stereo" ? "stereo" : "mono" }),
    [choose]
  );
  const chooseSampleRate = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({ sampleRate: Number(e.target.value) }),
    [choose]
  );
  const selectModel = useCallback(
    (model: string) => choose({ model }),
    [choose]
  );
  const chooseDiarizer = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({
        diarizer: e.target.value === "nemotron3" ? "nemotron3" : "sortformer",
      }),
    [choose]
  );
  const pickDiarizer = useCallback(async () => {
    const picked = await commands.pickDiarizerModel();
    if (picked.status === "ok" && picked.data) {
      choose({ nemotron3Path: picked.data });
      setError(null);
    } else if (picked.status === "error") {
      setError(picked.error);
    }
  }, [choose]);
  // Si applica al riavvio: l'interfaccia resta nella lingua con cui è partita.
  const chooseLanguage = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({
        interfaceLanguage:
          LANGUAGES.find((l) => l === e.target.value) ??
          settings.interfaceLanguage,
      }),
    [choose, settings.interfaceLanguage]
  );
  const chooseCopyFormat = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({
        copiaCome:
          COPY_FORMATS.find((f) => f === e.target.value) ?? settings.copiaCome,
      }),
    [choose, settings.copiaCome]
  );
  const chooseVista = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({
        vistaTrascrizione:
          VISTE.find((v) => v === e.target.value) ?? settings.vistaTrascrizione,
      }),
    [choose, settings.vistaTrascrizione]
  );
  const chooseNomeMicrofono = useCallback(
    (nome: string) => choose({ nomeMicrofono: nome.trim() || null }),
    [choose]
  );
  // Si applica subito: lo fa Rust al salvataggio.
  const chooseTema = useCallback(
    (e: ChangeEvent<HTMLInputElement>) =>
      choose({ tema: TEMI.find((m) => m === e.target.value) ?? settings.tema }),
    [choose, settings.tema]
  );
  const resetFolder = useCallback(
    () => choose({ recordingsFolder: null }),
    [choose]
  );

  useEffect(() => {
    commands.listMicrophones().then((result) => {
      if (result.status === "ok") {
        setMicrophones(result.data);
      } else {
        setError(result.error);
      }
    });
    commands.listOutputDevices().then((result) => {
      if (result.status === "ok") {
        setOutputs(result.data);
      } else {
        setError(result.error);
      }
    });
  }, []);

  // La cartella in uso cambia con la scelta salvata.
  // biome-ignore lint/correctness/useExhaustiveDependencies: si rilegge a ogni cambio della scelta.
  useEffect(() => {
    commands.recordingsFolder().then((result) => {
      if (result.status === "ok") {
        setFolder(result.data);
      }
    });
  }, [settings.recordingsFolder]);

  const pickFolder = useCallback(async () => {
    const picked = await commands.pickFolder();
    if (picked) {
      choose({ recordingsFolder: picked });
    }
  }, [choose]);

  return (
    <div className="fixed inset-0 z-10 flex flex-col bg-background">
      <header
        className={`flex h-12 shrink-0 items-center px-3 ${WINDOW_CONTROLS_PADDING}`}
        data-tauri-drag-region
      >
        <Button asChild size="sm" variant="ghost">
          <Link to="/">
            <ArrowLeft />
            {t("settings.back")}
          </Link>
        </Button>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
        <div className="mx-auto max-w-5xl px-10 pt-2 pb-16">
          <div className="pointer-events-none">
            <h1 className="text-balance font-display font-medium text-[2rem] leading-tight tracking-[-0.015em]">
              {t("settings.title")}
            </h1>
            <p className="mt-1.5 text-muted-foreground">
              {t("settings.subtitle")}
            </p>
          </div>
          <div className="mt-8 flex items-start gap-6">
            <nav
              aria-label={t("settings.title")}
              className="sticky top-6 -ml-2.5 flex w-60 shrink-0 flex-col gap-0.5"
            >
              {SECTIONS.map(({ icon: Icon, key, text }) => (
                <button
                  aria-current={key === section.key ? "page" : undefined}
                  className="flex h-9 items-center gap-2.5 rounded-lg px-2.5 text-left text-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-[current=page]:bg-accent aria-[current=page]:font-medium [&_svg]:size-4 [&_svg]:shrink-0 [&_svg]:text-foreground/80"
                  key={key}
                  onClick={show}
                  type="button"
                  value={key}
                >
                  <Icon />
                  <span className="min-w-0 truncate">{t(`${text}.title`)}</span>
                </button>
              ))}
            </nav>
            <section
              aria-labelledby="settings-section"
              className="flex min-w-0 flex-1 flex-col gap-7 rounded-xl border bg-card p-7"
            >
              <div className="flex flex-col gap-1">
                <h2 className="font-medium text-lg" id="settings-section">
                  {t(`${section.text}.title`)}
                </h2>
                <p className="text-muted-foreground text-sm leading-relaxed">
                  {t(`${section.text}.description`)}
                </p>
              </div>
              {error ? (
                <p className="text-destructive text-sm" role="alert">
                  {errorText(error, t)}
                </p>
              ) : null}
              {section.key === "registrazione" ? (
                <>
                  <fieldset className="flex flex-col">
                    <legend className="mb-2 flex items-center gap-1 font-medium text-sm">
                      {t("settings.recording.input")}
                      <SaveTick
                        label={t("settings.recording.input")}
                        name="recordingSource"
                      />
                    </legend>
                    <RecordingInputs onError={setError} />
                  </fieldset>
                  <RecordingInputSettings
                    devices={microphones}
                    input="mic"
                    onChange={chooseMicrophone}
                    onError={setError}
                  />
                  <RecordingInputSettings
                    devices={outputs}
                    input="system"
                    onChange={chooseOutput}
                    onError={setError}
                  />
                  <fieldset
                    aria-describedby={
                      settings.diarizer === "nemotron3"
                        ? "recording-parlanti-description recording-nemotron-description"
                        : "recording-parlanti-description"
                    }
                    className="flex flex-col gap-2"
                  >
                    <legend className="mb-2">
                      <span className="flex items-center gap-1 font-medium text-sm">
                        {t("settings.recording.parlanti")}
                        <FieldHelp label={t("settings.recording.parlanti")}>
                          <p>{t("settings.recording.parlantiDescription")}</p>
                          {settings.diarizer === "nemotron3" ? (
                            <p className="mt-2">
                              {t(
                                "settings.transcription.experimentalDescription"
                              )}
                            </p>
                          ) : null}
                        </FieldHelp>
                      </span>
                    </legend>
                    <div className="flex flex-wrap gap-x-5 gap-y-2">
                      {parlantiRegistrazione(settings).map((name) => (
                        <SettingSwitch
                          key={name}
                          label={t(PARLANTI_LABELS[name])}
                          name={name}
                          onError={setError}
                        />
                      ))}
                    </div>
                    <p className="sr-only" id="recording-parlanti-description">
                      {t("settings.recording.parlantiDescription")}
                    </p>
                    {settings.diarizer === "nemotron3" ? (
                      <p
                        className="sr-only"
                        id="recording-nemotron-description"
                      >
                        {t(
                          "settings.transcription.recordingDiarizationDescription"
                        )}
                      </p>
                    ) : null}
                  </fieldset>
                  <details className="group border-t pt-5">
                    <summary className="flex cursor-pointer list-none items-center justify-between gap-2 rounded-md font-medium text-sm focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40">
                      {t("settings.recording.quality")}
                      <ChevronDown
                        aria-hidden
                        className="size-4 shrink-0 transition-transform group-open:rotate-180 motion-reduce:transition-none"
                      />
                    </summary>
                    <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3">
                      <Field
                        description={t("settings.recording.bitrateDescription")}
                        id="bitrate"
                        label={t("settings.recording.bitrate")}
                        name="bitrateKbps"
                      >
                        <NativeSelect
                          aria-describedby="bitrate-description"
                          className="h-9 tabular-nums"
                          id="bitrate"
                          onChange={chooseBitrate}
                          value={settings.bitrateKbps}
                        >
                          {BITRATES_KBPS.map((value) => (
                            <option key={value} value={value}>
                              {t("settings.recording.kbps", { value })}
                            </option>
                          ))}
                        </NativeSelect>
                      </Field>
                      <Field
                        description={t(
                          "settings.recording.channelsDescription"
                        )}
                        id="channels"
                        label={t("settings.recording.channels")}
                        name="channels"
                      >
                        <NativeSelect
                          aria-describedby="channels-description"
                          className="h-9"
                          id="channels"
                          onChange={chooseChannels}
                          value={settings.channels}
                        >
                          <option value="mono">
                            {t("settings.recording.mono")}
                          </option>
                          <option value="stereo">
                            {t("settings.recording.stereo")}
                          </option>
                        </NativeSelect>
                      </Field>
                      <Field
                        description={t(
                          "settings.recording.sampleRateDescription"
                        )}
                        id="sample-rate"
                        label={t("settings.recording.sampleRate")}
                        name="sampleRate"
                      >
                        <NativeSelect
                          aria-describedby="sample-rate-description"
                          className="h-9 tabular-nums"
                          id="sample-rate"
                          onChange={chooseSampleRate}
                          value={settings.sampleRate}
                        >
                          {SAMPLE_RATES.map((value) => (
                            <option key={value} value={value}>
                              {t("settings.recording.hz", { value })}
                            </option>
                          ))}
                        </NativeSelect>
                      </Field>
                    </div>
                  </details>
                </>
              ) : null}
              {section.key === "trascrizione" ? (
                <>
                  <ModelList
                    kind="trascrizione"
                    onSelect={selectModel}
                    selected={settings.model}
                  />
                  <SettingFeedback label={t("models.choose")} name="model" />
                  <VocabolarioList />
                  <div className="flex items-center gap-1 border-t pt-7">
                    <h3 className="font-medium text-sm">
                      {t("settings.transcription.diarization")}
                    </h3>
                    <FieldHelp label={t("settings.transcription.diarization")}>
                      {t("settings.transcription.diarizationDescription")}
                    </FieldHelp>
                  </div>
                  <ModelList kind="diarizzazione" />
                  <DiarizerSettings
                    onChange={chooseDiarizer}
                    onPick={pickDiarizer}
                  />
                </>
              ) : null}
              {section.key === "generale" ? (
                <>
                  <Field
                    description={t("settings.general.folderDescription")}
                    label={t("settings.general.folder")}
                    name="recordingsFolder"
                  >
                    <div className="flex items-center gap-2">
                      <span
                        className="flex h-9 min-w-0 flex-1 items-center truncate rounded-md border border-input bg-background px-3 text-sm"
                        title={folder}
                      >
                        {folder}
                      </span>
                      <Button
                        className="h-9"
                        onClick={pickFolder}
                        variant="outline"
                      >
                        {t("settings.general.choose")}
                      </Button>
                    </div>
                    {settings.recordingsFolder === null ? null : (
                      <Button
                        className="-ml-2.5 self-start"
                        onClick={resetFolder}
                        size="sm"
                        variant="ghost"
                      >
                        {t("settings.general.reset")}
                      </Button>
                    )}
                  </Field>
                  <fieldset className="flex flex-col">
                    <legend className="mb-3 flex items-center gap-1 font-medium text-sm">
                      {t("settings.general.theme")}
                      <SaveTick
                        label={t("settings.general.theme")}
                        name="tema"
                      />
                    </legend>
                    <div className="grid max-w-xl grid-cols-3 gap-4">
                      {TEMI.map((value) => (
                        <label
                          className="group flex cursor-pointer flex-col items-center gap-2.5"
                          key={value}
                        >
                          <input
                            checked={(settings.tema ?? "sistema") === value}
                            className="peer sr-only"
                            name="tema"
                            onChange={chooseTema}
                            type="radio"
                            value={value}
                          />
                          <span className="relative aspect-[16/10] w-full overflow-hidden rounded-lg border outline-offset-[3px] transition-[box-shadow,border-color] group-hover:border-input peer-checked:outline-2 peer-checked:outline-foreground peer-focus-visible:ring-[3px] peer-focus-visible:ring-ring/40">
                            <Miniature night={value === "scuro"} />
                            {value === "sistema" ? (
                              <span className="absolute inset-0 [clip-path:polygon(100%_0,100%_100%,0_100%)]">
                                <Miniature night />
                              </span>
                            ) : null}
                          </span>
                          <span className="text-muted-foreground text-sm peer-checked:font-medium peer-checked:text-foreground">
                            {t(`settings.general.themes.${value}`)}
                          </span>
                        </label>
                      ))}
                    </div>
                  </fieldset>
                  <SettingFeedback
                    label={t("settings.general.theme")}
                    name="tema"
                  />
                  <Field
                    description={t("settings.general.vistaDescription")}
                    id="vista-trascrizione"
                    label={t("settings.general.vista")}
                    name="vistaTrascrizione"
                  >
                    <NativeSelect
                      aria-describedby="vista-trascrizione-description"
                      className="h-9"
                      id="vista-trascrizione"
                      onChange={chooseVista}
                      value={settings.vistaTrascrizione ?? "copione"}
                    >
                      {VISTE.map((value) => (
                        <option key={value} value={value}>
                          {t(`settings.general.viste.${value}`)}
                        </option>
                      ))}
                    </NativeSelect>
                  </Field>
                  <Field
                    description={t("settings.general.languageRestart")}
                    id="interface-language"
                    label={t("settings.general.language")}
                    name="interfaceLanguage"
                  >
                    <NativeSelect
                      aria-describedby="interface-language-description"
                      className="h-9"
                      id="interface-language"
                      onChange={chooseLanguage}
                      value={settings.interfaceLanguage ?? i18n.language}
                    >
                      {LANGUAGES.map((value) => (
                        <option key={value} lang={value} value={value}>
                          {LANGUAGE_NAMES[value]}
                        </option>
                      ))}
                    </NativeSelect>
                  </Field>
                  <Field
                    id="copy-format"
                    label={t("settings.general.copyAs")}
                    name="copiaCome"
                  >
                    <NativeSelect
                      className="h-9"
                      id="copy-format"
                      onChange={chooseCopyFormat}
                      value={settings.copiaCome ?? "testo"}
                    >
                      {COPY_FORMATS.map((value) => (
                        <option key={value} value={value}>
                          {t(`settings.general.copyFormats.${value}`)}
                        </option>
                      ))}
                    </NativeSelect>
                  </Field>
                  <Field
                    description={t("settings.general.nomeMicrofonoDescription")}
                    id="nome-microfono"
                    label={t("settings.general.nomeMicrofono")}
                    name="nomeMicrofono"
                  >
                    <NomeMicrofonoInput
                      onSave={chooseNomeMicrofono}
                      value={settings.nomeMicrofono ?? ""}
                    />
                  </Field>
                  <Field label={t("shortcuts.title")}>
                    <Button
                      aria-keyshortcuts={ariaTasti("pannello")}
                      className="h-9 self-start"
                      onClick={apriPannello}
                      title={conTasti(t, t("shortcuts.show"), "pannello")}
                      variant="outline"
                    >
                      <Keyboard />
                      {t("shortcuts.show")}
                    </Button>
                  </Field>
                </>
              ) : null}
              {section.key === "assistenti" ? (
                <AssistantsSection onError={setError} />
              ) : null}
              {section.key === "informazioni" ? <About /> : null}
            </section>
          </div>
        </div>
      </div>
    </div>
  );
}

/** Dispositivo e trattamento dello stesso Ingresso, sempre nello stesso ordine. */
function RecordingInputSettings({
  input,
  devices,
  onChange,
  onError,
}: {
  input: "mic" | "system";
  devices: AudioDevice[] | null;
  onChange: (event: ChangeEvent<HTMLSelectElement>) => void;
  onError: (error: AppError | null) => void;
}) {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const inactive =
    settings.recordingSource !== "both" && settings.recordingSource !== input;
  const mic = input === "mic";
  const label = t(`settings.recording.inputs.${input}`);
  const deviceId = mic ? "microphone" : "output-device";
  return (
    <section
      aria-labelledby={`input-${input}-title`}
      className="flex min-w-0 flex-col gap-4 border-t pt-5"
    >
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
        <h3 className="font-medium text-sm" id={`input-${input}-title`}>
          {label}
        </h3>
        {inactive ? (
          <span className="flex items-center gap-1 text-muted-foreground text-xs">
            {t("settings.recording.inactive")}
            <FieldHelp label={label}>
              {t("settings.recording.inactiveDescription")}
            </FieldHelp>
          </span>
        ) : null}
      </div>
      <div className="grid min-w-0 grid-cols-1 gap-4 sm:grid-cols-2">
        <Field
          feedbackLabel={t("settings.recording.deviceFor", { input: label })}
          id={deviceId}
          label={t("settings.recording.device")}
          name={mic ? "microphone" : "outputDevice"}
        >
          <DeviceSelect
            devices={devices}
            disabled={inactive}
            id={deviceId}
            onChange={onChange}
            value={mic ? settings.microphone : settings.outputDevice}
          />
        </Field>
        <Field
          description={t("settings.recording.gainDescription")}
          id={`guadagno-${input}`}
          label={t("settings.recording.guadagno")}
        >
          <GuadagnoSelect
            aria-describedby={`guadagno-${input}-description`}
            aria-label={t("recording.guadagno", { input: label })}
            className="h-9"
            disabled={inactive}
            id={`guadagno-${input}`}
            name={mic ? "guadagnoMicrofono" : "guadagnoSistema"}
            onError={onError}
          />
        </Field>
      </div>
      <CleaningProfile
        hideLegend
        name={mic ? "audioMicrofono" : "audioSistema"}
        onError={onError}
      />
    </section>
  );
}

/** Il modello dei Parlanti e le indicazioni per la scelta sperimentale. */
function DiarizerSettings({
  onChange,
  onPick,
}: {
  onChange: (event: ChangeEvent<HTMLSelectElement>) => void;
  onPick: () => void;
}) {
  const { t } = useTranslation();
  const { settings } = useSettings();
  return (
    <>
      <Field
        description={
          settings.diarizer === "nemotron3"
            ? `${t("settings.transcription.experimentalDescription")} ${t("settings.transcription.recordingDiarizationDescription")}`
            : undefined
        }
        id="diarizer"
        label={t("settings.transcription.diarizer")}
        name="diarizer"
      >
        <NativeSelect
          aria-describedby={
            settings.diarizer === "nemotron3"
              ? "diarizer-description"
              : undefined
          }
          id="diarizer"
          name="diarizer"
          onChange={onChange}
          value={settings.diarizer ?? "sortformer"}
        >
          <option value="sortformer">Sortformer 4spk v2.1</option>
          <option value="nemotron3">
            {t("settings.transcription.nemotron3")}
          </option>
        </NativeSelect>
      </Field>
      {settings.diarizer === "nemotron3" ? (
        <Field
          description={t("settings.transcription.localModelDescription")}
          label={t("settings.transcription.localModel")}
          name="nemotron3Path"
        >
          <span className="break-all text-sm">
            {settings.nemotron3Path ??
              t("settings.transcription.localModelMissing")}
          </span>
          <Button className="self-start" onClick={onPick} variant="outline">
            {t("settings.transcription.chooseLocalModel")}
          </Button>
        </Field>
      ) : null}
    </>
  );
}

/**
 * Il nome predefinito del Microfono: si scrive liberamente e si salva uscendo dal campo o con
 * Invio; Esc torna al nome salvato.
 */
function NomeMicrofonoInput({
  onSave,
  value,
}: {
  onSave: (nome: string) => void;
  value: string;
}) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  const commit = useCallback(() => {
    if (draft.trim() !== value) {
      onSave(draft);
    }
  }, [draft, onSave, value]);
  const change = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => setDraft(e.target.value),
    []
  );
  const key = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter") {
        commit();
      } else if (e.key === "Escape") {
        setDraft(value);
      }
    },
    [commit, value]
  );
  return (
    <Input
      aria-describedby="nome-microfono-description"
      className="h-9 max-w-72"
      id="nome-microfono"
      maxLength={60}
      onBlur={commit}
      onChange={change}
      onKeyDown={key}
      placeholder={t("settings.general.nomeMicrofonoPlaceholder")}
      value={draft}
    />
  );
}

/** Etichetta e aiuto contestuale, controllo ed esito del salvataggio. */
function Field({
  children,
  description,
  id,
  label,
  name,
  feedbackLabel,
}: {
  children: ReactNode;
  description?: string;
  id?: string;
  label: string;
  name?: SettingField;
  feedbackLabel?: string;
}) {
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <div className="flex items-center gap-1">
        {id ? (
          <label className="font-medium text-sm" htmlFor={id}>
            {label}
          </label>
        ) : (
          <span className="font-medium text-sm">{label}</span>
        )}
        {description ? (
          <FieldHelp label={label}>{description}</FieldHelp>
        ) : null}
        {name ? <SaveTick label={feedbackLabel ?? label} name={name} /> : null}
      </div>
      {children}
      {name ? (
        <SettingFeedback label={feedbackLabel ?? label} name={name} />
      ) : null}
      {description ? (
        <p className="sr-only" id={id ? `${id}-description` : undefined}>
          {description}
        </p>
      ) : null}
    </div>
  );
}

/** La finestra di Memotape in piccolo, di giorno o di sera, per la scelta del tema. */
function Miniature({ night = false }: { night?: boolean }) {
  const [paper, sidebar, ink] = night
    ? ["bg-night-paper", "bg-night-sidebar", "bg-night-ink"]
    : ["bg-day-paper", "bg-day-sidebar", "bg-day-ink"];
  return (
    <span aria-hidden className={`absolute inset-0 flex ${paper}`}>
      <span className={`flex w-[32%] flex-col gap-1.5 p-2 ${sidebar}`}>
        <span className={`h-2 w-full rounded-[3px] ${ink}`} />
        <span className={`mt-1 h-1 w-4/5 rounded-full opacity-25 ${ink}`} />
        <span className={`h-1 w-3/5 rounded-full opacity-25 ${ink}`} />
        <span className={`h-1 w-2/3 rounded-full opacity-25 ${ink}`} />
      </span>
      <span className="flex flex-1 flex-col gap-1.5 px-3 pt-3">
        <span className={`h-2 w-1/2 rounded-full opacity-90 ${ink}`} />
        <span className={`mt-1 h-1 w-full rounded-full opacity-25 ${ink}`} />
        <span className={`h-1 w-11/12 rounded-full opacity-25 ${ink}`} />
        <span className={`h-1 w-4/5 rounded-full opacity-25 ${ink}`} />
      </span>
    </span>
  );
}
