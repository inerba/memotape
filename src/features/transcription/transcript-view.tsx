import {
  ArrowDown,
  ArrowUp,
  Check,
  ChevronDown,
  ChevronUp,
  Copy,
  Merge,
  Mic,
  Play,
  RotateCcw,
  Speaker,
  X,
} from "lucide-react";
import {
  memo,
  type ReactNode,
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import type { Ingresso, TranscriptPartial, TranscriptPhrase } from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { keepFocus, type PlayerState } from "@/features/player/player";
import { autoScroll, playingAt } from "@/features/player/sync";
import { elapsedText } from "@/features/recording/recording";
import {
  ParlanteNameInput,
  VoiceDot,
} from "@/features/transcription/parlante-name";
import {
  type Conversation,
  mergeDestination,
  type Parlante,
  type PhraseRef,
  phraseKey,
  type Turn,
  textItemsOf,
  turnBody,
  turnsOf,
  turnText,
  voiceColors,
} from "@/features/transcription/phrases";
import { type CommitTurn, type EditTurn, TurnEditor } from "./turn-editor";

const REDUCED_MOTION = "(prefers-reduced-motion: reduce)";

/** Ai turni senza Frasi in ascolto: sempre lo stesso array, così `memo` non li ridisegna. */
const NOT_PLAYING: string[] = [];

/** Lo scorrimento automatico: morbido, a meno che Windows chieda meno animazioni. */
function scrollBehavior(): ScrollBehavior {
  return window.matchMedia(REDUCED_MOTION).matches ? "auto" : "smooth";
}

/**
 * La trascrizione come documento a turni: per ogni turno il pallino e il nome della voce (un clic
 * rinomina il Parlante sul posto), il tempo che porta lì il player, ▶ che lo avvia, e le Frasi. Con
 * `onEdit` offre un editor continuo del Turno: Invio va a capo, l'uscita salva, Esc ripristina, e
 * correggere non tocca mai il player. `highlight` è la Frase di un risultato della ricerca,
 * evidenziata e portata in vista. Con `player` la Frase in ascolto si evidenzia e resta in vista
 * finché l'utente non scorre da solo; allora "Torna al punto in ascolto" la riporta. `header` sta in
 * cima, dentro lo scorrimento.
 */
export function TranscriptView({
  conversation,
  empty,
  header,
  highlight,
  onEdit,
  onMerge,
  onRename,
  onRenaming,
  parlanti,
  player,
  renaming,
}: {
  conversation: Conversation;
  /** Al posto dei turni, se non ce ne sono. */
  empty?: ReactNode;
  header?: ReactNode;
  highlight?: PhraseRef | null;
  /** Salva la correzione; `false` se non si è salvata, e il testo resta com'è scritto. */
  onEdit?: EditTurn;
  onMerge?: (turn: Turn, target: PhraseRef) => Promise<boolean>;
  onRename?: (voce: Parlante, nome: string) => void;
  /** Apre (o con `null` chiude) il campo del nome di un Parlante. */
  onRenaming?: (voce: Parlante | null) => void;
  /** I Parlanti che si possono rinominare. */
  parlanti: Parlante[];
  player?: PlayerState;
  /** Il Parlante di cui si sta scrivendo il nome. */
  renaming?: Parlante | null;
}) {
  const { t } = useTranslation();
  const section = useRef<HTMLElement>(null);
  // Una Frase in correzione: il testo non scorre da solo.
  const [editing, setEditing] = useState(false);
  const [renamingTurn, setRenamingTurn] = useState<string | null>(null);
  const renameAt = useCallback(
    (voce: Parlante | null, turnKey: string) => {
      setRenamingTurn(voce ? turnKey : null);
      onRenaming?.(voce);
    },
    [onRenaming]
  );
  // Dove sta la Frase in ascolto quando il testo non la segue.
  const [away, setAway] = useState<"up" | "down" | null>(null);
  // A riposo, prima del primo ascolto, nessuna Frase è "in ascolto".
  const listening = player && (player.playing || player.positionMs > 0);
  const playing = listening
    ? playingAt(conversation.phrases, player.positionMs).map(phraseKey)
    : [];
  const [followed] = playing;
  const scrolls = player ? autoScroll(player.follow, editing) : false;
  const free = player?.follow === "free";
  // Turni e vicini dipendono dal testo, non dalla posizione del player: con un Tape lungo
  // ricalcolarli e ridisegnarli a ogni aggiornamento bloccava l'interfaccia durante l'ascolto.
  const turns = useMemo(() => turnsOf(conversation, t), [conversation, t]);
  const colors = useMemo(() => voiceColors(turns), [turns]);
  const neighbours = useMemo(
    () =>
      turns.map((turn, index) => ({
        above: mergeDestination(turn, turns[index - 1], conversation.partials),
        below: mergeDestination(turn, turns[index + 1], conversation.partials),
        voce: parlanti.find(
          (p) => p.ingresso === turn.ingresso && p.parlante === turn.parlante
        ),
      })),
    [conversation.partials, parlanti, turns]
  );

  useEffect(() => {
    const el =
      followed && scrolls
        ? section.current?.querySelector(`[data-phrase="${followed}"]`)
        : null;
    if (el && !inView(el, section.current)) {
      el.scrollIntoView({ behavior: scrollBehavior(), block: "center" });
    }
  }, [followed, scrolls]);

  // Lo scorrimento a mano si riconosce dagli input dell'utente (rotella, tasti, barra di
  // scorrimento): `scroll` scatta anche con quello automatico.
  const onFollow = player?.onFollow;
  useEffect(() => {
    const el = section.current;
    if (!el) {
      return;
    }
    const scrolled = () => onFollow?.("scroll");
    const key = (e: globalThis.KeyboardEvent) => {
      if (SCROLL_KEYS.has(e.key) && !isEditable(e.target)) {
        scrolled();
      }
    };
    // La barra di scorrimento sta fuori dal contenuto della sezione.
    const pointer = (e: globalThis.PointerEvent) => {
      if (e.target === el && e.offsetX >= el.clientWidth) {
        scrolled();
      }
    };
    const focus = (e: globalThis.FocusEvent) =>
      setEditing(isEditable(e.target));
    const blur = () => setEditing(false);
    el.addEventListener("wheel", scrolled, { passive: true });
    el.addEventListener("keydown", key);
    el.addEventListener("pointerdown", pointer);
    el.addEventListener("focusin", focus);
    el.addEventListener("focusout", blur);
    return () => {
      el.removeEventListener("wheel", scrolled);
      el.removeEventListener("keydown", key);
      el.removeEventListener("pointerdown", pointer);
      el.removeEventListener("focusin", focus);
      el.removeEventListener("focusout", blur);
    };
  }, [onFollow]);

  useEffect(() => {
    const el = section.current;
    if (!(el && free && followed)) {
      setAway(null);
      return;
    }
    const check = () => {
      const phrase = el.querySelector(`[data-phrase="${followed}"]`);
      const box = phrase?.getBoundingClientRect();
      const view = el.getBoundingClientRect();
      if (!box || (box.bottom >= view.top && box.top <= view.bottom)) {
        setAway(null);
      } else {
        setAway(box.bottom < view.top ? "up" : "down");
      }
    };
    check();
    el.addEventListener("scroll", check, { passive: true });
    return () => el.removeEventListener("scroll", check);
  }, [followed, free]);

  const move = player?.move;
  const jump = useCallback((ms: number) => move?.(ms, "jump"), [move]);
  const backToAudio = useCallback(() => onFollow?.("follow"), [onFollow]);

  return (
    <section
      aria-label={t("transcription.text")}
      className="relative min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]"
      ref={section}
    >
      <div className="mx-auto flex w-full max-w-[46rem] flex-col px-10 pb-12">
        {header}
        {turns.length === 0 ? empty : null}
        <div className="flex flex-col gap-1">
          {turns.map((turn, index) => {
            const { above, below, voce } = neighbours[index] ?? {};
            const heard = turn.items.some((item) =>
              playing.includes(phraseKey(item))
            );
            return (
              <TurnBlock
                above={above ?? null}
                active={
                  followed !== undefined &&
                  turn.items.some(
                    (item) =>
                      !conversation.partials.includes(
                        item as TranscriptPartial
                      ) && phraseKey(item) === followed
                  )
                }
                below={below ?? null}
                color={turn.label ? colors.get(turn.label) : undefined}
                followed={heard ? followed : undefined}
                highlight={highlight}
                key={turn.key}
                list={parlanti}
                onEdit={onEdit}
                onJump={player ? jump : undefined}
                onMerge={onMerge}
                onPlay={player?.playFrom}
                onRename={onRename}
                onRenaming={onRenaming ? renameAt : undefined}
                partials={conversation.partials}
                playing={heard ? playing : NOT_PLAYING}
                renaming={
                  renamingTurn === turn.key &&
                  voce !== undefined &&
                  renaming?.ingresso === voce.ingresso &&
                  renaming.parlante === voce.parlante
                }
                turn={turn}
                voce={voce}
              />
            );
          })}
        </div>
      </div>
      {free && followed ? (
        <div className="pointer-events-none sticky bottom-4 flex justify-center">
          <button
            className="motion-safe:fade-in motion-safe:slide-in-from-bottom-2 pointer-events-auto flex items-center gap-2 rounded-full border bg-card px-3.5 py-1.5 text-sm shadow-float transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 motion-safe:animate-in"
            onClick={backToAudio}
            onPointerDown={keepFocus}
            type="button"
          >
            {away === "up" ? (
              <ArrowUp className="size-4 text-play" />
            ) : (
              <ArrowDown
                className={`size-4 text-play ${away === null ? "opacity-0" : ""}`}
              />
            )}
            {t("player.backToAudio")}
          </button>
        </div>
      ) : null}
    </section>
  );
}

/**
 * Un turno: la riga della voce e il testo. Il turno in ascolto ha il fondo salvia, l'onda al posto
 * del pallino e Riascolta, che riparte dall'inizio della Frase in ascolto.
 */
const TurnBlock = memo(function TurnBlockView({
  above,
  below,
  active,
  color,
  followed,
  highlight,
  list,
  onEdit,
  onMerge,
  onJump,
  onPlay,
  onRename,
  onRenaming,
  partials,
  playing,
  renaming,
  turn,
  voce,
}: {
  above: PhraseRef | null;
  below: PhraseRef | null;
  active: boolean;
  color: number | undefined;
  followed: string | undefined;
  highlight?: PhraseRef | null;
  list: Parlante[];
  onEdit?: EditTurn;
  onMerge?: (turn: Turn, target: PhraseRef) => Promise<boolean>;
  onJump?: (ms: number) => void;
  onPlay?: (ms: number) => void;
  onRename?: (voce: Parlante, nome: string) => void;
  onRenaming?: (voce: Parlante | null, turnKey: string) => void;
  partials: TranscriptPartial[];
  playing: string[];
  renaming: boolean;
  turn: Turn;
  voce?: Parlante;
}) {
  const { t } = useTranslation();
  const [first] = turn.items;
  const startMs = first?.inizioMs ?? 0;
  const current = turn.items.find((item) => phraseKey(item) === followed);
  const jump = useCallback(() => onJump?.(startMs), [onJump, startMs]);
  const play = useCallback(() => onPlay?.(startMs), [onPlay, startMs]);
  const replay = useCallback(
    () => current && onPlay?.(current.inizioMs),
    [current, onPlay]
  );
  const startRename = useCallback(
    () => voce && onRenaming?.(voce, turn.key),
    [onRenaming, turn.key, voce]
  );
  const cancelRename = useCallback(
    () => onRenaming?.(null, turn.key),
    [onRenaming, turn.key]
  );
  const indent = turn.label ? "pl-[1.375rem]" : "";
  const commit = useRef<CommitTurn | null>(null);
  const registerCommit = useCallback((save: CommitTurn) => {
    commit.current = save;
  }, []);
  const beforeCopy = useCallback<CommitTurn>(
    () => commit.current?.() ?? Promise.resolve(turnBody(turn)),
    [turn]
  );
  const merge = useCallback(
    async (source: Turn, target: PhraseRef) => {
      const saved = await beforeCopy();
      return saved !== null && onMerge ? onMerge(source, target) : false;
    },
    [beforeCopy, onMerge]
  );

  return (
    <article
      className={`group/turno -mx-4 rounded-xl px-4 py-3 transition-colors duration-200 ease-out ${
        active ? "bg-play-soft/55" : ""
      }`}
    >
      <div className="flex h-7 items-center gap-3">
        {turn.label ? <VoiceMark active={active} color={color} /> : null}
        {turn.label ? (
          <VoiceName
            ingresso={turn.ingresso}
            label={turn.label}
            list={list}
            name={turn.name}
            onCancel={cancelRename}
            onRename={onRename}
            onStart={voce && onRenaming ? startRename : undefined}
            renaming={renaming}
            voce={voce}
          />
        ) : null}
        {onJump ? (
          <button
            className="min-h-6 rounded-sm text-muted-foreground text-sm tabular-nums transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
            onClick={jump}
            onPointerDown={keepFocus}
            title={t("player.jump")}
            type="button"
          >
            {elapsedText(startMs)}
          </button>
        ) : (
          <span className="text-muted-foreground text-sm tabular-nums">
            {elapsedText(startMs)}
          </span>
        )}
        {onPlay ? (
          <button
            aria-label={t("player.playTurn")}
            className="flex size-6 items-center justify-center rounded-md text-foreground/70 transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 [&_svg]:size-3 [&_svg]:fill-current"
            onClick={play}
            onPointerDown={keepFocus}
            title={t("player.playTurn")}
            type="button"
          >
            <Play />
          </button>
        ) : null}
        <span className="flex-1" />
        {active && current && onPlay ? (
          <button
            className="flex items-center gap-1.5 rounded-md px-2 py-1 text-foreground/80 text-sm transition-colors hover:bg-background/70 hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 [&_svg]:size-3.5"
            onClick={replay}
            onPointerDown={keepFocus}
            type="button"
          >
            <RotateCcw />
            {t("player.replay")}
          </button>
        ) : null}
        {onMerge &&
        !turn.items.some((item) =>
          partials.includes(item as TranscriptPartial)
        ) ? (
          <MergeTurn above={above} below={below} onMerge={merge} turn={turn} />
        ) : null}
        <CopyTurn beforeCopy={onEdit ? beforeCopy : undefined} turn={turn} />
      </div>
      <div
        className={`mt-1 flex flex-col gap-2 text-[1.0625rem] leading-[1.7] ${indent}`}
      >
        {onEdit &&
        !turn.items.some((item) =>
          partials.includes(item as TranscriptPartial)
        ) ? (
          <TurnEditor
            highlight={highlight}
            onCommit={registerCommit}
            onEdit={onEdit}
            playing={playing}
            turn={turn}
          />
        ) : (
          textItemsOf(turn).map((item, i) => {
            const partial = partials.includes(item as TranscriptPartial);
            const key = phraseKey(item);
            return (
              <p key={`${key}${partial ? ":parziale" : ""}`}>
                <PhraseText
                  highlighted={
                    highlight?.ingresso === item.ingresso &&
                    highlight.phraseId === item.phraseId
                  }
                  item={item}
                  onJump={partial || i === 0 ? undefined : onJump}
                  partial={partial}
                  playing={
                    !(
                      partial ||
                      ("testoCorretto" in item && item.testoCorretto)
                    ) && playing.includes(key)
                  }
                />
              </p>
            );
          })
        )}
      </div>
    </article>
  );
});

/** Il menu di questo Turno ha un'ancora propria, anche quando la stessa voce compare più volte. */
function MergeTurn({
  above,
  below,
  onMerge,
  turn,
}: {
  above: PhraseRef | null;
  below: PhraseRef | null;
  onMerge: (turn: Turn, target: PhraseRef) => Promise<boolean>;
  turn: Turn;
}) {
  const { t } = useTranslation();
  const id = `merge-${useId().replace(/[^a-zA-Z0-9_-]/g, "")}`;
  const [pending, setPending] = useState(false);
  const saving = useRef(false);
  const merge = useCallback(
    async (target: PhraseRef | null) => {
      if (!target || saving.current) {
        return;
      }
      saving.current = true;
      setPending(true);
      try {
        await onMerge(turn, target);
      } finally {
        saving.current = false;
        setPending(false);
      }
    },
    [onMerge, turn]
  );
  const mergeAbove = useCallback(() => merge(above), [above, merge]);
  const mergeBelow = useCallback(() => merge(below), [below, merge]);
  const itemClass =
    "flex h-8 w-full items-center gap-2 whitespace-nowrap rounded-md px-2 text-left hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-40 [&_svg]:size-4";
  return (
    <PopoverMenu
      className="flex size-6 items-center justify-center rounded-md text-foreground/70 opacity-0 transition-[opacity,color,background-color] hover:bg-accent hover:text-foreground focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 group-focus-within/turno:opacity-100 group-hover/turno:opacity-100 [&_svg]:size-3.5"
      disabled={pending}
      icon={<Merge />}
      id={id}
      label={t("transcription.merge")}
      variant="ghost"
    >
      <button
        className={itemClass}
        disabled={!above || pending}
        onClick={mergeAbove}
        popoverTarget={`menu-${id}`}
        popoverTargetAction="hide"
        type="button"
      >
        <ChevronUp />
        {t("transcription.mergeAbove")}
      </button>
      <button
        className={itemClass}
        disabled={!below || pending}
        onClick={mergeBelow}
        popoverTarget={`menu-${id}`}
        popoverTargetAction="hide"
        type="button"
      >
        <ChevronDown />
        {t("transcription.mergeBelow")}
      </button>
      <hr className="my-1 border-border" />
      <button
        className={`${itemClass} text-muted-foreground`}
        popoverTarget={`menu-${id}`}
        popoverTargetAction="hide"
        type="button"
      >
        <X />
        {t("transcription.mergeCancel")}
      </button>
    </PopoverMenu>
  );
}

/** Per quanto Copia turno mostra la spunta. */
const COPIED_MS = 1500;

/** Copia turno: il testo semplice del turno negli appunti; si vede col mouse o il focus sul turno. */
function CopyTurn({
  turn,
  beforeCopy,
}: {
  turn: Turn;
  beforeCopy?: CommitTurn;
}) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = setTimeout(() => setCopied(false), COPIED_MS);
    return () => clearTimeout(timer);
  }, [copied]);
  const copy = useCallback(async () => {
    const savedText = await beforeCopy?.();
    if (savedText === null) {
      return;
    }
    await navigator.clipboard.writeText(savedText ?? turnText(turn));
    setCopied(true);
  }, [turn, beforeCopy]);
  const label = copied
    ? t("transcription.copied")
    : t("transcription.copyTurn");
  return (
    <button
      aria-label={label}
      className={`flex size-6 items-center justify-center rounded-md text-foreground/70 transition-[opacity,color,background-color] hover:bg-accent hover:text-foreground focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 group-focus-within/turno:opacity-100 group-hover/turno:opacity-100 [&_svg]:size-3.5 ${
        copied ? "opacity-100" : "opacity-0"
      }`}
      onClick={copy}
      onPointerDown={keepFocus}
      title={label}
      type="button"
    >
      {copied ? <Check className="text-play" /> : <Copy />}
    </button>
  );
}

/** Il nome della voce: un clic apre il campo del nome, se è un Parlante. */
function VoiceName({
  ingresso,
  label,
  list,
  onCancel,
  onRename,
  onStart,
  name,
  renaming,
  voce,
}: {
  ingresso: Ingresso;
  label: string;
  list: Parlante[];
  onCancel: () => void;
  onRename?: (voce: Parlante, nome: string) => void;
  onStart?: () => void;
  name: string | null;
  renaming: boolean;
  voce?: Parlante;
}) {
  const { t } = useTranslation();
  if (renaming && voce && onRename) {
    return (
      <ParlanteNameInput
        list={list}
        onCancel={onCancel}
        onRename={onRename}
        voce={voce}
      />
    );
  }
  if (!onStart) {
    return (
      <span
        className="inline-flex items-center gap-2 font-semibold text-sm"
        title={label}
      >
        <span className="sr-only">{label}</span>
        <VoiceLabel ingresso={ingresso} name={name} />
      </span>
    );
  }
  return (
    <button
      aria-label={label}
      className="inline-flex items-center gap-2 rounded-sm font-semibold text-sm decoration-muted-foreground/50 underline-offset-4 hover:underline focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
      onClick={onStart}
      title={t("transcription.rename")}
      type="button"
    >
      <VoiceLabel ingresso={ingresso} name={name} />
    </button>
  );
}

function VoiceLabel({
  ingresso,
  name,
}: {
  ingresso: Ingresso;
  name: string | null;
}) {
  return (
    <>
      {ingresso === "microfono" ? (
        <Mic aria-hidden className="size-4 shrink-0" />
      ) : null}
      {ingresso === "sistema" ? (
        <Speaker aria-hidden className="size-4 shrink-0" />
      ) : null}
      {name ? <span aria-hidden>{name}</span> : null}
    </>
  );
}

/** Il segno della voce: il pallino del suo colore, o nel turno in ascolto le barre che si muovono. */
function VoiceMark({
  active,
  color,
}: {
  active: boolean;
  color: number | undefined;
}) {
  return active ? <NowPlaying /> : <VoiceDot color={color} />;
}

/** Al posto del pallino, nel turno in ascolto: tre barre che si muovono. */
function NowPlaying() {
  return (
    <span
      aria-hidden
      className="flex h-2.5 w-2.5 shrink-0 items-end justify-between"
    >
      {[0, 1, 2].map((bar) => (
        <span
          className="w-[2px] origin-bottom rounded-full bg-play motion-safe:animate-[equalizer_0.9s_ease-in-out_infinite]"
          key={bar}
          style={{ animationDelay: `${bar * -0.3}s`, height: "100%" }}
        />
      ))}
    </span>
  );
}

const SCROLL_KEYS = new Set([
  "ArrowDown",
  "ArrowUp",
  "End",
  "Home",
  "PageDown",
  "PageUp",
]);

/** Se `target` è una Frase in correzione o un altro campo, dove i tasti non scorrono il testo. */
function isEditable(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || target.matches("input, textarea, select"))
  );
}

/** Se `el` si vede per intero dentro `container`. */
function inView(el: Element, container: HTMLElement | null): boolean {
  if (!container) {
    return true;
  }
  const box = el.getBoundingClientRect();
  const view = container.getBoundingClientRect();
  return box.top >= view.top && box.bottom <= view.bottom;
}

/**
 * Una Frase in lettura o un Parziale, seguiti da uno spazio.
 * Con `onJump`, passando il mouse (o con il focus) compare sopra la Frase il pulsante del suo tempo,
 * che porta lì il player senza spostare il testo.
 */
function PhraseText({
  highlighted,
  item,
  onJump,
  partial,
  playing,
}: {
  highlighted: boolean;
  item: TranscriptPhrase | TranscriptPartial;
  onJump?: (ms: number) => void;
  /** Il Parziale della Frase in corso, ancora provvisorio. */
  partial: boolean;
  playing: boolean;
}) {
  const { t } = useTranslation();
  const span = useRef<HTMLSpanElement>(null);
  useEffect(() => {
    if (highlighted) {
      span.current?.scrollIntoView({ block: "center" });
    }
  }, [highlighted]);
  const jump = useCallback(
    () => onJump?.(item.inizioMs),
    [item.inizioMs, onJump]
  );
  let className =
    "-mx-0.5 rounded-[0.3rem] box-decoration-clone px-0.5 transition-colors duration-200 ease-out";
  if (playing) {
    className += " bg-play/18";
  }
  if (highlighted) {
    className += " ring-1 ring-play/60";
  }
  if (partial) {
    className += " text-muted-foreground italic";
  }
  const time = onJump ? (
    <button
      className="pointer-events-none absolute bottom-full left-0 z-10 select-none rounded-md border bg-card px-1.5 py-0.5 text-muted-foreground text-xs tabular-nums leading-none opacity-0 shadow-float transition-opacity duration-150 hover:text-foreground focus-visible:pointer-events-auto focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 group-hover/frase:pointer-events-auto group-hover/frase:opacity-100"
      onClick={jump}
      onPointerDown={keepFocus}
      title={t("player.jump")}
      type="button"
    >
      {elapsedText(item.inizioMs)}
    </button>
  ) : null;
  return (
    <span className="group/frase relative" data-phrase={phraseKey(item)}>
      {time}
      <span className={`${className} whitespace-pre-wrap`} ref={span}>
        {item.text}
      </span>{" "}
    </span>
  );
}
