import { ChevronDown, Mic } from "lucide-react";
import { type ChangeEvent, useCallback, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, type AudioDevice, commands } from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { MicFill, SpeakerFill } from "@/features/recording/ingresso-icons";
import { CleaningProfile } from "@/features/settings/cleaning-profile";
import { DeviceSelect } from "@/features/settings/device-select";
import {
  SettingSwitch,
  SWITCH_CLASS,
} from "@/features/settings/setting-switch";
import {
  type ParlantiRegistrazione,
  type RecordingInput,
  withInput,
} from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { ariaTasti, conTasti } from "@/features/shortcuts/shortcuts-provider";

/** Microfono e Audio di sistema: icona, profilo audio e scelta del dispositivo. */
const INPUTS = {
  mic: {
    device: "microphone",
    icon: MicFill,
    profilo: "audioMicrofono",
  },
  system: {
    device: "outputDevice",
    icon: SpeakerFill,
    profilo: "audioSistema",
  },
} as const;

/** I dispositivi rilevati per ogni Ingresso; `null` finché l'elenco non arriva. */
interface Devices {
  mic: AudioDevice[] | null;
  system: AudioDevice[] | null;
}

/**
 * Nuova registrazione ▾, l'azione principale della barra laterale. Il menu ha un riquadro per
 * Ingresso: nella testata l'interruttore "Registra da" e, se acceso, sotto con un solo rientro il
 * dispositivo, Filtra rumore, Sensibilità e Riconosci i parlanti (che vale solo dal vivo). In fondo,
 * separato, Trascrivi dal vivo con la sua spiegazione. I dispositivi si rileggono a ogni apertura.
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
  const [devices, setDevices] = useState<Devices>({ mic: null, system: null });
  const loadDevices = useCallback(async () => {
    const [mic, system] = await Promise.all([
      commands.listMicrophones(),
      commands.listOutputDevices(),
    ]);
    for (const result of [mic, system]) {
      if (result.status === "error") {
        onError(result.error);
      }
    }
    setDevices({
      mic: mic.status === "ok" ? mic.data : [],
      system: system.status === "ok" ? system.data : [],
    });
  }, [onError]);
  return (
    <div className="flex">
      <Button
        aria-keyshortcuts={ariaTasti("nuovaRegistrazione")}
        className="h-10 flex-1 justify-start gap-2.5 rounded-r-none pl-3.5 text-[0.9375rem]"
        disabled={disabled}
        onClick={onRecord}
        title={conTasti(t, t("sidebar.newRecording"), "nuovaRegistrazione")}
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
        onOpen={loadDevices}
        panelClassName="w-[22rem]"
        variant="default"
      >
        <div className="flex flex-col gap-2 p-0.5">
          <InputGroup devices={devices.mic} input="mic" onError={onError} />
          <InputGroup
            devices={devices.system}
            input="system"
            onError={onError}
          />
          <div className="flex flex-col gap-0.5 border-t px-2.5 pt-2 pb-1">
            <div className="font-medium">
              <SettingSwitch
                end
                inlineFeedback={false}
                label={t("recording.live")}
                name="trascrizioneDalVivo"
                onError={onError}
              />
            </div>
            <p className="text-muted-foreground text-xs">
              {t("recording.liveHint")}
            </p>
          </div>
        </div>
      </PopoverMenu>
    </div>
  );
}

/**
 * Il riquadro di un Ingresso nel menu: le sue opzioni si vedono solo se l'Ingresso è acceso; spento
 * resta la sola testata, che dice "Spento" senza sembrare disabilitata.
 */
function InputGroup({
  devices,
  input,
  onError,
}: {
  devices: AudioDevice[] | null;
  input: RecordingInput;
  onError: (error: AppError) => void;
}) {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  const id = useId();
  const { device, icon: Icon, profilo } = INPUTS[input];
  const source = settings.recordingSource;
  const on = source === "both" || source === input;
  // Serve almeno una fonte: l'ultima accesa non si spegne.
  const last = on && source !== "both";
  const live = settings.trascrizioneDalVivo ?? false;
  const report = useCallback(
    (error: AppError | null) => {
      if (error) {
        onError(error);
      }
    },
    [onError]
  );
  const toggle = useCallback(
    async (checked: boolean) => {
      report(
        await save(
          (current) => ({
            ...current,
            recordingSource: withInput(current.recordingSource, input, checked),
          }),
          "recordingSource"
        )
      );
    },
    [input, report, save]
  );
  const chooseDevice = useCallback(
    async (e: ChangeEvent<HTMLSelectElement>) => {
      const value = e.target.value || null;
      report(
        await save((current) => ({ ...current, [device]: value }), device)
      );
    },
    [device, report, save]
  );
  const label = t(`settings.recording.inputs.${input}`);
  let parlanti: ParlantiRegistrazione = "parlantiMix";
  if (source === "both") {
    parlanti = input === "mic" ? "parlantiMicrofono" : "parlantiSistema";
  }
  return (
    <div className="flex flex-col gap-1.5 rounded-lg border p-1.5">
      <label
        className="flex min-h-8 cursor-pointer items-center gap-2.5 px-1 font-medium has-[:disabled]:cursor-default"
        htmlFor={id}
        title={last ? t("settings.recording.lastInput") : undefined}
      >
        <Icon className="size-4 shrink-0" />
        <span className="flex-1">{label}</span>
        {on ? null : (
          <span
            aria-hidden
            className="font-normal text-muted-foreground text-xs"
          >
            {t("recording.inputOff")}
          </span>
        )}
        <Switch
          checked={on}
          className={SWITCH_CLASS}
          disabled={last}
          id={id}
          onCheckedChange={toggle}
        />
      </label>
      {on ? (
        // Un solo rientro (30 px) per tutte le opzioni: partono dove inizia il nome dell'Ingresso.
        <div className="motion-safe:fade-in-0 motion-safe:slide-in-from-top-1 flex flex-col gap-2 pt-0.5 pr-1 pb-1.5 pl-[1.875rem] motion-safe:animate-in">
          <DeviceSelect
            className="h-[1.875rem] text-[0.8125rem]"
            compact
            devices={devices}
            label={t("settings.recording.deviceFor", { input: label })}
            onChange={chooseDevice}
            value={settings[device]}
          />
          <CleaningProfile
            inlineFeedback={false}
            layout="menu"
            name={profilo}
            onError={report}
          />
          <SettingSwitch
            disabled={!live}
            end
            inlineFeedback={false}
            label={t("settings.recording.parlanti")}
            name={parlanti}
            onError={onError}
          />
        </div>
      ) : null}
    </div>
  );
}
