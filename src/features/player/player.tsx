import { convertFileSrc } from "@tauri-apps/api/core";
import {
  Pause,
  Play,
  RotateCcw,
  RotateCw,
  Volume2,
  VolumeX,
} from "lucide-react";
import {
  type ChangeEvent,
  type PointerEvent,
  type ReactNode,
  type RefObject,
  type SyntheticEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import {
  type Follow,
  type FollowEvent,
  nextFollow,
} from "@/features/player/sync";
import { elapsedText } from "@/features/recording/recording";

const RATES = [1, 1.25, 1.5, 2];
const SKIP_MS = 10_000;
/** Le barre della forma d'onda: abbastanza fitte per la larghezza del player. */
const BARS = 180;
/** Dove resta il volume scelto: una comodità di questo PC, non un'impostazione. */
const VOLUME_KEY = "sbobino.volume";

/** Lo stato del player di un Tape, condiviso tra il player e il testo che lo segue. */
export interface PlayerState {
  audio: RefObject<HTMLAudioElement | null>;
  /** La durata dal mix; prima dei metadati quella del Tape. */
  durationMs: number;
  follow: Follow;
  /** Porta il player a `ms` senza cambiare Play/Pausa; il testo torna a seguire l'audio. */
  move: (ms: number, event: Exclude<FollowEvent, "scroll">) => void;
  onFollow: (event: FollowEvent) => void;
  /** Porta il player a `ms` e lo avvia: l'ascolto di un turno e Riascolta. */
  playFrom: (ms: number) => void;
  playing: boolean;
  positionMs: number;
  setDurationMs: (ms: number) => void;
  setPlaying: (playing: boolean) => void;
  setPositionMs: (ms: number) => void;
}

/** Il player di un Tape lungo `durataMs`. */
export function usePlayer(durataMs: number): PlayerState {
  const audio = useRef<HTMLAudioElement>(null);
  const [positionMs, setPositionMs] = useState(0);
  const [durationMs, setDurationMs] = useState(durataMs);
  const [playing, setPlaying] = useState(false);
  const [follow, setFollow] = useState<Follow>("following");
  const onFollow = useCallback(
    (event: FollowEvent) => setFollow(nextFollow(event)),
    []
  );
  const move = useCallback(
    (ms: number, event: Exclude<FollowEvent, "scroll">) => {
      const target = Math.max(0, ms);
      // Prima dei metadati vale come posizione di partenza.
      if (audio.current) {
        audio.current.currentTime = target / 1000;
      }
      setPositionMs(target);
      onFollow(event);
    },
    [onFollow]
  );
  const playFrom = useCallback(
    (ms: number) => {
      move(ms, "jump");
      audio.current?.play().catch(() => setPlaying(false));
    },
    [move]
  );
  return {
    audio,
    durationMs,
    follow,
    move,
    onFollow,
    playFrom,
    playing,
    positionMs,
    setDurationMs,
    setPlaying,
    setPositionMs,
  };
}

/** Se con il focus su `target` Spazio non è per il player: si scrive, o attiva un controllo. */
function spaceTaken(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    target.closest(
      "textarea, select, button, a, [contenteditable], input:not([type=range])"
    ) !== null
  );
}

/** Un clic sui pulsanti del player non sposta il focus: Spazio resta Play/Pausa. */
export function keepFocus(e: PointerEvent) {
  e.preventDefault();
}

function savedVolume(): number {
  try {
    const value = Number(localStorage.getItem(VOLUME_KEY) ?? "1");
    return Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : 1;
  } catch {
    return 1;
  }
}

const ICON_BUTTON =
  "flex size-8 shrink-0 items-center justify-center rounded-md text-foreground/80 transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:opacity-50 [&_svg]:size-4";

/**
 * Il player del mix di un Tape: indietro e avanti di 10 s, Play/Pausa (anche con Spazio), posizione,
 * forma d'onda che fa da barra di avanzamento, durata, velocità e volume. `label` dice che audio è.
 * `disabled` durante una Registrazione, che registrerebbe l'audio riascoltato.
 */
export function Player({
  disabled,
  label,
  path,
  player,
}: {
  disabled: boolean;
  label: string;
  path: string;
  player: PlayerState;
}) {
  const { i18n, t } = useTranslation();
  const [rate, setRate] = useState(1);
  const [volume, setVolume] = useState(savedVolume);
  const [muted, setMuted] = useState(false);
  const {
    audio,
    durationMs,
    move,
    playing,
    positionMs,
    setDurationMs,
    setPlaying,
    setPositionMs,
  } = player;

  const toggle = useCallback(() => {
    const el = audio.current;
    if (!el) {
      return;
    }
    if (el.paused) {
      el.play().catch(() => setPlaying(false));
    } else {
      el.pause();
    }
  }, [audio, setPlaying]);

  useEffect(() => {
    if (disabled) {
      audio.current?.pause();
      return;
    }
    const space = (e: KeyboardEvent) => {
      if (
        e.key !== " " ||
        e.repeat ||
        spaceTaken(e.target) ||
        audio.current?.closest("[inert]")
      ) {
        return;
      }
      e.preventDefault();
      toggle();
    };
    window.addEventListener("keydown", space);
    return () => window.removeEventListener("keydown", space);
  }, [audio, disabled, toggle]);

  useEffect(() => {
    if (audio.current) {
      audio.current.playbackRate = rate;
    }
  }, [audio, rate]);

  useEffect(() => {
    if (audio.current) {
      audio.current.volume = volume;
      audio.current.muted = muted;
    }
    try {
      localStorage.setItem(VOLUME_KEY, String(volume));
    } catch {
      // Senza memoria del browser il volume vale solo finché l'app resta aperta.
    }
  }, [audio, muted, volume]);

  const time = useCallback(
    (e: SyntheticEvent<HTMLAudioElement>) =>
      setPositionMs(e.currentTarget.currentTime * 1000),
    [setPositionMs]
  );
  // `pause` arriva anche a fine audio.
  const playState = useCallback(
    (e: SyntheticEvent<HTMLAudioElement>) =>
      setPlaying(!e.currentTarget.paused),
    [setPlaying]
  );
  const duration = useCallback(
    (e: SyntheticEvent<HTMLAudioElement>) => {
      const seconds = e.currentTarget.duration;
      if (Number.isFinite(seconds)) {
        setDurationMs(seconds * 1000);
      }
    },
    [setDurationMs]
  );
  const back = useCallback(
    () => move(positionMs - SKIP_MS, "seek"),
    [move, positionMs]
  );
  const forward = useCallback(
    () => move(Math.min(positionMs + SKIP_MS, durationMs), "seek"),
    [durationMs, move, positionMs]
  );
  const seek = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => move(Number(e.target.value), "seek"),
    [move]
  );
  const chooseRate = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) => setRate(Number(e.target.value)),
    []
  );
  const chooseVolume = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setVolume(Number(e.target.value));
    setMuted(false);
  }, []);
  const toggleMute = useCallback(() => setMuted((m) => !m), []);
  const rateText = new Intl.NumberFormat(i18n.language);
  const silent = muted || volume === 0;

  return (
    <section
      aria-label={t("player.label")}
      className="mx-auto flex w-full max-w-[52rem] flex-col gap-1 rounded-2xl border bg-card px-4 pt-2.5 pb-3 shadow-float"
    >
      {/* biome-ignore lint/a11y/useMediaCaption: il testo della Trascrizione è accanto */}
      <audio
        onDurationChange={duration}
        onPause={playState}
        onPlay={playState}
        onTimeUpdate={time}
        preload="metadata"
        ref={audio}
        src={convertFileSrc(path, "bino")}
      />
      <div className="flex items-center gap-2 text-muted-foreground text-xs">
        <span className="min-w-0 flex-1 truncate pl-1" title={label}>
          {label}
        </span>
        <select
          aria-label={t("player.rate")}
          className="h-7 cursor-pointer appearance-none rounded-md bg-transparent px-2 text-center text-foreground text-sm tabular-nums transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:opacity-50"
          disabled={disabled}
          onChange={chooseRate}
          title={t("player.rate")}
          value={rate}
        >
          {RATES.map((r) => (
            <option key={r} value={r}>
              {`${rateText.format(r)}×`}
            </option>
          ))}
        </select>
        <button
          aria-label={silent ? t("player.unmute") : t("player.mute")}
          className={ICON_BUTTON}
          disabled={disabled}
          onClick={toggleMute}
          onPointerDown={keepFocus}
          title={silent ? t("player.unmute") : t("player.mute")}
          type="button"
        >
          {silent ? <VolumeX /> : <Volume2 />}
        </button>
        <input
          aria-label={t("player.volume")}
          className="h-1 w-20 cursor-pointer accent-play disabled:cursor-default"
          disabled={disabled}
          max={1}
          min={0}
          onChange={chooseVolume}
          step={0.05}
          type="range"
          value={muted ? 0 : volume}
        />
      </div>
      <div className="flex items-center gap-1.5">
        <button
          aria-label={t("player.back")}
          className={ICON_BUTTON}
          disabled={disabled}
          onClick={back}
          onPointerDown={keepFocus}
          title={t("player.back")}
          type="button"
        >
          <SkipIcon icon={<RotateCcw />} />
        </button>
        <button
          aria-label={playing ? t("player.pause") : t("player.play")}
          className="flex size-11 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground transition-transform duration-150 ease-out hover:scale-[1.04] focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 active:scale-95 disabled:opacity-50 [&_svg]:size-[1.125rem] [&_svg]:fill-current"
          disabled={disabled}
          onClick={toggle}
          onPointerDown={keepFocus}
          title={playing ? t("player.pause") : t("player.play")}
          type="button"
        >
          {playing ? <Pause /> : <Play className="translate-x-px" />}
        </button>
        <button
          aria-label={t("player.forward")}
          className={ICON_BUTTON}
          disabled={disabled}
          onClick={forward}
          onPointerDown={keepFocus}
          title={t("player.forward")}
          type="button"
        >
          <SkipIcon icon={<RotateCw />} />
        </button>
        <span className="w-14 shrink-0 pr-2 text-right text-sm tabular-nums">
          {elapsedText(positionMs)}
        </span>
        <Waveform
          disabled={disabled}
          durationMs={durationMs}
          label={t("player.position")}
          onSeek={seek}
          path={path}
          positionMs={positionMs}
        />
        <span className="w-14 shrink-0 pl-2 text-muted-foreground text-sm tabular-nums">
          {elapsedText(durationMs)}
        </span>
      </div>
    </section>
  );
}

/** La freccia circolare di ±10 s con il numero dentro. */
function SkipIcon({ icon }: { icon: ReactNode }) {
  return (
    <span className="relative flex size-6 items-center justify-center [&_svg]:size-6 [&_svg]:stroke-[1.5]">
      {icon}
      <span className="absolute pt-px font-semibold text-[0.5rem] tabular-nums">
        10
      </span>
    </span>
  );
}

/**
 * La forma d'onda del mix come barra di avanzamento: la parte ascoltata è in salvia. Sopra c'è un
 * cursore nativo trasparente, che dà trascinamento, frecce e lettore di schermo.
 */
function Waveform({
  disabled,
  durationMs,
  label,
  onSeek,
  path,
  positionMs,
}: {
  disabled: boolean;
  durationMs: number;
  label: string;
  onSeek: (e: ChangeEvent<HTMLInputElement>) => void;
  path: string;
  positionMs: number;
}) {
  const peaks = usePeaks(path);
  const position = Math.min(positionMs, durationMs);
  const ratio = durationMs > 0 ? position / durationMs : 0;
  const loudest = Math.max(0.05, ...(peaks ?? []));
  return (
    <div className="relative h-10 min-w-0 flex-1 rounded-md has-[input:focus-visible]:ring-[3px] has-[input:focus-visible]:ring-ring/40">
      <svg
        aria-hidden
        className="absolute inset-0 size-full"
        preserveAspectRatio="none"
        viewBox={`0 0 ${peaks?.length || BARS} 40`}
      >
        {peaks ? (
          peaks.map((peak, i) => {
            // La radice alza il parlato sommesso, che altrimenti sparirebbe accanto ai picchi.
            const height = Math.max(2, Math.sqrt(peak / loudest) * 34);
            return (
              <rect
                className={
                  (i + 0.5) / peaks.length <= ratio
                    ? "fill-play"
                    : "fill-foreground/22"
                }
                height={height}
                // biome-ignore lint/suspicious/noArrayIndexKey: le barre non cambiano ordine
                key={i}
                rx={0.2}
                width={0.56}
                x={i + 0.22}
                y={20 - height / 2}
              />
            );
          })
        ) : (
          <rect
            className="fill-foreground/15"
            height={1}
            width={BARS}
            y={19.5}
          />
        )}
      </svg>
      <span
        aria-hidden
        className="pointer-events-none absolute inset-y-1 w-0.5 -translate-x-1/2 rounded-full bg-foreground"
        style={{ left: `${ratio * 100}%` }}
      >
        <span className="absolute top-1/2 left-1/2 size-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-foreground" />
      </span>
      <input
        aria-label={label}
        aria-valuetext={elapsedText(position)}
        className="absolute inset-0 size-full cursor-pointer opacity-0 disabled:cursor-default"
        disabled={disabled}
        max={durationMs}
        min={0}
        onChange={onSeek}
        step={100}
        type="range"
        value={position}
      />
    </div>
  );
}

/** I picchi del mix di `path`, `null` finché il backend non li ha calcolati (o se non ci riesce). */
function usePeaks(path: string): number[] | null {
  const [peaks, setPeaks] = useState<number[] | null>(null);
  useEffect(() => {
    let stale = false;
    setPeaks(null);
    commands.tapePeaks(path, BARS).then((result) => {
      if (!stale && result.status === "ok") {
        setPeaks(result.data.map((p) => p ?? 0));
      }
    });
    return () => {
      stale = true;
    };
  }, [path]);
  return peaks;
}
