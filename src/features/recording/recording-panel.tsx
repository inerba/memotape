import { Circle, Pause, Play, Square } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands, events } from "@/bindings";
import { Button } from "@/components/ui/button";
import { elapsedText, levelPercent } from "@/features/recording/recording";

/**
 * La Registrazione in corso: timer, livello, Pausa/Riprendi e Stop. L'esito arriva a chi ha
 * chiamato `record`; `onPausedChange` riceve la pausa confermata dal backend.
 */
export function RecordingPanel({
  onPausedChange,
  paused,
}: {
  onPausedChange: (paused: boolean) => void;
  paused: boolean;
}) {
  const { t } = useTranslation();
  // Durata e livello dall'ultimo `recording-tick`.
  const [tick, setTick] = useState({ elapsedMs: 0, level: 0 });
  const [stopping, setStopping] = useState(false);

  useEffect(() => {
    const ticks = events.recordingTick.listen(({ payload }) => {
      setTick({
        elapsedMs: payload.elapsedMs,
        level: levelPercent(payload.levels[0] ?? null),
      });
    });
    return () => {
      ticks.then((unlisten) => unlisten());
    };
  }, []);

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
      className="flex items-center gap-4 rounded-md border px-4 py-3"
    >
      <Circle
        aria-hidden
        className={
          paused
            ? "size-3 shrink-0 fill-muted-foreground text-muted-foreground"
            : "size-3 shrink-0 animate-pulse fill-destructive text-destructive"
        }
      />
      <span className="font-medium text-lg tabular-nums">
        <span className="sr-only">{t("recording.elapsed")} </span>
        {elapsedText(tick.elapsedMs)}
      </span>
      {/* Verde fino a -15 dBFS, poi giallo, rosso vicino alla saturazione. */}
      <meter
        aria-label={t("recording.level")}
        className="h-3 min-w-0 flex-1"
        high={95}
        low={75}
        max={100}
        min={0}
        optimum={0}
        value={paused ? 0 : tick.level}
      />
      <Button disabled={stopping} onClick={togglePause} variant="outline">
        {paused ? <Play /> : <Pause />}
        {paused ? t("recording.resume") : t("recording.pause")}
      </Button>
      <Button disabled={stopping} onClick={stop}>
        <Square />
        {stopping ? t("recording.stopping") : t("recording.stop")}
      </Button>
    </section>
  );
}
