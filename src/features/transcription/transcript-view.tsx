import {
  type FocusEvent,
  type KeyboardEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import type { TranscriptPartial, TranscriptPhrase } from "@/bindings";
import { keepFocus, type PlayerState } from "@/features/player/player";
import { autoScroll, playingAt } from "@/features/player/sync";
import { elapsedText } from "@/features/recording/recording";
import {
  type Conversation,
  type Parlante,
  type PhraseRef,
  phraseKey,
  turnsOf,
} from "@/features/transcription/phrases";

/**
 * La trascrizione a turni: l'etichetta della voce (un clic rinomina il Parlante) e le sue Frasi una
 * dopo l'altra. Con `onEdit` un clic su una Frase la rende modificabile: Invio o l'uscita salvano,
 * Esc ripristina. `highlight` è la Frase di un risultato della ricerca, evidenziata e portata in
 * vista. Con `player` le Frasi in riproduzione si evidenziano e restano in vista finché l'utente non
 * scorre da solo, e ogni Frase ha il pulsante del suo tempo.
 */
export function TranscriptView({
  conversation,
  highlight,
  onEdit,
  onRename,
  parlanti,
  player,
}: {
  conversation: Conversation;
  highlight?: PhraseRef | null;
  /** Salva la correzione; `false` se non si è salvata, e il testo resta com'è scritto. */
  onEdit?: (phrase: PhraseRef, text: string) => Promise<boolean>;
  onRename?: (voce: Parlante) => void;
  /** I Parlanti che si possono rinominare. */
  parlanti: Parlante[];
  player?: PlayerState;
}) {
  const { t } = useTranslation();
  const section = useRef<HTMLElement>(null);
  // Una Frase in correzione: il testo non scorre da solo.
  const [editing, setEditing] = useState(false);
  const playing = player
    ? playingAt(conversation.phrases, player.positionMs).map(phraseKey)
    : [];
  const [followed] = playing;
  const scrolls = player ? autoScroll(player.follow, editing) : false;

  useEffect(() => {
    const el =
      followed && scrolls
        ? section.current?.querySelector(`[data-phrase="${followed}"]`)
        : null;
    if (el && !inView(el, section.current)) {
      el.scrollIntoView({ block: "center" });
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
  const move = player?.move;
  const jump = useCallback((ms: number) => move?.(ms, "jump"), [move]);

  return (
    <section
      aria-label={t("transcription.text")}
      className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto pr-2 leading-relaxed"
      ref={section}
    >
      {turnsOf(conversation, t).map((turn) => {
        const voce = parlanti.find(
          (p) => p.ingresso === turn.ingresso && p.parlante === turn.parlante
        );
        return (
          <div className="flex flex-col gap-1" key={turn.key}>
            {turn.label ? (
              <TurnLabel label={turn.label} onRename={onRename} voce={voce} />
            ) : null}
            <p>
              {turn.items.map((item, i) => {
                const partial = conversation.partials.includes(
                  item as TranscriptPartial
                );
                const key = phraseKey(item);
                return (
                  <PhraseText
                    highlighted={
                      highlight?.ingresso === item.ingresso &&
                      highlight.phraseId === item.phraseId
                    }
                    item={item}
                    key={`${key}${partial ? ":parziale" : ""}`}
                    onEdit={partial ? undefined : onEdit}
                    onJump={partial || !player ? undefined : jump}
                    partial={partial}
                    playing={!partial && playing.includes(key)}
                    showTime={i === 0}
                  />
                );
              })}
            </p>
          </div>
        );
      })}
    </section>
  );
}

function TurnLabel({
  label,
  onRename,
  voce,
}: {
  label: string;
  onRename?: (voce: Parlante) => void;
  voce?: Parlante;
}) {
  const { t } = useTranslation();
  const rename = useCallback(() => voce && onRename?.(voce), [onRename, voce]);
  if (!(voce && onRename)) {
    return (
      <span className="font-medium text-muted-foreground text-sm">{label}</span>
    );
  }
  return (
    <button
      className="self-start rounded-sm font-medium text-muted-foreground text-sm hover:text-foreground hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={rename}
      title={t("transcription.rename")}
      type="button"
    >
      {label}
    </button>
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

/** Il testo scritto nella Frase: senza a capo né spazi non separabili. */
function typedText(el: HTMLElement): string {
  return (el.textContent ?? "").replace(/\s/g, " ");
}

/**
 * Una Frase, seguita da uno spazio. Modificabile solo come testo semplice; il `key` del suo span
 * cambia quando il testo arriva da fuori o con Esc, così React non scrive mai nel testo modificato.
 * Con `onJump` la precede il pulsante del suo tempo, che porta lì il player: si vede passando il
 * mouse, con il focus, sempre nel punto del player (`playing`) e con `showTime` (inizio del turno).
 */
function PhraseText({
  highlighted,
  item,
  onEdit,
  onJump,
  partial,
  playing,
  showTime,
}: {
  highlighted: boolean;
  item: TranscriptPhrase | TranscriptPartial;
  onEdit?: (phrase: PhraseRef, text: string) => Promise<boolean>;
  onJump?: (ms: number) => void;
  /** Il Parziale della Frase in corso, ancora provvisorio. */
  partial: boolean;
  playing: boolean;
  showTime: boolean;
}) {
  const { t } = useTranslation();
  const [resets, setResets] = useState(0);
  const span = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (highlighted) {
      span.current?.scrollIntoView({ block: "center" });
    }
  }, [highlighted]);

  const save = useCallback(
    (e: FocusEvent<HTMLSpanElement>) => {
      const text = typedText(e.currentTarget);
      if (onEdit && text !== item.text) {
        onEdit(item, text);
      }
    },
    [item, onEdit]
  );
  const keyDown = useCallback(
    (e: KeyboardEvent<HTMLSpanElement>) => {
      if (e.key === "Enter") {
        e.preventDefault();
        e.currentTarget.blur();
      } else if (e.key === "Escape") {
        e.currentTarget.textContent = item.text;
        e.currentTarget.blur();
        setResets((n) => n + 1);
      }
    },
    [item.text]
  );

  const jump = useCallback(
    () => onJump?.(item.inizioMs),
    [item.inizioMs, onJump]
  );

  let className = "-mx-0.5 rounded-sm px-0.5";
  if (playing) {
    className += " bg-primary/15";
  }
  if (highlighted) {
    className += " ring-1 ring-primary/50";
  }
  if (partial) {
    className += " text-muted-foreground italic";
  }
  const time = onJump ? (
    <button
      className={`mr-1 select-none rounded-sm text-muted-foreground text-xs tabular-nums hover:text-foreground focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring group-hover/frase:opacity-100 ${playing || showTime ? "" : "opacity-0"}`}
      onClick={jump}
      onPointerDown={keepFocus}
      title={t("player.jump")}
      type="button"
    >
      {elapsedText(item.inizioMs)}
    </button>
  ) : null;
  if (!onEdit) {
    return (
      <span className="group/frase" data-phrase={phraseKey(item)}>
        {time}
        <span className={className} ref={span}>
          {item.text}
        </span>{" "}
      </span>
    );
  }
  return (
    <span className="group/frase" data-phrase={phraseKey(item)}>
      {time}
      {/* biome-ignore lint/a11y/useSemanticElements: un campo spezzerebbe il testo del turno, la Frase si corregge al suo posto */}
      <span
        aria-label={t("transcription.edit")}
        className={`${className} cursor-text empty:inline-block empty:min-w-8 empty:border-b empty:border-dashed hover:bg-accent focus:bg-accent focus:outline-none`}
        contentEditable="plaintext-only"
        key={`${resets}:${item.text}`}
        onBlur={save}
        onKeyDown={keyDown}
        ref={span}
        role="textbox"
        suppressContentEditableWarning
        tabIndex={0}
        title={t("transcription.edit")}
      >
        {item.text}
      </span>{" "}
    </span>
  );
}
