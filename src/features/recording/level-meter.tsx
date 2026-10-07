import { useRef } from "react";
import { cn } from "@/lib/utils";
import { nextPeak, type PeakHold } from "./recording";

/** Le zone dell'indicatore: salvia fino a −15 dBFS, senape fino a −3, mattone oltre. */
const ZONES =
  "linear-gradient(90deg, var(--color-play) 0 75%, var(--color-level-warm) 75% 95%, var(--color-destructive) 95% 100%)";
/** Le tacche ogni 10 dB, nel colore del foglio. */
const TICKS =
  "repeating-linear-gradient(90deg, transparent 0 calc(16.666% - 1px), color-mix(in oklch, var(--color-card) 70%, transparent) calc(16.666% - 1px) 16.666%)";
/** Il tratteggio dell'Ingresso in Muto. */
const MUTED =
  "repeating-linear-gradient(135deg, transparent 0 4px, color-mix(in oklch, var(--color-foreground) 9%, transparent) 4px 6px)";

/**
 * Il livello di un Ingresso (`percent`, 0–100 in scala di decibel) con il segno di picco che
 * resta un secondo e poi scende. In Muto resta visibile in grigio tratteggiato: si vede se si
 * parla. `mini`: la versione sottile della barra ridotta.
 */
export function LevelMeter({
  label,
  mini = false,
  muted = false,
  paused = false,
  percent,
}: {
  label: string;
  mini?: boolean;
  muted?: boolean;
  /** In Pausa l'indicatore è a zero e tenue. */
  paused?: boolean;
  percent: number;
}) {
  const peak = useRef<PeakHold>({ percent: 0, until: 0 });
  peak.current = nextPeak(peak.current, percent, performance.now());
  return (
    // biome-ignore lint/a11y/useSemanticElements: il <meter> nativo non si disegna con zone, tacche e picco in modo affidabile.
    <div
      aria-label={label}
      aria-valuemax={100}
      aria-valuemin={0}
      aria-valuenow={percent}
      className={cn(
        "relative w-full min-w-0 overflow-hidden rounded-full bg-secondary transition-opacity duration-200",
        mini ? "h-1.5" : "h-2",
        paused && "opacity-50"
      )}
      role="meter"
      style={muted ? { backgroundImage: MUTED } : undefined}
    >
      <div
        className={cn(
          "absolute inset-0 transition-[clip-path] duration-100 ease-linear motion-reduce:transition-none",
          muted && "opacity-35 grayscale"
        )}
        style={{
          backgroundImage: ZONES,
          clipPath: `inset(0 ${100 - percent}% 0 0 round 9999px)`,
        }}
      />
      {mini ? null : (
        <div className="absolute inset-0" style={{ backgroundImage: TICKS }} />
      )}
      {peak.current.percent > 0 ? (
        <div
          className={cn(
            "absolute inset-y-0 w-0.5 rounded-full bg-foreground transition-[left] duration-100 ease-linear motion-reduce:transition-none",
            muted ? "opacity-25" : "opacity-70"
          )}
          style={{ left: `calc(${peak.current.percent}% - 2px)` }}
        />
      ) : null}
    </div>
  );
}
