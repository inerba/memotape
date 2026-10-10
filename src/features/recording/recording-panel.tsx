import {
  ChevronsDownUp,
  ChevronsUpDown,
  Circle,
  Pause,
  Play,
  Square,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type AppError,
  commands,
  events,
  type Levels,
  type RecordingCleaningFailed,
  type RecordingCleaningPreparing,
  type RecordingTick,
} from "@/bindings";
import { FieldHelp } from "@/components/field-help";
import { Button } from "@/components/ui/button";
import {
  BanMark,
  MicFill,
  SpeakerFill,
} from "@/features/recording/ingresso-icons";
import { LevelMeter } from "@/features/recording/level-meter";
import {
  elapsedText,
  meters,
  silentLevels,
} from "@/features/recording/recording";
import { CleaningProfile } from "@/features/settings/cleaning-profile";
import { GuadagnoSelect } from "@/features/settings/guadagno-select";
import { useSettings } from "@/features/settings/settings-context";
import {
  ariaTasti,
  conTasti,
  useScorciatoia,
} from "@/features/shortcuts/shortcuts-provider";
import { cn } from "@/lib/utils";

const AUDIO_INPUTS = {
  microphone: {
    guadagno: "guadagnoMicrofono",
    icon: MicFill,
    ingresso: "microfono",
    name: "settings.recording.inputs.mic",
    profilo: "audioMicrofono",
  },
  system: {
    guadagno: "guadagnoSistema",
    icon: SpeakerFill,
    ingresso: "sistema",
    name: "settings.recording.inputs.system",
    profilo: "audioSistema",
  },
} as const;

/** Dove si ricorda, su questo PC, se la barra è ridotta. */
const COLLAPSED_KEY = "memotape.recordingBar";

function savedCollapsed(): boolean {
  try {
    return localStorage.getItem(COLLAPSED_KEY) === "collapsed";
  } catch {
    return false;
  }
}

/**
 * La Registrazione in corso: timer, Pausa/Riprendi e Stop, e per ogni ingresso l'icona (un clic lo
 * mette in Muto), il livello con il suo Guadagno, Filtra rumore e Sensibilità. Ridotta, resta una
 * riga con timer, icone e livelli. L'esito arriva a chi ha chiamato `record`; `onPausedChange`
 * riceve la pausa confermata dal backend; gli esiti del salvataggio restano accanto ai controlli.
 * Preparazioni e guasti della pulizia arrivano già filtrati per la Registrazione in corso.
 */
export function RecordingPanel({
  cleaningFailures = [],
  cleaningPreparing = [],
  onError,
  onPausedChange,
  paused,
  sessionId,
}: {
  cleaningFailures?: RecordingCleaningFailed[];
  cleaningPreparing?: RecordingCleaningPreparing[];
  onError: (error: AppError) => void;
  onPausedChange: (paused: boolean) => void;
  paused: boolean;
  sessionId?: string;
}) {
  const { t } = useTranslation();
  const { settings } = useSettings();
  // L'ultimo `recording-tick`; prima, gli indicatori degli ingressi scelti.
  const [tick, setTick] = useState<RecordingTick>(() => ({
    elapsedMs: 0,
    levels: silentLevels(settings.recordingSource),
    sessionId: sessionId ?? "",
  }));
  const [stopping, setStopping] = useState(false);
  const [collapsed, setCollapsed] = useState(savedCollapsed);
  // Ogni Registrazione parte senza Muto.
  const [muti, setMuti] = useState<Record<keyof Levels, boolean>>({
    microphone: false,
    system: false,
  });
  const reportError = useCallback(
    (error: AppError | null) => {
      if (error) {
        onError(error);
      }
    },
    [onError]
  );

  useEffect(() => {
    const ticks = events.recordingTick.listen(({ payload }) => {
      if (payload.sessionId === sessionId) {
        setTick(payload);
      }
    });
    return () => {
      ticks.then((unlisten) => unlisten());
    };
  }, [sessionId]);

  const togglePause = useCallback(async () => {
    if (await commands.pauseRecording(!paused)) {
      onPausedChange(!paused);
    }
  }, [onPausedChange, paused]);
  useScorciatoia("pausaRegistrazione", stopping ? null : togglePause);

  const stop = useCallback(async () => {
    setStopping(true);
    // `false`: la Registrazione è già finita, l'esito arriva comunque da `record`.
    await commands.stopRecording();
  }, []);

  const toggleCollapsed = useCallback(() => {
    setCollapsed((current) => {
      try {
        localStorage.setItem(COLLAPSED_KEY, current ? "expanded" : "collapsed");
      } catch {
        // Senza memoria del browser la scelta vale finché la barra resta aperta.
      }
      return !current;
    });
  }, []);

  const toggleMuto = useCallback(
    async (input: keyof Levels) => {
      const next = !muti[input];
      if (await commands.setMuto(AUDIO_INPUTS[input].ingresso, next)) {
        setMuti((current) => ({ ...current, [input]: next }));
      }
    },
    [muti]
  );

  const levels = meters(tick.levels);
  const nameOf = (input: keyof Levels) => t(AUDIO_INPUTS[input].name);
  return (
    <section
      aria-label={t("recording.title")}
      className={cn(
        "mx-auto w-full max-w-[52rem] rounded-2xl border bg-card px-4 shadow-float",
        collapsed ? "py-2" : "pt-2.5 pb-0.5"
      )}
    >
      <div
        className={cn(
          "flex min-w-0 items-center gap-2.5",
          collapsed ? "" : "pb-2.5"
        )}
      >
        <Circle
          aria-hidden
          className={
            paused
              ? "size-[9px] shrink-0 fill-muted-foreground text-muted-foreground"
              : "size-[9px] shrink-0 fill-destructive text-destructive motion-safe:animate-pulse"
          }
        />
        <span className="w-[4.25rem] shrink-0 font-medium text-xl tabular-nums">
          <span className="sr-only">{t("recording.elapsed")} </span>
          {elapsedText(tick.elapsedMs)}
        </span>
        {collapsed ? (
          <div className="flex min-w-0 flex-1 items-center gap-4 pl-1">
            {levels.map(({ input, percent }) => (
              <div
                className="flex min-w-20 max-w-56 flex-1 items-center gap-2"
                key={input}
              >
                <MutoButton
                  disabled={stopping}
                  input={input}
                  mini
                  muted={muti[input]}
                  name={nameOf(input)}
                  onToggle={toggleMuto}
                />
                <LevelMeter
                  label={t("recording.level", { input: nameOf(input) })}
                  mini
                  muted={muti[input]}
                  paused={paused}
                  percent={paused ? 0 : percent}
                />
              </div>
            ))}
          </div>
        ) : (
          <p className="min-w-0 flex-1 truncate text-muted-foreground text-sm">
            {t(paused ? "status.recordingPaused" : "recording.started")}
          </p>
        )}
        <Button
          aria-expanded={!collapsed}
          aria-label={t(collapsed ? "recording.expand" : "recording.collapse")}
          className="size-[30px] text-foreground/80"
          onClick={toggleCollapsed}
          size="icon"
          title={t(collapsed ? "recording.expand" : "recording.collapse")}
          variant="ghost"
        >
          {collapsed ? <ChevronsUpDown /> : <ChevronsDownUp />}
        </Button>
        <Button
          aria-keyshortcuts={ariaTasti("pausaRegistrazione")}
          className="h-[30px] px-3 text-sm [&_svg]:size-3.5"
          disabled={stopping}
          onClick={togglePause}
          title={conTasti(
            t,
            paused ? t("recording.resume") : t("recording.pause"),
            "pausaRegistrazione"
          )}
          variant="outline"
        >
          {paused ? <Play /> : <Pause />}
          {paused ? t("recording.resume") : t("recording.pause")}
        </Button>
        <Button
          className="h-[30px] px-3 text-sm [&_svg]:size-2.5"
          disabled={stopping}
          onClick={stop}
        >
          <Square className="fill-current" />
          {stopping ? t("recording.stopping") : t("recording.stop")}
        </Button>
      </div>
      <div
        className="grid transition-[grid-template-rows] duration-[260ms] ease-[cubic-bezier(0.2,0.8,0.2,1)] motion-reduce:transition-none"
        inert={collapsed}
        style={{ gridTemplateRows: collapsed ? "0fr" : "1fr" }}
      >
        <div className="-mx-1 min-h-0 overflow-hidden px-1">
          {levels.map(({ input, percent }) => {
            const audio = AUDIO_INPUTS[input];
            const name = nameOf(input);
            const muted = muti[input];
            return (
              <div
                className="grid min-w-0 grid-cols-[2.25rem_1fr_11.5rem] items-center gap-x-3.5 gap-y-1.5 border-t py-2.5"
                key={input}
              >
                <div className="row-span-2 justify-self-center">
                  <MutoButton
                    disabled={stopping}
                    input={input}
                    muted={muted}
                    name={name}
                    onToggle={toggleMuto}
                  />
                </div>
                <LevelMeter
                  label={t("recording.level", { input: name })}
                  muted={muted}
                  paused={paused}
                  percent={paused ? 0 : percent}
                />
                {/* Si cambia anche in Pausa, non dopo Stop: l'audio è già tutto scritto. */}
                <div className="flex items-center justify-end gap-1.5">
                  {muted ? (
                    <span className="whitespace-nowrap text-destructive text-xs">
                      {t("recording.muted")}
                    </span>
                  ) : null}
                  <GuadagnoSelect
                    aria-label={t("recording.guadagno", { input: name })}
                    className="h-[26px] w-[5.25rem] text-xs"
                    disabled={stopping}
                    name={audio.guadagno}
                    onError={onError}
                  />
                  <FieldHelp label={t("recording.guadagno", { input: name })}>
                    {t("settings.recording.gainDescription")}
                  </FieldHelp>
                </div>
                <div className="col-span-2 col-start-2 min-w-0">
                  {settings[audio.profilo]?.pulizia &&
                  cleaningPreparing.some(
                    (item) => item.ingresso === audio.ingresso && item.preparing
                  ) &&
                  !cleaningFailures.some(
                    (item) => item.ingresso === audio.ingresso
                  ) ? (
                    <p
                      className="mb-1 text-muted-foreground text-xs"
                      role="status"
                    >
                      {t("recording.cleaningPreparing")}
                    </p>
                  ) : null}
                  <CleaningProfile
                    bypass={cleaningFailures.some(
                      (failure) => failure.ingresso === audio.ingresso
                    )}
                    disabled={stopping}
                    layout="bar"
                    name={audio.profilo}
                    onError={reportError}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}

/**
 * L'icona di un Ingresso, che è anche il suo Muto: un clic lo mette in Muto e la sbarra con il
 * segno di divieto, un altro lo toglie.
 */
function MutoButton({
  disabled,
  input,
  mini = false,
  muted,
  name,
  onToggle,
}: {
  disabled: boolean;
  input: keyof Levels;
  mini?: boolean;
  muted: boolean;
  name: string;
  onToggle: (input: keyof Levels) => void;
}) {
  const { t } = useTranslation();
  const Icon = AUDIO_INPUTS[input].icon;
  const toggle = useCallback(() => onToggle(input), [input, onToggle]);
  return (
    <button
      aria-label={t("recording.mute", { input: name })}
      aria-pressed={muted}
      className="grid place-items-center rounded-lg p-1 text-foreground transition-colors duration-150 ease-out hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:opacity-50"
      disabled={disabled}
      onClick={toggle}
      title={t(muted ? "recording.mutedHint" : "recording.muteHint", {
        input: name,
      })}
      type="button"
    >
      <Icon
        className={cn(
          "col-start-1 row-start-1 transition-opacity duration-150",
          mini ? "size-[18px]" : "size-8",
          muted && "opacity-40"
        )}
      />
      <BanMark
        className={cn(
          "col-start-1 row-start-1 text-destructive transition-[opacity,scale] duration-200 ease-[cubic-bezier(0.2,0.8,0.2,1)] motion-reduce:transition-none",
          mini ? "size-6" : "size-[38px]",
          muted ? "scale-100 opacity-100" : "scale-75 opacity-0"
        )}
      />
    </button>
  );
}
