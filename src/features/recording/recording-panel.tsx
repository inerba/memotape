import { Circle, Pause, Play, Square } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type AppError,
  commands,
  events,
  type RecordingCleaningFailed,
  type RecordingCleaningPreparing,
  type RecordingTick,
} from "@/bindings";
import { FieldHelp } from "@/components/field-help";
import { Button } from "@/components/ui/button";
import {
  elapsedText,
  meters,
  silentLevels,
} from "@/features/recording/recording";
import { CleaningProfile } from "@/features/settings/cleaning-profile";
import { GuadagnoSelect } from "@/features/settings/guadagno-select";
import { useSettings } from "@/features/settings/settings-context";
import { inEventSession } from "@/lib/event-session";

const AUDIO_INPUTS = {
  microphone: {
    guadagno: "guadagnoMicrofono",
    ingresso: "microfono",
    profilo: "audioMicrofono",
  },
  system: {
    guadagno: "guadagnoSistema",
    ingresso: "sistema",
    profilo: "audioSistema",
  },
} as const;

/**
 * La Registrazione in corso: timer, un livello per ingresso con il suo Guadagno, Pausa/Riprendi e
 * Stop. L'esito arriva a chi ha chiamato `record`; `onPausedChange` riceve la pausa confermata dal
 * backend; gli esiti del salvataggio restano accanto ai controlli.
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
      if (inEventSession(payload, sessionId)) {
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

  const stop = useCallback(async () => {
    setStopping(true);
    // `false`: la Registrazione è già finita, l'esito arriva comunque da `record`.
    await commands.stopRecording();
  }, []);

  return (
    <section
      aria-label={t("recording.title")}
      className="mx-auto flex w-full max-w-[52rem] flex-wrap items-center gap-3 rounded-2xl border bg-card px-5 py-3.5 shadow-float"
    >
      <p className="w-full text-muted-foreground text-sm">
        {t(paused ? "status.recordingPaused" : "recording.started")}
      </p>
      <span className="flex items-center gap-3">
        <Circle
          aria-hidden
          className={
            paused
              ? "size-3 shrink-0 fill-muted-foreground text-muted-foreground"
              : "size-3 shrink-0 fill-destructive text-destructive motion-safe:animate-pulse"
          }
        />
        <span className="w-[4.5rem] font-medium text-2xl tabular-nums">
          <span className="sr-only">{t("recording.elapsed")} </span>
          {elapsedText(tick.elapsedMs)}
        </span>
      </span>
      <div className="order-last grid w-full min-w-0 gap-3">
        {meters(tick.levels).map(({ input, percent }) => (
          <div
            className="grid min-w-0 grid-cols-[auto_1fr_auto] items-center gap-x-3 gap-y-2 border-t pt-3 first:border-0 first:pt-0"
            key={input}
          >
            <span aria-hidden className="font-medium text-sm">
              {t(`recording.levels.${input}`)}
            </span>
            {/* Verde fino a -15 dBFS, poi giallo, rosso vicino alla saturazione. */}
            <meter
              aria-label={t("recording.level", {
                input: t(`recording.levels.${input}`),
              })}
              className="h-2 w-full min-w-0"
              high={95}
              low={75}
              max={100}
              min={0}
              optimum={0}
              value={paused ? 0 : percent}
            />
            {/* Si cambia anche in Pausa, non dopo Stop: l'audio è già tutto scritto. */}
            <div className="flex items-start gap-1">
              <GuadagnoSelect
                aria-label={t("recording.guadagno", {
                  input: t(`recording.levels.${input}`),
                })}
                className="h-7 text-xs"
                disabled={stopping}
                name={AUDIO_INPUTS[input].guadagno}
                onError={onError}
              />
              <FieldHelp
                label={t("recording.guadagno", {
                  input: t(`recording.levels.${input}`),
                })}
              >
                {t("settings.recording.gainDescription")}
              </FieldHelp>
            </div>
            <div className="col-span-3 min-w-0">
              {settings[AUDIO_INPUTS[input].profilo]?.pulizia &&
              cleaningPreparing.some(
                (item) =>
                  inEventSession(item, sessionId) &&
                  item.ingresso === AUDIO_INPUTS[input].ingresso &&
                  item.preparing
              ) &&
              !cleaningFailures.some(
                (item) =>
                  item.ingresso === AUDIO_INPUTS[input].ingresso &&
                  inEventSession(item, sessionId)
              ) ? (
                <p className="mb-1 text-muted-foreground text-xs" role="status">
                  {t("recording.cleaningPreparing")}
                </p>
              ) : null}
              <CleaningProfile
                bypass={cleaningFailures.some(
                  (failure) =>
                    inEventSession(failure, sessionId) &&
                    failure.ingresso === AUDIO_INPUTS[input].ingresso
                )}
                compact
                disabled={stopping}
                name={AUDIO_INPUTS[input].profilo}
                onError={reportError}
              />
            </div>
          </div>
        ))}
      </div>
      <span className="flex-1" />
      <Button
        className="h-10"
        disabled={stopping}
        onClick={togglePause}
        variant="outline"
      >
        {paused ? <Play /> : <Pause />}
        {paused ? t("recording.resume") : t("recording.pause")}
      </Button>
      <Button className="h-10" disabled={stopping} onClick={stop}>
        <Square className="fill-current" />
        {stopping ? t("recording.stopping") : t("recording.stop")}
      </Button>
    </section>
  );
}
