import { convertFileSrc } from "@tauri-apps/api/core";
import { Pause, Play, RotateCcw, RotateCw } from "lucide-react";
import {
  type ChangeEvent,
  type PointerEvent,
  type RefObject,
  type SyntheticEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { SELECT } from "@/features/library/move-select";
import {
  type Follow,
  type FollowEvent,
  nextFollow,
} from "@/features/player/sync";
import { elapsedText } from "@/features/recording/recording";

const RATES = [1, 1.25, 1.5, 2];
const SKIP_MS = 10_000;

/** Lo stato del player di un Bino, condiviso tra il player e il testo che lo segue. */
export interface PlayerState {
  audio: RefObject<HTMLAudioElement | null>;
  /** La durata dal mix; prima dei metadati quella del Bino. */
  durationMs: number;
  follow: Follow;
  /** Porta il player a `ms` senza cambiare Play/Pausa; il testo torna a seguire l'audio. */
  move: (ms: number, event: Exclude<FollowEvent, "scroll">) => void;
  onFollow: (event: FollowEvent) => void;
  playing: boolean;
  positionMs: number;
  setDurationMs: (ms: number) => void;
  setPlaying: (playing: boolean) => void;
  setPositionMs: (ms: number) => void;
}

/** Il player di un Bino lungo `durataMs`. */
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
  return {
    audio,
    durationMs,
    follow,
    move,
    onFollow,
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

/**
 * Il player del mix di un Bino: Play/Pausa (anche con Spazio), indietro e avanti di 10 s,
 * posizione, durata, barra di avanzamento, velocità e "Segui l'audio" quando il testo non lo segue.
 * `disabled` durante una Registrazione, che registrerebbe l'audio riascoltato.
 */
export function Player({
  disabled,
  path,
  player,
}: {
  disabled: boolean;
  path: string;
  player: PlayerState;
}) {
  const { i18n, t } = useTranslation();
  const [rate, setRate] = useState(1);
  const {
    audio,
    durationMs,
    follow,
    move,
    onFollow,
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
  const followAudio = useCallback(() => onFollow("follow"), [onFollow]);
  const rateText = new Intl.NumberFormat(i18n.language);

  return (
    <section
      aria-label={t("player.label")}
      className="flex shrink-0 items-center gap-2 border-t pt-3"
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
      <Button
        aria-label={t("player.back")}
        disabled={disabled}
        onClick={back}
        onPointerDown={keepFocus}
        size="icon"
        title={t("player.back")}
        variant="ghost"
      >
        <RotateCcw />
      </Button>
      <Button
        aria-label={playing ? t("player.pause") : t("player.play")}
        disabled={disabled}
        onClick={toggle}
        onPointerDown={keepFocus}
        size="icon"
        title={playing ? t("player.pause") : t("player.play")}
      >
        {playing ? <Pause /> : <Play />}
      </Button>
      <Button
        aria-label={t("player.forward")}
        disabled={disabled}
        onClick={forward}
        onPointerDown={keepFocus}
        size="icon"
        title={t("player.forward")}
        variant="ghost"
      >
        <RotateCw />
      </Button>
      <span className="text-muted-foreground text-xs tabular-nums">
        {elapsedText(positionMs)}
      </span>
      <input
        aria-label={t("player.position")}
        aria-valuetext={elapsedText(positionMs)}
        className="min-w-0 flex-1 accent-primary"
        disabled={disabled}
        max={durationMs}
        min={0}
        onChange={seek}
        step={100}
        type="range"
        value={Math.min(positionMs, durationMs)}
      />
      <span className="text-muted-foreground text-xs tabular-nums">
        {elapsedText(durationMs)}
      </span>
      <select
        aria-label={t("player.rate")}
        className={SELECT}
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
      {follow === "free" ? (
        <Button
          disabled={disabled}
          onClick={followAudio}
          size="sm"
          variant="outline"
        >
          {t("player.follow")}
        </Button>
      ) : null}
    </section>
  );
}
