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
  Fragment,
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
import type {
  Ingresso,
  TranscriptPartial,
  TranscriptPhrase,
  VistaTrascrizione,
} from "@/bindings";
import { PopoverMenu } from "@/components/popover-menu";
import { keepFocus, type PlayerState } from "@/features/player/player";
import { autoScroll, playingAt } from "@/features/player/sync";
import { elapsedText } from "@/features/recording/recording";
import { useSettings } from "@/features/settings/settings-context";
import { ariaTasti, conTasti } from "@/features/shortcuts/shortcuts-provider";
import {
  ParlanteNameInput,
  VOICE_DOTS,
} from "@/features/transcription/parlante-name";
import {
  type Conversation,
  mergeDestination,
  type Parlante,
  type PhraseRef,
  pauseDuration,
  pausesOf,
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

/** La colonna dei nomi del Copione; con gli Ingressi separati c'è anche l'icona. */
const NAME_COLUMN = "88px";
const NAME_COLUMN_SEPARATE = "140px";

/** Il nodo del Nastro e la sottolineatura dell'Intervista, per colore della voce (da `voiceColors`). */
const VOICE_RINGS = [
  "ring-voice-1",
  "ring-voice-2",
  "ring-voice-3",
  "ring-voice-4",
  "ring-voice-5",
  "ring-voice-6",
];
const VOICE_UNDERLINES = [
  "decoration-voice-1",
  "decoration-voice-2",
  "decoration-voice-3",
  "decoration-voice-4",
  "decoration-voice-5",
  "decoration-voice-6",
];

/** Un pulsante a icona della barretta del turno. */
const ICON_BUTTON =
  "flex size-6 items-center justify-center rounded-md text-foreground/70 transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40";

/** Il nome che si rinomina con un clic si sottolinea al passaggio. */
const HOVER_UNDERLINE =
  "decoration-muted-foreground/50 underline-offset-4 enabled:hover:underline";

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
  const pauses = useMemo(() => pausesOf(turns), [turns]);
  const vista = useSettings().settings.vistaTrascrizione ?? "copione";
  let nameWidth: string | null = null;
  if (turns.some((turn) => turn.label !== null)) {
    nameWidth = turns.some((turn) => turn.ingresso !== "mix")
      ? NAME_COLUMN_SEPARATE
      : NAME_COLUMN;
  }
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
        <div className={`flex flex-col ${vista === "nastro" ? "" : "gap-0.5"}`}>
          {turns.map((turn, index) => {
            const { above, below, voce } = neighbours[index] ?? {};
            const heard = turn.items.some((item) =>
              playing.includes(phraseKey(item))
            );
            const pause = vista === "intervista" ? null : pauses[index];
            return (
              <Fragment key={turn.key}>
                {pause ? (
                  <PauseMark ms={pause} nameWidth={nameWidth} vista={vista} />
                ) : null}
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
                  list={parlanti}
                  nameWidth={nameWidth}
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
                  vista={vista}
                  voce={voce}
                />
              </Fragment>
            );
          })}
        </div>
      </div>
      {free && followed ? (
        <div className="pointer-events-none sticky bottom-4 flex justify-center">
          <button
            aria-keyshortcuts={ariaTasti("tornaAlPunto")}
            className="motion-safe:fade-in motion-safe:slide-in-from-bottom-2 pointer-events-auto flex items-center gap-2 rounded-full border bg-card px-3.5 py-1.5 text-sm shadow-float transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 motion-safe:animate-in"
            onClick={backToAudio}
            onPointerDown={keepFocus}
            title={conTasti(t, t("player.backToAudio"), "tornaAlPunto")}
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

/** Il separatore di una pausa lunga tra due turni, in Copione e Nastro. */
function PauseMark({
  ms,
  nameWidth,
  vista,
}: {
  ms: number;
  nameWidth: string | null;
  vista: VistaTrascrizione;
}) {
  const { t } = useTranslation();
  const text = t("transcription.pause", { durata: pauseDuration(ms) });
  if (vista === "nastro") {
    return (
      <div className="grid grid-cols-[3rem_1.25rem_minmax(0,1fr)] gap-3">
        <span />
        <span aria-hidden className="relative">
          <span className="absolute inset-y-0 left-1/2 w-[1.5px] -translate-x-1/2 bg-[repeating-linear-gradient(to_bottom,color-mix(in_oklch,var(--foreground)_25%,transparent)_0_3px,transparent_3px_7px)]" />
        </span>
        <span className="pt-0.5 pb-3 text-muted-foreground text-xs">
          {text}
        </span>
      </div>
    );
  }
  return (
    <div
      className="flex items-center gap-3 py-2 text-muted-foreground text-xs after:flex-1 after:border-input after:border-t after:border-dashed after:content-['']"
      style={{
        paddingLeft: nameWidth ? `calc(${nameWidth} + 1rem)` : undefined,
      }}
    >
      {text}
    </div>
  );
}

/** Il tempo d'inizio del turno: con `onJump` porta lì il player. */
function TurnTime({
  className,
  ms,
  onJump,
}: {
  className: string;
  ms: number;
  onJump?: (ms: number) => void;
}) {
  const { t } = useTranslation();
  const jump = useCallback(() => onJump?.(ms), [ms, onJump]);
  const base = `h-fit text-[0.78125rem] text-muted-foreground tabular-nums ${className}`;
  return onJump ? (
    <button
      className={`${base} rounded-sm transition-[color,opacity] hover:text-foreground focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40`}
      onClick={jump}
      onPointerDown={keepFocus}
      title={t("player.jump")}
      type="button"
    >
      {elapsedText(ms)}
    </button>
  ) : (
    <span className={base}>{elapsedText(ms)}</span>
  );
}

/** L'icona dell'Ingresso separato davanti al nome; il mix non ne ha. */
function IngressoIcon({
  className,
  ingresso,
}: {
  className: string;
  ingresso: Ingresso;
}) {
  if (ingresso === "microfono") {
    return <Mic aria-hidden className={`shrink-0 ${className}`} />;
  }
  return ingresso === "sistema" ? (
    <Speaker aria-hidden className={`shrink-0 ${className}`} />
  ) : null;
}

/**
 * Un turno, nella resa di `vista`: Copione (nome in colonna, tempo a destra), Intervista (il nome
 * apre il paragrafo, tempo nel margine) o Nastro (un nodo per turno sulla linea, tempo a sinistra).
 * Il turno in ascolto ha il fondo salvia (nel Nastro anche le barre al posto del nodo); ▶, Riascolta,
 * Unisci e Copia turno stanno in una barretta che compare in hover o con il focus.
 */
const TurnBlock = memo(function TurnBlockView({
  above,
  below,
  active,
  color,
  followed,
  highlight,
  list,
  nameWidth,
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
  vista,
  voce,
}: {
  above: PhraseRef | null;
  below: PhraseRef | null;
  active: boolean;
  color: number | undefined;
  followed: string | undefined;
  highlight?: PhraseRef | null;
  list: Parlante[];
  /** La colonna dei nomi del Copione; `null` se nessun turno ha un'etichetta. */
  nameWidth: string | null;
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
  vista: VistaTrascrizione;
  voce?: Parlante;
}) {
  const uncertain = Boolean(turn.items[0]?.parlanteNonDeterminato);
  const startRename = useCallback(
    () => voce && onRenaming?.(voce, turn.key),
    [onRenaming, turn.key, voce]
  );
  const cancelRename = useCallback(
    () => onRenaming?.(null, turn.key),
    [onRenaming, turn.key]
  );
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
  const live = turn.items.some((item) =>
    partials.includes(item as TranscriptPartial)
  );

  const row: RowProps = {
    actions: (
      <TurnActions
        above={above}
        active={active}
        beforeCopy={onEdit ? beforeCopy : undefined}
        below={below}
        current={turn.items.find((item) => phraseKey(item) === followed)}
        onMerge={onMerge && !live ? merge : undefined}
        onPlay={onPlay}
        turn={turn}
      />
    ),
    active,
    color: uncertain ? undefined : color,
    onJump,
    text: (
      <TurnText
        highlight={highlight}
        indented={vista === "intervista"}
        onCommit={registerCommit}
        onEdit={live ? undefined : onEdit}
        onJump={onJump}
        partials={partials}
        playing={playing}
        turn={turn}
        uncertain={uncertain}
      />
    ),
    turn,
    uncertain,
    voice: turn.label
      ? {
          label: turn.label,
          list,
          onCancel: cancelRename,
          onRename,
          onStart: voce && onRenaming ? startRename : undefined,
          renaming,
          voce,
        }
      : null,
  };
  if (vista === "copione") {
    return <CopioneRow {...row} nameWidth={nameWidth} />;
  }
  return vista === "intervista" ? (
    <IntervistaRow {...row} />
  ) : (
    <NastroRow {...row} />
  );
});

type VoiceProps = Omit<
  Parameters<typeof VoiceName>[0],
  "children" | "className" | "inputClassName"
>;

/** Quello che le tre rese di un turno hanno in comune. */
interface RowProps {
  /** La barretta di ▶, Riascolta, Unisci e Copia turno. */
  actions: ReactNode;
  active: boolean;
  /** Il colore della voce; nessuno per il Parlante non determinato o senza etichetta. */
  color: number | undefined;
  onJump?: (ms: number) => void;
  text: ReactNode;
  turn: Turn;
  /** Il Parlante non determinato: «?» o il nodo vuoto, testo tenue. */
  uncertain: boolean;
  /** Il nome, se il turno ha un'etichetta. */
  voice: VoiceProps | null;
}

/** Copione: il nome in maiuscoletto nella sua colonna, il testo accanto, il tempo a destra. */
function CopioneRow({
  actions,
  active,
  color,
  nameWidth,
  onJump,
  text,
  turn,
  uncertain,
  voice,
}: RowProps & { nameWidth: string | null }) {
  const name = uncertain ? "?" : turn.name;
  return (
    <article
      className={`group/turno relative -mx-3 grid items-start gap-4 rounded-[10px] px-3 py-[5px] transition-colors duration-200 ease-out ${
        active ? "bg-play-soft/55" : ""
      }`}
      style={{
        gridTemplateColumns: nameWidth
          ? `${nameWidth} minmax(0, 1fr) 3rem`
          : "minmax(0, 1fr) 3rem",
      }}
    >
      {nameWidth && voice ? (
        <VoiceName
          {...voice}
          className={`flex min-w-0 items-center gap-[7px] pt-[0.4rem] text-left font-semibold text-xs uppercase leading-normal tracking-[0.07em] ${HOVER_UNDERLINE} ${
            uncertain ? "text-muted-foreground" : ""
          }`}
          inputClassName="relative z-10"
        >
          <span
            aria-hidden
            className={`size-[7px] shrink-0 rounded-full ${
              color === undefined
                ? "ring-[1.5px] ring-muted-foreground ring-inset"
                : VOICE_DOTS[color]
            }`}
          />
          <IngressoIcon className="size-3.5" ingresso={turn.ingresso} />
          {name ? <span className="truncate">{name}</span> : null}
        </VoiceName>
      ) : null}
      {nameWidth && !voice ? <span /> : null}
      {text}
      <TurnTime
        className={`justify-self-end pt-[0.3rem] group-hover/turno:opacity-100 ${
          active ? "" : "opacity-55"
        }`}
        ms={turn.items[0]?.inizioMs ?? 0}
        onJump={onJump}
      />
      {actions}
    </article>
  );
}

/** Intervista: il nome in grassetto apre il paragrafo, il tempo sta nel margine sinistro. */
function IntervistaRow({
  actions,
  active,
  color,
  onJump,
  text,
  turn,
  uncertain,
  voice,
}: RowProps) {
  const name = uncertain ? "?" : turn.name;
  // Il nome sta sopra il testo, che rientra della sua larghezza nella prima riga: così anche
  // l'editor del Turno, un campo nativo, comincia dopo il nome.
  const indentText = useCallback((nameBox: HTMLSpanElement | null) => {
    const box = nameBox?.parentElement;
    if (!(nameBox && box)) {
      return;
    }
    const indent = () =>
      box.style.setProperty("--rientro", `${nameBox.offsetWidth}px`);
    indent();
    const observer = new ResizeObserver(indent);
    observer.observe(nameBox);
    return () => observer.disconnect();
  }, []);
  return (
    <article
      className={`group/turno relative -mx-3 rounded-[10px] py-1.5 pr-3 pl-[4.25rem] transition-colors duration-200 ease-out ${
        active ? "bg-play-soft/55" : ""
      }`}
    >
      <TurnTime
        className="absolute top-[0.6rem] left-3"
        ms={turn.items[0]?.inizioMs ?? 0}
        onJump={onJump}
      />
      <div className="relative">
        {voice ? (
          <span
            className="absolute top-0 left-0 pr-[0.45em] text-[1.0625rem] leading-[1.7]"
            ref={indentText}
          >
            <VoiceName
              {...voice}
              className={`inline-flex items-center gap-1.5 ${
                uncertain
                  ? "font-medium text-muted-foreground"
                  : "font-semibold"
              }`}
            >
              <IngressoIcon className="size-4" ingresso={turn.ingresso} />
              {name ? (
                <span
                  className={
                    color === undefined
                      ? undefined
                      : `underline decoration-[3px] underline-offset-[5px] [text-decoration-skip-ink:none] ${VOICE_UNDERLINES[color]}`
                  }
                >
                  {name}
                </span>
              ) : null}
            </VoiceName>
          </span>
        ) : null}
        {text}
      </div>
      {actions}
    </article>
  );
}

/** Nastro: una linea con un nodo per turno nel colore della voce, il tempo a sinistra. */
function NastroRow({
  actions,
  active,
  color,
  onJump,
  text,
  turn,
  uncertain,
  voice,
}: RowProps) {
  let node = (
    <span
      className={`relative mt-2 size-2.5 rounded-full shadow-[0_0_0_4px_var(--background)] ring-[1.5px] ${
        color === undefined
          ? "bg-background ring-muted-foreground"
          : `${VOICE_DOTS[color]} ${VOICE_RINGS[color]}`
      }`}
    />
  );
  if (active) {
    node = (
      <span className="relative mt-[7px] h-fit rounded-sm bg-background p-0.5">
        <NowPlaying />
      </span>
    );
  }
  return (
    <article className="group/riga group/turno relative grid grid-cols-[3rem_1.25rem_minmax(0,1fr)] gap-3">
      <TurnTime
        className="justify-self-end pt-1"
        ms={turn.items[0]?.inizioMs ?? 0}
        onJump={onJump}
      />
      <span aria-hidden className="relative flex justify-center">
        <span className="absolute inset-y-0 left-1/2 w-[1.5px] -translate-x-1/2 bg-foreground/16 group-first/riga:top-3 group-last/riga:bottom-[calc(100%-0.75rem)]" />
        {node}
      </span>
      <div
        className={`min-w-0 transition-colors duration-200 ease-out ${
          active
            ? "-mx-3 mb-2 rounded-[10px] bg-play-soft/55 px-3 pt-0.5 pb-2.5"
            : "pt-0.5 pb-3.5"
        }`}
      >
        {voice && !uncertain ? (
          <VoiceName
            {...voice}
            className={`mt-[3px] flex w-fit max-w-full items-center gap-1.5 font-medium text-[0.8125rem] leading-[1.4] ${HOVER_UNDERLINE}`}
          >
            <IngressoIcon className="size-3.5" ingresso={turn.ingresso} />
            {turn.name ? <span className="truncate">{turn.name}</span> : null}
          </VoiceName>
        ) : null}
        {voice && uncertain ? (
          <span className="sr-only">{voice.label}</span>
        ) : null}
        {text}
      </div>
      {actions}
    </article>
  );
}

/**
 * La barretta del turno, in alto a destra, che compare in hover o con il focus: ▶ (ascolta il
 * turno), Riascolta sul turno in ascolto, Unisci e Copia turno.
 */
function TurnActions({
  above,
  active,
  below,
  beforeCopy,
  current,
  onMerge,
  onPlay,
  turn,
}: {
  above: PhraseRef | null;
  active: boolean;
  below: PhraseRef | null;
  beforeCopy?: CommitTurn;
  /** La Frase in ascolto del turno, da cui riparte Riascolta. */
  current?: { inizioMs: number };
  onMerge?: (turn: Turn, target: PhraseRef) => Promise<boolean>;
  onPlay?: (ms: number) => void;
  turn: Turn;
}) {
  const { t } = useTranslation();
  const startMs = turn.items[0]?.inizioMs ?? 0;
  const play = useCallback(() => onPlay?.(startMs), [onPlay, startMs]);
  const replay = useCallback(
    () => current && onPlay?.(current.inizioMs),
    [current, onPlay]
  );
  return (
    <div className="pointer-events-none absolute -top-3.5 right-1 z-10 flex items-center gap-0.5 rounded-lg border bg-card p-0.5 opacity-0 shadow-float transition-opacity duration-150 group-focus-within/turno:pointer-events-auto group-focus-within/turno:opacity-100 group-hover/turno:pointer-events-auto group-hover/turno:opacity-100">
      {onPlay ? (
        <button
          aria-label={t("player.playTurn")}
          className={`${ICON_BUTTON} [&_svg]:size-3 [&_svg]:fill-current`}
          onClick={play}
          onPointerDown={keepFocus}
          title={t("player.playTurn")}
          type="button"
        >
          <Play />
        </button>
      ) : null}
      {active && current && onPlay ? (
        <button
          className="flex h-6 items-center gap-1.5 rounded-md px-2 text-foreground/80 text-sm transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 [&_svg]:size-3.5"
          onClick={replay}
          onPointerDown={keepFocus}
          type="button"
        >
          <RotateCcw />
          {t("player.replay")}
        </button>
      ) : null}
      {onMerge ? (
        <MergeTurn above={above} below={below} onMerge={onMerge} turn={turn} />
      ) : null}
      <CopyTurn beforeCopy={beforeCopy} turn={turn} />
    </div>
  );
}

/**
 * Il testo del turno: con `onEdit` l'editor continuo del Turno, altrimenti le Frasi e i Parziali
 * in lettura, un paragrafo per Frase. `indented`: la prima riga rientra del nome (Intervista).
 */
function TurnText({
  highlight,
  indented,
  onCommit,
  onEdit,
  onJump,
  partials,
  playing,
  turn,
  uncertain,
}: {
  highlight?: PhraseRef | null;
  indented: boolean;
  onCommit: (commit: CommitTurn) => void;
  onEdit?: EditTurn;
  onJump?: (ms: number) => void;
  partials: TranscriptPartial[];
  playing: string[];
  turn: Turn;
  uncertain: boolean;
}) {
  return (
    <div
      className={`flex min-w-0 flex-col gap-2 text-[1.0625rem] leading-[1.7] ${
        uncertain ? "text-muted-foreground" : ""
      } ${indented ? "[&>:first-child]:[text-indent:var(--rientro,0px)]" : ""}`}
    >
      {onEdit ? (
        <TurnEditor
          highlight={highlight}
          onCommit={onCommit}
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
  );
}

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
      className={`${ICON_BUTTON} [&_svg]:size-3.5`}
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
      className={`${ICON_BUTTON} [&_svg]:size-3.5`}
      onClick={copy}
      onPointerDown={keepFocus}
      title={label}
      type="button"
    >
      {copied ? <Check className="text-play" /> : <Copy />}
    </button>
  );
}

/**
 * Il nome della voce, come lo disegna la vista (`children`): un clic apre il campo del nome, se è un
 * Parlante. Il lettore di schermo legge sempre l'etichetta intera (`Microfono · Parlante non
 * determinato`), anche dove si vede solo «?»; il tooltip la mostra per i nomi tagliati.
 */
function VoiceName({
  children,
  className,
  inputClassName,
  label,
  list,
  onCancel,
  onRename,
  onStart,
  renaming,
  voce,
}: {
  children: ReactNode;
  className: string;
  inputClassName?: string;
  label: string;
  list: Parlante[];
  onCancel: () => void;
  onRename?: (voce: Parlante, nome: string) => void;
  onStart?: () => void;
  renaming: boolean;
  voce?: Parlante;
}) {
  const { t } = useTranslation();
  if (renaming && voce && onRename) {
    return (
      <ParlanteNameInput
        className={inputClassName}
        list={list}
        onCancel={onCancel}
        onRename={onRename}
        voce={voce}
      />
    );
  }
  if (!onStart) {
    return (
      <span className={className} title={label}>
        <span className="sr-only">{label}</span>
        <span aria-hidden className="contents">
          {children}
        </span>
      </span>
    );
  }
  return (
    <button
      aria-label={label}
      className={`rounded-sm focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 ${className}`}
      onClick={onStart}
      title={`${label}\n${t("transcription.rename")}`}
      type="button"
    >
      {children}
    </button>
  );
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
