import {
  type FocusEvent,
  type FormEvent,
  type KeyboardEvent,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import {
  type PhraseRef,
  phraseKey,
  type Turn,
  textItemsOf,
  turnBody,
} from "./phrases";

export type EditTurn = (
  turn: Turn,
  original: string,
  text: string
) => Promise<boolean>;

export type CommitTurn = () => Promise<string | null>;

function fit(el: HTMLTextAreaElement) {
  el.style.height = "auto";
  el.style.height = `${el.scrollHeight}px`;
}

/** Campo nativo non controllato: riproduzione e rerender non cambiano cursore, selezione o undo. */
export function TurnEditor({
  turn,
  onEdit,
  highlight,
  playing,
  onCommit,
}: {
  turn: Turn;
  onEdit: EditTurn;
  highlight?: PhraseRef | null;
  playing: string[];
  onCommit: (commit: CommitTurn) => void;
}) {
  const { t } = useTranslation();
  const text = turnBody(turn);
  const input = useRef<HTMLTextAreaElement>(null);
  const session = useRef<{
    turn: Turn;
    original: string;
    draft: string;
  } | null>(null);
  const cancelled = useRef(false);
  const saving = useRef<Promise<string | null> | null>(null);
  const failedDraft = useRef(false);
  const [pending, setPending] = useState(false);
  const [dirty, setDirty] = useState(false);
  const [editing, setEditing] = useState(false);
  const [saveFailed, setSaveFailed] = useState(false);

  useLayoutEffect(() => {
    const el = input.current;
    if (!el) {
      return;
    }
    if (document.activeElement !== el && !failedDraft.current) {
      el.value = text;
    }
    fit(el);
  }, [text]);
  useEffect(() => {
    const el = input.current;
    if (!el) {
      return;
    }
    let width = el.clientWidth;
    const observer = new ResizeObserver(() => {
      if (el.clientWidth !== width) {
        width = el.clientWidth;
        fit(el);
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const save = useCallback<CommitTurn>(() => {
    if (saving.current) {
      return saving.current;
    }
    const { current } = session;
    const el = input.current;
    if (!el) {
      return Promise.resolve(null);
    }
    if (!current || cancelled.current) {
      cancelled.current = false;
      return Promise.resolve(el.value);
    }
    const draft = el.value;
    if (draft === current.original) {
      return Promise.resolve(draft);
    }
    setPending(true);
    setSaveFailed(false);
    failedDraft.current = true;
    const job = (async () => {
      try {
        const saved = await onEdit(current.turn, current.original, draft);
        if (saved) {
          failedDraft.current = false;
          setDirty(false);
          session.current = { ...current, draft, original: draft };
        }
        setSaveFailed(!saved);
        return saved ? draft : null;
      } catch {
        setSaveFailed(true);
        return null;
      } finally {
        saving.current = null;
        setPending(false);
      }
    })();
    saving.current = job;
    return job;
  }, [onEdit]);
  useLayoutEffect(() => {
    onCommit(save);
  }, [onCommit, save]);
  const blur = useCallback(() => {
    setEditing(false);
    save();
  }, [save]);
  const keyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key !== "Escape" || e.nativeEvent.isComposing) {
        return;
      }
      e.preventDefault();
      cancelled.current = true;
      e.currentTarget.value = session.current?.draft ?? text;
      setDirty(e.currentTarget.value !== text);
      fit(e.currentTarget);
      e.currentTarget.blur();
    },
    [text]
  );
  const focus = useCallback(
    (e: FocusEvent<HTMLTextAreaElement>) => {
      session.current = { draft: e.currentTarget.value, original: text, turn };
      cancelled.current = false;
      setEditing(true);
    },
    [turn, text]
  );
  const inputChanged = useCallback(
    (e: FormEvent<HTMLTextAreaElement>) => {
      setDirty(e.currentTarget.value !== text);
      fit(e.currentTarget);
    },
    [text]
  );
  const corrected = turn.items.some(
    (item) => "testoCorretto" in item && item.testoCorretto
  );
  const searchHit = turn.items.some(
    (item) =>
      item.ingresso === highlight?.ingresso &&
      item.phraseId === highlight.phraseId
  );
  const units = textItemsOf(turn);

  return (
    <div className="relative">
      {/* La base mostra solo il fondo delle Frasi in ascolto; testo e cursore sono del campo nativo. */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 whitespace-pre-wrap break-words text-transparent"
      >
        {units.map((item, index) => (
          <span key={phraseKey(item)}>
            {index === 0 ? null : "\n"}
            <span
              className={
                !(corrected || editing || dirty) &&
                playing.includes(phraseKey(item))
                  ? "rounded-sm bg-play/18"
                  : undefined
              }
              data-phrase={phraseKey(item)}
            >
              {item.text}
            </span>
          </span>
        ))}
        {turn.items
          .filter((item) => !units.includes(item))
          .map((item) => (
            <span
              className="absolute inset-0"
              data-phrase={phraseKey(item)}
              key={phraseKey(item)}
            />
          ))}
      </div>
      <textarea
        aria-busy={pending}
        aria-invalid={saveFailed}
        aria-label={t("transcription.editTurn", {
          name: turn.label ?? t("transcription.text"),
        })}
        className={`transcript-editor relative block min-h-[1lh] w-full select-text resize-none overflow-hidden rounded-sm bg-transparent p-0 text-inherit leading-[inherit] outline-none hover:bg-accent/30 focus-visible:ring-[3px] focus-visible:ring-ring/40 ${searchHit ? "ring-1 ring-play/60" : ""}`}
        defaultValue={text}
        onBlur={blur}
        onFocus={focus}
        onInput={inputChanged}
        onKeyDown={keyDown}
        placeholder={t("transcription.emptyTurn")}
        readOnly={pending}
        ref={input}
        rows={1}
        spellCheck
      />
      {saveFailed ? (
        <p className="mt-2 text-destructive text-sm" role="alert">
          {t("transcription.turnSaveFailed")}
        </p>
      ) : null}
    </div>
  );
}
