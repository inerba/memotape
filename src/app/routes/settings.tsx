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
  commands,
  type Settings,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import { ModelList } from "@/features/models/model-list";
import {
  BITRATES_KBPS,
  SAMPLE_RATES,
  settingsSchema,
} from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { errorText } from "@/features/status/status";

const SELECT =
  "h-9 rounded-md border border-input bg-transparent px-2 text-sm shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50";

// ponytail: manca la sezione Informazioni e la Lingua dell'interfaccia (ticket 10); la sorgente
// di registrazione e il dispositivo di uscita arrivano con "Audio di sistema" (ticket 09).
export function SettingsPage() {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  const [error, setError] = useState<AppError | null>(null);
  const [microphones, setMicrophones] = useState<AudioDevice[]>([]);
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

  const defaultMicrophone = microphones.find((m) => m.isDefault);
  const savedMissing =
    settings.microphone !== null &&
    !microphones.some((m) => m.id === settings.microphone);

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
            <label className="text-sm" htmlFor="microphone">
              {t("settings.recording.microphone")}
            </label>
            <select
              className={SELECT}
              id="microphone"
              onChange={chooseMicrophone}
              value={settings.microphone ?? ""}
            >
              <option value="">
                {defaultMicrophone
                  ? t("settings.recording.defaultMicrophoneNamed", {
                      name: defaultMicrophone.name,
                    })
                  : t("settings.recording.defaultMicrophone")}
              </option>
              {microphones.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
              {savedMissing ? (
                <option value={settings.microphone ?? ""}>
                  {t("settings.recording.missingMicrophone")}
                </option>
              ) : null}
            </select>
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
          description={t("settings.general.folderDescription")}
          id="settings-general"
          title={t("settings.general.title")}
        >
          <div className="flex max-w-xl flex-col gap-2">
            <span className="text-sm">{t("settings.general.folder")}</span>
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
        </Section>
      </main>
    </div>
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
