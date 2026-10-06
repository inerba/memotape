import { zodResolver } from "@hookform/resolvers/zod";
import {
  ArrowLeft,
  Captions,
  Info,
  type LucideIcon,
  Mic,
  Plug,
  SlidersHorizontal,
} from "lucide-react";
import {
  type ChangeEvent,
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
} from "@/bindings";
import { NativeSelect } from "@/components/native-select";
import { Button } from "@/components/ui/button";
import { WINDOW_CONTROLS_PADDING } from "@/components/window-controls";
import { About } from "@/features/about/about";
import { ModelList } from "@/features/models/model-list";
import { AssistantsSection } from "@/features/settings/assistants-section";
import { GuadagnoSelect } from "@/features/settings/guadagno-select";
import { RecordingInputs } from "@/features/settings/recording-inputs";
import { SettingCheckbox } from "@/features/settings/setting-checkbox";
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
import { errorText } from "@/features/status/status";

const COPY_FORMATS: CopiaCome[] = ["testo", "markdown"];
const TEMI: Tema[] = ["sistema", "chiaro", "scuro"];

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
      handleSubmit(async (values) => {
        const failed = await save(values);
        setError(failed);
        if (failed) {
          // La scelta non è salvata: il form torna alle impostazioni correnti.
          reset(settings);
        }
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
              className="sticky top-6 flex w-60 shrink-0 flex-col gap-0.5 rounded-xl border bg-card p-1.5"
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
                    <legend className="mb-2 font-medium text-sm">
                      {t("settings.recording.input")}
                    </legend>
                    <RecordingInputs onError={setError} />
                  </fieldset>
                  <div className="grid grid-cols-2 gap-4">
                    <Field
                      id="microphone"
                      label={t("settings.recording.microphone")}
                    >
                      <DeviceSelect
                        devices={microphones}
                        disabled={settings.recordingSource === "system"}
                        id="microphone"
                        onChange={chooseMicrophone}
                        value={settings.microphone}
                      />
                    </Field>
                    <Field
                      id="output-device"
                      label={t("settings.recording.outputDevice")}
                    >
                      <DeviceSelect
                        devices={outputs}
                        disabled={settings.recordingSource === "mic"}
                        id="output-device"
                        onChange={chooseOutput}
                        value={settings.outputDevice}
                      />
                    </Field>
                    <Field
                      id="guadagno-microfono"
                      label={t("settings.recording.guadagno")}
                    >
                      <GuadagnoSelect
                        aria-label={t("recording.guadagno", {
                          input: t("settings.recording.inputs.mic"),
                        })}
                        className="h-9"
                        disabled={settings.recordingSource === "system"}
                        id="guadagno-microfono"
                        name="guadagnoMicrofono"
                        onError={setError}
                      />
                    </Field>
                    <Field
                      id="guadagno-sistema"
                      label={t("settings.recording.guadagno")}
                    >
                      <GuadagnoSelect
                        aria-label={t("recording.guadagno", {
                          input: t("settings.recording.inputs.system"),
                        })}
                        className="h-9"
                        disabled={settings.recordingSource === "mic"}
                        id="guadagno-sistema"
                        name="guadagnoSistema"
                        onError={setError}
                      />
                    </Field>
                  </div>
                  <fieldset
                    aria-describedby={
                      settings.diarizer === "nemotron3"
                        ? "recording-parlanti-description recording-nemotron-description"
                        : "recording-parlanti-description"
                    }
                    className="flex flex-col gap-2"
                  >
                    <legend className="mb-2 font-medium text-sm">
                      {t("settings.recording.parlanti")}
                    </legend>
                    <div className="flex flex-wrap gap-x-5 gap-y-2">
                      {parlantiRegistrazione(settings).map((name) => (
                        <SettingCheckbox
                          key={name}
                          label={t(PARLANTI_LABELS[name])}
                          name={name}
                          onError={setError}
                        />
                      ))}
                    </div>
                    <p
                      className="text-muted-foreground text-sm leading-relaxed"
                      id="recording-parlanti-description"
                    >
                      {t("settings.recording.parlantiDescription")}
                    </p>
                    {settings.diarizer === "nemotron3" ? (
                      <p
                        className="text-muted-foreground text-sm leading-relaxed"
                        id="recording-nemotron-description"
                      >
                        {t(
                          "settings.transcription.experimentalLiveDescription"
                        )}
                      </p>
                    ) : null}
                  </fieldset>
                  <div className="grid grid-cols-3 gap-4 border-t pt-7">
                    <Field id="bitrate" label={t("settings.recording.bitrate")}>
                      <NativeSelect
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
                      id="channels"
                      label={t("settings.recording.channels")}
                    >
                      <NativeSelect
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
                      id="sample-rate"
                      label={t("settings.recording.sampleRate")}
                    >
                      <NativeSelect
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
                </>
              ) : null}
              {section.key === "trascrizione" ? (
                <>
                  <ModelList
                    kind="trascrizione"
                    onSelect={selectModel}
                    selected={settings.model}
                  />
                  <div className="flex flex-col gap-1 border-t pt-7">
                    <h3 className="font-medium text-sm">
                      {t("settings.transcription.diarization")}
                    </h3>
                    <p className="text-muted-foreground text-sm leading-relaxed">
                      {t("settings.transcription.diarizationDescription")}
                    </p>
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
                    <legend className="mb-3 font-medium text-sm">
                      {t("settings.general.theme")}
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
                  <Field
                    description={t("settings.general.languageRestart")}
                    id="interface-language"
                    label={t("settings.general.language")}
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
                  <Field id="copy-format" label={t("settings.general.copyAs")}>
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
            ? t("settings.transcription.experimentalDescription")
            : undefined
        }
        id="diarizer"
        label={t("settings.transcription.diarizer")}
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
        <>
          <Field
            description={t("settings.transcription.localModelDescription")}
            label={t("settings.transcription.localModel")}
          >
            <span className="break-all text-sm">
              {settings.nemotron3Path ??
                t("settings.transcription.localModelMissing")}
            </span>
            <Button className="self-start" onClick={onPick} variant="outline">
              {t("settings.transcription.chooseLocalModel")}
            </Button>
          </Field>
          <p className="text-muted-foreground text-sm leading-relaxed">
            {t("settings.transcription.experimentalLiveDescription")}
          </p>
        </>
      ) : null}
    </>
  );
}

/**
 * Un microfono o un dispositivo di uscita: il predefinito di sistema (`null`) o uno dei rilevati.
 * Un dispositivo salvato ma non collegato resta scelto, come "Non collegato". `devices` è `null`
 * finché l'elenco non arriva.
 */
function DeviceSelect({
  devices,
  disabled,
  id,
  onChange,
  value,
}: {
  devices: AudioDevice[] | null;
  disabled: boolean;
  id: string;
  onChange: (e: ChangeEvent<HTMLSelectElement>) => void;
  value: string | null;
}) {
  const { t } = useTranslation();
  const fallback = devices?.find((d) => d.isDefault);
  const missing =
    devices !== null && value !== null && !devices.some((d) => d.id === value);
  return (
    <NativeSelect
      className="h-9"
      disabled={disabled}
      id={id}
      onChange={onChange}
      value={value ?? ""}
    >
      <option value="">
        {fallback
          ? t("settings.recording.defaultDeviceNamed", { name: fallback.name })
          : t("settings.recording.defaultDevice")}
      </option>
      {devices?.map((d) => (
        <option key={d.id} value={d.id}>
          {d.name}
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

/** Etichetta sopra, controllo, nota sotto (`<id>-description`, per `aria-describedby`). */
function Field({
  children,
  description,
  id,
  label,
}: {
  children: ReactNode;
  description?: string;
  id?: string;
  label: string;
}) {
  return (
    <div className="flex min-w-0 flex-col gap-2">
      {id ? (
        <label className="font-medium text-sm" htmlFor={id}>
          {label}
        </label>
      ) : (
        <span className="font-medium text-sm">{label}</span>
      )}
      {children}
      {description ? (
        <p
          className="text-muted-foreground text-sm leading-relaxed"
          id={id ? `${id}-description` : undefined}
        >
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
