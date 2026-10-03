import { zodResolver } from "@hookform/resolvers/zod";
import { ArrowLeft } from "lucide-react";
import {
  type ChangeEvent,
  type ReactNode,
  useCallback,
  useEffect,
  useState,
} from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";
import {
  type AppError,
  type AudioDevice,
  type CopiaCome,
  commands,
  type ModalitaDalVivo,
  type RecordingSource,
  type Settings,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import { About } from "@/features/about/about";
import { ModelList } from "@/features/models/model-list";
import {
  BITRATES_KBPS,
  LANGUAGE_NAMES,
  LANGUAGES,
  SAMPLE_RATES,
  settingsSchema,
} from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { errorText } from "@/features/status/status";

const SELECT =
  "h-9 rounded-md border border-input bg-transparent px-2 text-sm shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50";

const SOURCES: RecordingSource[] = ["mic", "system", "both"];
const COPY_FORMATS: CopiaCome[] = ["testo", "markdown"];
const LIVE_MODES: ModalitaDalVivo[] = ["mix", "ingressiSeparati"];

export function SettingsPage() {
  const { i18n, t } = useTranslation();
  const { save, settings } = useSettings();
  const [error, setError] = useState<AppError | null>(null);
  // `null` finché `list_microphones` e `list_output_devices` non rispondono.
  const [microphones, setMicrophones] = useState<AudioDevice[] | null>(null);
  const [outputs, setOutputs] = useState<AudioDevice[] | null>(null);
  const [folder, setFolder] = useState("");
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
  const chooseSource = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({
        recordingSource:
          SOURCES.find((s) => s === e.target.value) ?? settings.recordingSource,
      }),
    [choose, settings.recordingSource]
  );
  const chooseLiveMode = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) =>
      choose({
        modalitaDalVivo:
          LIVE_MODES.find((m) => m === e.target.value) ??
          settings.modalitaDalVivo,
      }),
    [choose, settings.modalitaDalVivo]
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
      <header className="flex items-center gap-3 border-b px-6 py-3">
        <Button asChild size="sm" variant="ghost">
          <Link to="/">
            <ArrowLeft />
            {t("settings.back")}
          </Link>
        </Button>
        <h1 className="font-semibold text-lg">{t("settings.title")}</h1>
      </header>
      <main className="flex min-h-0 flex-1 flex-col gap-8 overflow-y-auto p-6">
        {error ? (
          <p className="text-destructive text-sm" role="alert">
            {errorText(error, t)}
          </p>
        ) : null}
        <Section
          description={t("settings.recording.description")}
          id="settings-recording"
          title={t("settings.recording.title")}
        >
          <div className="grid max-w-xl grid-cols-[auto_1fr] items-center gap-x-4 gap-y-3">
            <label className="text-sm" htmlFor="recording-source">
              {t("settings.recording.input")}
            </label>
            <select
              className={SELECT}
              id="recording-source"
              onChange={chooseSource}
              value={settings.recordingSource}
            >
              {SOURCES.map((value) => (
                <option key={value} value={value}>
                  {t(`settings.recording.inputs.${value}`)}
                </option>
              ))}
            </select>
            <label className="text-sm" htmlFor="microphone">
              {t("settings.recording.microphone")}
            </label>
            <DeviceSelect
              devices={microphones}
              disabled={settings.recordingSource === "system"}
              id="microphone"
              onChange={chooseMicrophone}
              value={settings.microphone}
            />
            <label className="text-sm" htmlFor="output-device">
              {t("settings.recording.outputDevice")}
            </label>
            <DeviceSelect
              devices={outputs}
              disabled={settings.recordingSource === "mic"}
              id="output-device"
              onChange={chooseOutput}
              value={settings.outputDevice}
            />
            <label className="text-sm" htmlFor="live-mode">
              {t("settings.recording.liveMode")}
            </label>
            <select
              aria-describedby="live-mode-description"
              className={SELECT}
              disabled={settings.recordingSource !== "both"}
              id="live-mode"
              onChange={chooseLiveMode}
              value={settings.modalitaDalVivo ?? "mix"}
            >
              {LIVE_MODES.map((value) => (
                <option key={value} value={value}>
                  {t(`settings.recording.liveModes.${value}`)}
                </option>
              ))}
            </select>
            <p
              className="col-start-2 text-muted-foreground text-sm"
              id="live-mode-description"
            >
              {t("settings.recording.liveModeDescription")}
            </p>
            <label className="text-sm" htmlFor="bitrate">
              {t("settings.recording.bitrate")}
            </label>
            <select
              className={SELECT}
              id="bitrate"
              onChange={chooseBitrate}
              value={settings.bitrateKbps}
            >
              {BITRATES_KBPS.map((value) => (
                <option key={value} value={value}>
                  {t("settings.recording.kbps", { value })}
                </option>
              ))}
            </select>
            <label className="text-sm" htmlFor="channels">
              {t("settings.recording.channels")}
            </label>
            <select
              className={SELECT}
              id="channels"
              onChange={chooseChannels}
              value={settings.channels}
            >
              <option value="mono">{t("settings.recording.mono")}</option>
              <option value="stereo">{t("settings.recording.stereo")}</option>
            </select>
            <label className="text-sm" htmlFor="sample-rate">
              {t("settings.recording.sampleRate")}
            </label>
            <select
              className={SELECT}
              id="sample-rate"
              onChange={chooseSampleRate}
              value={settings.sampleRate}
            >
              {SAMPLE_RATES.map((value) => (
                <option key={value} value={value}>
                  {t("settings.recording.hz", { value })}
                </option>
              ))}
            </select>
          </div>
        </Section>
        <Section
          description={t("settings.transcription.description")}
          id="settings-transcription"
          title={t("settings.transcription.title")}
        >
          <ModelList onSelect={selectModel} selected={settings.model} />
        </Section>
        <Section
          description={t("settings.general.description")}
          id="settings-general"
          title={t("settings.general.title")}
        >
          <div className="flex max-w-xl flex-col gap-2">
            <span className="text-sm">{t("settings.general.folder")}</span>
            <p className="text-muted-foreground text-sm">
              {t("settings.general.folderDescription")}
            </p>
            <div className="flex items-center gap-2">
              <span
                className="min-w-0 flex-1 truncate rounded-md border px-3 py-2 text-sm"
                title={folder}
              >
                {folder}
              </span>
              <Button onClick={pickFolder} variant="outline">
                {t("settings.general.choose")}
              </Button>
            </div>
            {settings.recordingsFolder === null ? null : (
              <Button
                className="self-start"
                onClick={resetFolder}
                size="sm"
                variant="ghost"
              >
                {t("settings.general.reset")}
              </Button>
            )}
          </div>
          <div className="flex max-w-xl flex-col gap-2">
            <label className="text-sm" htmlFor="copy-format">
              {t("settings.general.copyAs")}
            </label>
            <select
              className={`${SELECT} self-start`}
              id="copy-format"
              onChange={chooseCopyFormat}
              value={settings.copiaCome ?? "testo"}
            >
              {COPY_FORMATS.map((value) => (
                <option key={value} value={value}>
                  {t(`settings.general.copyFormats.${value}`)}
                </option>
              ))}
            </select>
          </div>
          <div className="flex max-w-xl flex-col gap-2">
            <label className="text-sm" htmlFor="interface-language">
              {t("settings.general.language")}
            </label>
            <select
              aria-describedby="interface-language-restart"
              className={`${SELECT} self-start`}
              id="interface-language"
              onChange={chooseLanguage}
              value={settings.interfaceLanguage ?? i18n.language}
            >
              {LANGUAGES.map((value) => (
                <option key={value} lang={value} value={value}>
                  {LANGUAGE_NAMES[value]}
                </option>
              ))}
            </select>
            <p
              className="text-muted-foreground text-sm"
              id="interface-language-restart"
            >
              {t("settings.general.languageRestart")}
            </p>
          </div>
        </Section>
        <Section
          description={t("about.description")}
          id="settings-about"
          title={t("about.title")}
        >
          <About />
        </Section>
      </main>
    </div>
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
    <select
      className={SELECT}
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
    </select>
  );
}

function Section({
  children,
  description,
  id,
  title,
}: {
  children: ReactNode;
  description: string;
  id: string;
  title: string;
}) {
  return (
    <section aria-labelledby={id} className="flex flex-col gap-3">
      <div>
        <h2 className="font-medium" id={id}>
          {title}
        </h2>
        <p className="text-muted-foreground text-sm">{description}</p>
      </div>
      {children}
    </section>
  );
}
