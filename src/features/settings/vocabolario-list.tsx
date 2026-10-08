import { X } from "lucide-react";
import {
  type ChangeEvent,
  type ClipboardEvent,
  type FormEvent,
  type MouseEvent,
  useCallback,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { SaveTick, SettingFeedback } from "./setting-feedback";
import { useSettings } from "./settings-context";
import {
  type AddMessage,
  addMessage,
  addTermini,
  removeTermine,
} from "./vocabolario";

/**
 * Impostazioni → Trascrizione: i Termini del Vocabolario come pillole. Invio o Aggiungi aggiunge
 * il testo del campo, incollare più righe aggiunge un Termine per riga; ogni modifica si salva subito.
 */
export function VocabolarioList() {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  const termini = settings.vocabolario ?? [];
  const [draft, setDraft] = useState("");
  const [message, setMessage] = useState<AddMessage | null>(null);
  const field = useRef<HTMLInputElement>(null);
  const title = t("settings.vocabolario.title");

  const add = useCallback(
    (testo: string) => {
      const esito = addTermini(termini, testo);
      const shown = addMessage(esito);
      setMessage(shown);
      if (esito.aggiunti > 0) {
        // L'intenzione si riapplica alla lista corrente, comprese le scelte ancora in coda.
        save(
          (current) => ({
            ...current,
            vocabolario: addTermini(current.vocabolario ?? [], testo).termini,
          }),
          "vocabolario"
        );
      }
      return shown?.keep ?? false;
    },
    [save, termini]
  );
  const submit = useCallback(
    (e: FormEvent<HTMLFormElement>) => {
      e.preventDefault();
      if (!add(draft)) {
        setDraft("");
      }
    },
    [add, draft]
  );
  const paste = useCallback(
    (e: ClipboardEvent<HTMLInputElement>) => {
      const text = e.clipboardData.getData("text");
      if (text.trim().includes("\n")) {
        e.preventDefault();
        add(text);
      }
    },
    [add]
  );
  const change = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setDraft(e.target.value);
    setMessage(null);
  }, []);
  const remove = useCallback(
    (e: MouseEvent<HTMLButtonElement>) => {
      const termine = e.currentTarget.value;
      setMessage(null);
      // La pillola sparisce con il suo pulsante: il focus non resta sul documento.
      field.current?.focus();
      save(
        (current) => ({
          ...current,
          vocabolario: removeTermine(current.vocabolario ?? [], termine),
        }),
        "vocabolario"
      );
    },
    [save]
  );

  return (
    <section
      aria-labelledby="vocabolario-title"
      className="flex min-w-0 flex-col gap-3 border-t pt-7"
    >
      <div className="flex flex-col gap-1">
        <h3
          className="flex items-center gap-1 font-medium text-sm"
          id="vocabolario-title"
        >
          {title}
          <SaveTick label={title} name="vocabolario" />
        </h3>
        {termini.length === 0 ? (
          <p className="text-muted-foreground text-sm leading-relaxed">
            {t("settings.vocabolario.description")}
          </p>
        ) : null}
      </div>
      <form className="flex max-w-xl items-center gap-2" onSubmit={submit}>
        <Input
          aria-describedby="vocabolario-message"
          aria-invalid={message?.keep ? true : undefined}
          aria-label={t("settings.vocabolario.label")}
          className="h-9 min-w-0 flex-1"
          onChange={change}
          onPaste={paste}
          placeholder={t("settings.vocabolario.placeholder")}
          ref={field}
          value={draft}
        />
        <Button
          className="h-9"
          disabled={!draft.trim()}
          type="submit"
          variant="outline"
        >
          {t("settings.vocabolario.add")}
        </Button>
      </form>
      <p
        aria-live="polite"
        className="-mt-1 text-destructive text-xs empty:hidden"
        id="vocabolario-message"
      >
        {message ? t(message.key, message.params) : null}
      </p>
      <SettingFeedback label={title} name="vocabolario" />
      {termini.length > 0 ? (
        <ul
          aria-labelledby="vocabolario-title"
          className="flex flex-wrap gap-2"
        >
          {termini.map((termine) => (
            <li
              className="inline-flex min-w-0 max-w-full items-center gap-0.5 rounded-lg border bg-card py-0.5 pr-0.5 pl-2.5 text-foreground/85 text-sm"
              key={termine}
            >
              <span className="min-w-0 [overflow-wrap:anywhere]">
                {termine}
              </span>
              <button
                aria-label={t("settings.vocabolario.remove", { termine })}
                className="inline-flex size-6 shrink-0 items-center justify-center rounded-md text-foreground/70 transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
                onClick={remove}
                type="button"
                value={termine}
              >
                <X aria-hidden className="size-3.5" />
              </button>
            </li>
          ))}
        </ul>
      ) : null}
      <p className="text-muted-foreground text-xs leading-relaxed">
        {t("settings.vocabolario.whisper")}
      </p>
    </section>
  );
}
