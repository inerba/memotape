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
import {
  type Conversation,
  type Parlante,
  type PhraseRef,
  turnsOf,
} from "@/features/transcription/phrases";

/**
 * La trascrizione a turni: l'etichetta della voce (un clic rinomina il Parlante) e le sue Frasi una
 * dopo l'altra. Con `onEdit` un clic su una Frase la rende modificabile: Invio o l'uscita salvano,
 * Esc ripristina. `highlight` è la Frase di un risultato della ricerca, evidenziata e portata in
 * vista.
 */
export function TranscriptView({
  conversation,
  highlight,
  onEdit,
  onRename,
  parlanti,
}: {
  conversation: Conversation;
  highlight?: PhraseRef | null;
  /** Salva la correzione; `false` se non si è salvata, e il testo resta com'è scritto. */
  onEdit?: (phrase: PhraseRef, text: string) => Promise<boolean>;
  onRename?: (voce: Parlante) => void;
  /** I Parlanti che si possono rinominare. */
  parlanti: Parlante[];
}) {
  const { t } = useTranslation();
  return (
    <section
      aria-label={t("transcription.text")}
      className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto pr-2 leading-relaxed"
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
              {turn.items.map((item) => {
                const partial = conversation.partials.includes(
                  item as TranscriptPartial
                );
                return (
                  <PhraseText
                    highlighted={
                      highlight?.ingresso === item.ingresso &&
                      highlight.phraseId === item.phraseId
                    }
                    item={item}
                    key={`${item.ingresso}:${item.phraseId}${partial ? ":parziale" : ""}`}
                    onEdit={partial ? undefined : onEdit}
                    partial={partial}
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

/** Il testo scritto nella Frase: senza a capo né spazi non separabili. */
function typedText(el: HTMLElement): string {
  return (el.textContent ?? "").replace(/\s/g, " ");
}

/**
 * Una Frase, seguita da uno spazio. Modificabile solo come testo semplice; il `key` del suo span
 * cambia quando il testo arriva da fuori o con Esc, così React non scrive mai nel testo modificato.
 */
function PhraseText({
  highlighted,
  item,
  onEdit,
  partial,
}: {
  highlighted: boolean;
  item: TranscriptPhrase | TranscriptPartial;
  onEdit?: (phrase: PhraseRef, text: string) => Promise<boolean>;
  /** Il Parziale della Frase in corso, ancora provvisorio. */
  partial: boolean;
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

  let className = "-mx-0.5 rounded-sm px-0.5";
  if (highlighted) {
    className += " bg-primary/15";
  }
  if (partial) {
    className += " text-muted-foreground italic";
  }
  if (!onEdit) {
    return (
      <>
        <span className={className} ref={span}>
          {item.text}
        </span>{" "}
      </>
    );
  }
  return (
    <>
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
    </>
  );
}
