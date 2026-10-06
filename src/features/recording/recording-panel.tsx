import { Circle, Pause, Play, Square } from "lucide-react";
import { Fragment, useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type AppError,
  commands,
  events,
  type RecordingTick,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  elapsedText,
  meters,
  silentLevels,
} from "@/features/recording/recording";
import { GuadagnoSelect } from "@/features/settings/guadagno-select";
import { useSettings } from "@/features/settings/settings-context";
import { inEventSession } from "@/lib/event-session";

/**
 * La Registrazione in corso: timer, un livello per ingresso con il suo Guadagno, Pausa/Riprendi e
 * Stop. L'esito arriva a chi ha chiamato `record`; `onPausedChange` riceve la pausa confermata dal
 * backend, `onError` l'errore di un Guadagno non salvato.
 */
export function RecordingPanel({
  onError,
  onPausedChange,
  paused,
  sessionId,
}: {
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
      className="mx-auto flex w-full max-w-[52rem] items-center gap-5 rounded-2xl border bg-card px-5 py-3.5 shadow-float"
    >
      <span className="flex items-center gap-3">
        <Circle
          aria-hidden
          className={
            paused
              ? "size-3 shrink-0 fill-muted-foreground text-muted-foreground"
              : "size-3 shrink-0 animate-pulse fill-destructive text-destructive"
          }
        />
        <span className="w-[4.5rem] font-medium text-2xl tabular-nums">
          <span className="sr-only">{t("recording.elapsed")} </span>
          {elapsedText(tick.elapsedMs)}
        </span>
      </span>
      <div className="grid min-w-0 flex-1 grid-cols-[auto_1fr_auto] items-center gap-x-3 gap-y-1.5">
        {meters(tick.levels).map(({ input, percent }) => (
          <Fragment key={input}>
            <span aria-hidden className="text-muted-foreground text-xs">
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
            <GuadagnoSelect
              aria-label={t("recording.guadagno", {
                input: t(`recording.levels.${input}`),
              })}
              className="h-7 text-xs"
              disabled={stopping}
              name={
                input === "microphone" ? "guadagnoMicrofono" : "guadagnoSistema"
              }
              onError={onError}
            />
          </Fragment>
        ))}
      </div>
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
