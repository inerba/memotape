import { Settings } from "lucide-react";
import { type ChangeEvent, useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useOutlet } from "react-router";
import { commands, events, type TranscriptPartial } from "@/bindings";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useModels } from "@/features/models/use-models";
import { speechLanguageChoice } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { fileName } from "@/features/source/file-name";
import {
  afterTranscription,
  type Status,
  statusText,
  withProgress,
} from "@/features/status/status";
import {
  afterPhrase,
  appendPhrase,
  withPartial,
} from "@/features/transcription/phrases";

const COPIED_MS = 2000;

export function HomePage() {
  const { t } = useTranslation();
  const [source, setSource] = useState<string | null>(null);
  const [text, setText] = useState("");
  // Il Parziale della Frase in corso (Nemotron), mostrato nella riga dopo le Frasi.
  const [partial, setPartial] = useState<TranscriptPartial | null>(null);
  const [status, setStatus] = useState<Status>({ phase: "idle", source: null });
  const [copied, setCopied] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const [confirmReplace, setConfirmReplace] = useState(false);
  const running = status.phase === "transcribing";
  // Impostazioni, aperta sopra questa finestra.
  const settingsPage = useOutlet();
  const { save, settings } = useSettings();
  const models = useModels();
  const modelLanguages =
    models.find((m) => m.id === settings.model)?.languages ?? null;
  const { options: languages, value: language } = speechLanguageChoice(
    modelLanguages,
    settings.speechLanguage
  );

  useEffect(() => {
    // Una Frase per riga, nell'ordine in cui arrivano; la Frase fissa il suo Parziale.
    const phrases = events.transcriptPhrase.listen(({ payload }) => {
      setText((current) => appendPhrase(current, payload.text));
      setPartial((current) => afterPhrase(current, payload.phraseId));
    });
    const partials = events.transcriptPartial.listen(({ payload }) => {
      setPartial(payload);
    });
    const progress = events.transcriptionProgress.listen(({ payload }) => {
      setStatus((current) => withProgress(current, payload.percent));
    });
    return () => {
      phrases.then((stop) => stop());
      partials.then((stop) => stop());
      progress.then((stop) => stop());
    };
  }, []);

  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = setTimeout(() => setCopied(false), COPIED_MS);
    return () => clearTimeout(timer);
  }, [copied]);

  const browse = useCallback(async () => {
    const picked = await commands.pickSource(t("source.filter"));
    if (picked) {
      setSource(picked);
      setStatus({ phase: "idle", source: picked });
    }
  }, [t]);

  const open = useCallback(async () => {
    if (!source) {
      return;
    }
    const result = await commands.openSource(source);
    if (result.status === "error") {
      setStatus({ error: result.error, phase: "failed" });
    }
  }, [source]);

  const transcribe = useCallback(async () => {
    if (!source) {
      return;
    }
    setText("");
    setPartial(null);
    setCancelling(false);
    setStatus({ percent: null, phase: "transcribing" });
    try {
      setStatus(afterTranscription(await commands.transcribe(source)));
    } catch (e) {
      // `typedError` rilancia gli `Error` di IPC: la Trascrizione non deve restare "in corso".
      setStatus({
        error: { code: "internal", detail: String(e) },
        phase: "failed",
      });
    } finally {
      // Il Parziale di una Frase annullata o finita vuota non resta nel testo.
      setPartial(null);
    }
  }, [source]);

  // Il testo nell'area, anche se modificato a mano, si sostituisce solo dopo conferma.
  const requestTranscription = useCallback(() => {
    if (text.trim()) {
      setConfirmReplace(true);
    } else {
      transcribe();
    }
  }, [text, transcribe]);

  const cancel = useCallback(async () => {
    setCancelling(true);
    try {
      // `false`: il backend non ha ancora avviato l'Attività, Annulla va ripremuto.
      setCancelling(await commands.cancelTranscription());
    } catch {
      setCancelling(false);
    }
  }, []);

  const copy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
    } catch (e) {
      setStatus({
        error: { code: "internal", detail: String(e) },
        phase: "failed",
      });
    }
  }, [text]);

  const chooseLanguage = useCallback(
    async (e: ChangeEvent<HTMLSelectElement>) => {
      const { value } = e.target;
      const speechLanguage = languages.find((l) => l === value) ?? "auto";
      const error = await save({ ...settings, speechLanguage });
      if (error) {
        setStatus({ error, phase: "failed" });
      }
    },
    [languages, save, settings]
  );

  const edit = useCallback(
    (e: ChangeEvent<HTMLTextAreaElement>) => setText(e.target.value),
    []
  );

  // La sezione Trascrizione segue l'Attività: c'è durante e dopo una Trascrizione, o se c'è testo.
  const showTranscription =
    running || status.phase === "finished" || text !== "";
  const message = statusText(status, t);

  return (
    <>
      <div className="flex h-screen flex-col" inert={settingsPage !== null}>
        <main className="flex min-h-0 flex-1 flex-col gap-4 p-6">
          <section className="flex items-center gap-3">
            <Button disabled={running} onClick={browse} variant="outline">
              {t("source.browse")}
            </Button>
            {source ? (
              <div className="min-w-0 flex-1">
                <button
                  className="block max-w-full cursor-pointer truncate rounded-sm text-left text-sm underline decoration-muted-foreground/40 underline-offset-4 hover:decoration-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  onClick={open}
                  title={t("source.open", { path: source })}
                  type="button"
                >
                  {fileName(source)}
                </button>
              </div>
            ) : (
              <span className="min-w-0 flex-1 truncate text-muted-foreground text-sm">
                {t("source.none")}
              </span>
            )}
            <select
              aria-label={t("speechLanguage.label")}
              className="h-9 rounded-md border border-input bg-transparent px-2 text-sm shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
              disabled={running}
              onChange={chooseLanguage}
              title={t("speechLanguage.label")}
              value={language}
            >
              <option value="auto">{t("speechLanguage.auto")}</option>
              {languages.map((l) => (
                <option key={l} value={l}>
                  {t(`speechLanguage.languages.${l}`)}
                </option>
              ))}
            </select>
            <Button
              disabled={!source || running}
              onClick={requestTranscription}
            >
              {running ? t("transcription.running") : t("transcription.start")}
            </Button>
            {running ? (
              <Button disabled={cancelling} onClick={cancel} variant="outline">
                {cancelling
                  ? t("transcription.cancelling")
                  : t("transcription.cancel")}
              </Button>
            ) : null}
            <Button asChild size="icon" variant="ghost">
              <Link
                aria-label={t("settings.open")}
                title={t("settings.open")}
                to="/settings"
              >
                <Settings />
              </Link>
            </Button>
          </section>
          <AlertDialog onOpenChange={setConfirmReplace} open={confirmReplace}>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  {t("transcription.replace.title")}
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {t("transcription.replace.description")}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>
                  {t("transcription.replace.keep")}
                </AlertDialogCancel>
                <AlertDialogAction onClick={transcribe}>
                  {t("transcription.replace.confirm")}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
          {showTranscription ? (
            <section className="flex min-h-0 flex-1 flex-col gap-2">
              <Textarea
                aria-label={t("transcription.text")}
                className="flex-1 resize-none"
                onChange={edit}
                readOnly={running}
                value={withPartial(text, partial)}
              />
              <div className="flex justify-end">
                <Button
                  disabled={!text}
                  onClick={copy}
                  size="sm"
                  variant="outline"
                >
                  {copied ? t("transcription.copied") : t("transcription.copy")}
                </Button>
              </div>
            </section>
          ) : null}
        </main>
        <footer
          aria-live="polite"
          className="flex items-center gap-3 border-t px-6 py-2 text-muted-foreground text-xs"
          role={status.phase === "failed" ? "alert" : "status"}
        >
          <span
            className={
              status.phase === "failed"
                ? "min-w-0 flex-1 truncate text-destructive"
                : "min-w-0 flex-1 truncate"
            }
            title={message}
          >
            {message}
          </span>
          {status.phase === "failed" && status.error.code === "modelMissing" ? (
            <Link
              className="shrink-0 text-foreground underline underline-offset-4"
              to="/settings"
            >
              {t("status.openModels")}
            </Link>
          ) : null}
          {status.phase === "transcribing" ? (
            <progress
              aria-label={t("status.progress")}
              className="h-1.5 w-40 shrink-0 accent-primary"
              max={100}
              value={status.percent ?? undefined}
            />
          ) : null}
        </footer>
      </div>
      {settingsPage}
    </>
  );
}
