import type { TFunction } from "i18next";
import { Circle, Settings } from "lucide-react";
import {
  type ChangeEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useOutlet } from "react-router";
import { type AppError, commands, events } from "@/bindings";
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
import { afterRecording } from "@/features/recording/recording";
import { RecordingPanel } from "@/features/recording/recording-panel";
import { speechLanguageChoice } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { fileName, isBino } from "@/features/source/file-name";
import {
  afterTranscription,
  needsSettings,
  type Status,
  statusText,
  withDiarizing,
  withLiveError,
  withProgress,
} from "@/features/status/status";
import {
  type Conversation,
  conversationText,
  EMPTY_CONVERSATION,
  withoutPartials,
  withParlanti,
  withPartial,
  withPhrase,
} from "@/features/transcription/phrases";
import {
  type ReplaceAction,
  replaceDescription,
  transcribeNeedsConfirm,
} from "@/features/transcription/replace";

const COPIED_MS = 2000;

export function HomePage() {
  const { t } = useTranslation();
  const [source, setSource] = useState<string | null>(null);
  const [text, setText] = useState("");
  // Le Frasi dell'ultima Trascrizione (o del Bino aperto) e i Parziali in corso (Nemotron), uno per
  // Ingresso: l'area li mostra mentre arrivano, poi il testo resta modificabile.
  const [conversation, setConversation] =
    useState<Conversation>(EMPTY_CONVERSATION);
  // Gli eventi possono arrivare dopo la risposta di `transcribe`: un Parziale tardivo si ignora.
  const acceptPartials = useRef<boolean>(false);
  // Il testo è stato modificato a mano dopo l'ultima Trascrizione: Copia testo lo copia com'è.
  const edited = useRef<boolean>(false);
  const { loadError, save, settings } = useSettings();
  // Impostazioni illeggibili all'avvio: la status bar lo dice finché non c'è altro da mostrare.
  const [status, setStatus] = useState<Status>(() =>
    loadError
      ? { error: loadError, phase: "failed" }
      : { phase: "idle", source: null }
  );
  const [copied, setCopied] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  // Cosa aspetta la conferma prima di sostituire il testo nell'area: un'Attività o l'apertura di
  // un Bino.
  const [confirmReplace, setConfirmReplace] = useState<{
    action: ReplaceAction;
    path?: string;
  } | null>(null);
  // Il Bino del doppio clic in Esplora file, finché non si può aprire.
  const [pendingBino, setPendingBino] = useState<string | null>(null);
  const running = status.phase === "transcribing";
  const recording = status.phase === "recording";
  // Dopo Stop, finché la Trascrizione dal vivo smaltisce la coda: fa ancora parte della Registrazione.
  const completing = status.phase === "completing";
  const paused = status.phase === "recording" && status.paused;
  // Una Attività alla volta: durante l'una, l'altra e Sfoglia sono disabilitate.
  const busy = running || recording || completing;
  const live = settings.trascrizioneDalVivo ?? false;
  // Il testo arriva dalla pipeline: non si modifica finché non è finita.
  const writing = running || completing || (recording && live);
  // Impostazioni, aperta sopra questa finestra.
  const settingsPage = useOutlet();
  const navigate = useNavigate();
  const models = useModels();
  const modelLanguages =
    models.find((m) => m.id === settings.model)?.languages ?? null;
  const { options: languages, value: language } = speechLanguageChoice(
    modelLanguages,
    settings.speechLanguage
  );

  useEffect(() => {
    // Le Frasi in ordine di inizio; la Frase fissa il Parziale del suo Ingresso.
    const phrases = events.transcriptPhrase.listen(({ payload }) => {
      setConversation((current) => withPhrase(current, payload));
    });
    const partials = events.transcriptPartial.listen(({ payload }) => {
      if (acceptPartials.current) {
        setConversation((current) => withPartial(current, payload));
      }
    });
    const progress = events.transcriptionProgress.listen(({ payload }) => {
      setStatus((current) => withProgress(current, payload.percent));
    });
    // Finita la Trascrizione, Riconosci i parlanti attribuisce le Frasi ai Parlanti.
    const diarizing = events.diarizationStarted.listen(() => {
      setStatus(withDiarizing);
    });
    const assigned = events.speakersAssigned.listen(({ payload }) => {
      setConversation((current) => withParlanti(current, payload.speakers));
    });
    // La Trascrizione dal vivo si è fermata: la Registrazione continua senza testo.
    const liveFailed = events.liveTranscriptionFailed.listen(({ payload }) => {
      setConversation(withoutPartials);
      setStatus((current) => withLiveError(current, payload.error));
    });
    // Il Bino dell'avvio, e quelli del doppio clic con l'app aperta. Si prende dopo aver registrato
    // il listener, così uno arrivato nel frattempo non si perde.
    const takeBino = () =>
      commands.takePendingBino().then((path) => path && setPendingBino(path));
    const binoRequested = events.binoRequested.listen(takeBino);
    binoRequested.then(takeBino);
    return () => {
      phrases.then((stop) => stop());
      partials.then((stop) => stop());
      progress.then((stop) => stop());
      liveFailed.then((stop) => stop());
      diarizing.then((stop) => stop());
      assigned.then((stop) => stop());
      binoRequested.then((stop) => stop());
    };
  }, []);

  // Il testo dell'area segue le Frasi che arrivano, una per riga (con gli Ingressi separati come
  // conversazione); dopo si può modificare.
  useEffect(() => {
    setText(conversationText(conversation.phrases, [], t));
  }, [conversation.phrases, t]);

  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = setTimeout(() => setCopied(false), COPIED_MS);
    return () => clearTimeout(timer);
  }, [copied]);

  // Un Bino si apre con il testo che contiene, senza ritrascrivere.
  const openBino = useCallback(async (path: string) => {
    const result = await commands.openBino(path);
    if (result.status === "error") {
      setStatus({ error: result.error, phase: "failed" });
      return;
    }
    setSource(path);
    setConversation({ partials: [], phrases: result.data });
    edited.current = false;
    setStatus({ phase: "idle", source: path });
  }, []);

  // Una Sorgente scelta con Sfoglia o con il doppio clic su un Bino in Esplora file.
  const openPath = useCallback(
    (path: string) => {
      if (!isBino(path)) {
        setSource(path);
        setStatus({ phase: "idle", source: path });
      } else if (text.trim()) {
        setConfirmReplace({ action: "open", path });
      } else {
        openBino(path);
      }
    },
    [openBino, text]
  );

  const browse = useCallback(async () => {
    const picked = await commands.pickSource(t("source.filter"));
    if (picked) {
      openPath(picked);
    }
  }, [openPath, t]);

  // Un Bino arrivato con il doppio clic in Esplora file aspetta che finisca l'Attività (Sfoglia
  // intanto è disabilitata) e che si chiuda una conferma aperta, che altrimenti cambierebbe azione.
  // Si apre sulla finestra principale, anche se c'era Impostazioni sopra.
  useEffect(() => {
    if (pendingBino && !busy && !confirmReplace) {
      setPendingBino(null);
      navigate("/");
      openPath(pendingBino);
    }
  }, [busy, confirmReplace, navigate, openPath, pendingBino]);

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
    setConversation(EMPTY_CONVERSATION);
    acceptPartials.current = true;
    edited.current = false;
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
      // Il Parziale di una Frase annullata non resta nel testo.
      acceptPartials.current = false;
      setConversation(withoutPartials);
    }
  }, [source]);

  const record = useCallback(async () => {
    if (live) {
      setText("");
      setConversation(EMPTY_CONVERSATION);
      acceptPartials.current = true;
      edited.current = false;
    }
    setCancelling(false);
    setStatus({ paused: false, phase: "recording" });
    try {
      const after = afterRecording(
        await commands.record(t("recording.prefix"))
      );
      if (after.source) {
        setSource(after.source);
      }
      setStatus(after.status);
    } catch (e) {
      setStatus({
        error: { code: "internal", detail: String(e) },
        phase: "failed",
      });
    } finally {
      acceptPartials.current = false;
      setConversation(withoutPartials);
    }
  }, [live, t]);

  const setPaused = useCallback((value: boolean) => {
    setStatus((current) =>
      current.phase === "recording" ? { ...current, paused: value } : current
    );
  }, []);

  // Il testo nell'area, anche se modificato a mano, e quello dentro un Bino si sostituiscono solo
  // dopo conferma.
  const requestTranscription = useCallback(() => {
    if (transcribeNeedsConfirm(text, source)) {
      setConfirmReplace({ action: "transcribe" });
    } else {
      transcribe();
    }
  }, [source, text, transcribe]);

  // Con Trascrivi dal vivo la Registrazione sostituisce il testo: anche lei chiede conferma.
  const requestRecording = useCallback(() => {
    if (live && text.trim()) {
      setConfirmReplace({ action: "record" });
    } else {
      record();
    }
  }, [live, record, text]);

  const closeConfirm = useCallback((opened: boolean) => {
    if (!opened) {
      setConfirmReplace(null);
    }
  }, []);

  const replace = useCallback(() => {
    if (confirmReplace?.action === "record") {
      record();
    } else if (confirmReplace?.action === "open" && confirmReplace.path) {
      openBino(confirmReplace.path);
    } else {
      transcribe();
    }
  }, [confirmReplace, openBino, record, transcribe]);

  const failed = useCallback(
    (error: AppError) => setStatus({ error, phase: "failed" }),
    []
  );

  const cancel = useCallback(async () => {
    setCancelling(true);
    try {
      // `false`: il backend non ha ancora avviato l'Attività, Annulla va ripremuto.
      setCancelling(await commands.cancelTranscription());
    } catch {
      setCancelling(false);
    }
  }, []);

  // Il documento dell'ultima Trascrizione, in testo semplice o Markdown secondo le impostazioni;
  // il testo modificato a mano si copia com'è.
  const copy = useCallback(async () => {
    try {
      const rendered = edited.current ? null : await commands.transcriptText();
      await navigator.clipboard.writeText(rendered ?? text);
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

  const edit = useCallback((e: ChangeEvent<HTMLTextAreaElement>) => {
    edited.current = true;
    setText(e.target.value);
  }, []);

  // La sezione Trascrizione segue l'Attività: c'è durante e dopo una Trascrizione, anche dal vivo,
  // o se c'è testo.
  const showTranscription =
    writing || status.phase === "finished" || text !== "";

  return (
    <>
      <div className="flex h-screen flex-col" inert={settingsPage !== null}>
        <main className="flex min-h-0 flex-1 flex-col gap-4 p-6">
          <section className="flex items-center gap-3">
            <Button disabled={busy} onClick={browse} variant="outline">
              {t("source.browse")}
            </Button>
            <Button
              disabled={busy}
              onClick={requestRecording}
              variant="outline"
            >
              <Circle className="fill-destructive text-destructive" />
              {t("recording.start")}
            </Button>
            <SettingCheckbox
              disabled={busy}
              label={t("recording.live")}
              name="trascrizioneDalVivo"
              onError={failed}
            />
            {source ? (
              <div className="min-w-0 flex-1">
                <button
                  className="block max-w-full cursor-pointer truncate rounded-sm text-left text-sm underline decoration-muted-foreground/40 underline-offset-4 hover:decoration-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  onClick={open}
                  title={
                    isBino(source)
                      ? t("source.reveal", { path: source })
                      : t("source.open", { path: source })
                  }
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
              disabled={busy}
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
            <SettingCheckbox
              disabled={busy}
              label={t("transcription.parlanti")}
              name="parlantiFile"
              note={t("transcription.parlantiNote")}
              onError={failed}
            />
            <Button disabled={!source || busy} onClick={requestTranscription}>
              {running ? t("transcription.running") : t("transcription.start")}
            </Button>
            {running || completing ? (
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
          {recording ? (
            <RecordingPanel onPausedChange={setPaused} paused={paused} />
          ) : null}
          <AlertDialog
            onOpenChange={closeConfirm}
            open={confirmReplace !== null}
          >
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  {t("transcription.replace.title")}
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {t(replaceDescription(confirmReplace?.action, source))}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>
                  {t("transcription.replace.keep")}
                </AlertDialogCancel>
                <AlertDialogAction onClick={replace}>
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
                readOnly={writing}
                value={shownText(writing, conversation, text, t)}
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
        <StatusBar status={status} />
      </div>
      {settingsPage}
    </>
  );
}

/** Una casella della barra che salva subito un'impostazione; un errore va nella status bar. */
function SettingCheckbox({
  disabled,
  label,
  name,
  note,
  onError,
}: {
  disabled: boolean;
  label: string;
  name: "parlantiFile" | "trascrizioneDalVivo";
  note?: string;
  onError: (error: AppError) => void;
}) {
  const { save, settings } = useSettings();
  const change = useCallback(
    async (e: ChangeEvent<HTMLInputElement>) => {
      const error = await save({ ...settings, [name]: e.target.checked });
      if (error) {
        onError(error);
      }
    },
    [name, onError, save, settings]
  );
  return (
    <label
      className="flex shrink-0 items-center gap-2 text-sm has-[:disabled]:opacity-50"
      title={note}
    >
      <input
        checked={settings[name] ?? false}
        className="size-4 accent-primary"
        disabled={disabled}
        onChange={change}
        type="checkbox"
      />
      {label}
    </label>
  );
}

/** Mentre la Trascrizione arriva l'area mostra anche i Parziali; poi il testo, modificabile. */
function shownText(
  writing: boolean,
  conversation: Conversation,
  text: string,
  t: TFunction
) {
  return writing
    ? conversationText(conversation.phrases, conversation.partials, t)
    : text;
}

/** La status bar: il messaggio, il link alle Impostazioni quando serve e l'avanzamento. */
function StatusBar({ status }: { status: Status }) {
  const { t } = useTranslation();
  const message = statusText(status, t);
  const failed = status.phase === "failed";
  return (
    <footer
      aria-live="polite"
      className="flex items-center gap-3 border-t px-6 py-2 text-muted-foreground text-xs"
      role={failed ? "alert" : "status"}
    >
      <span
        className={
          failed
            ? "min-w-0 flex-1 truncate text-destructive"
            : "min-w-0 flex-1 truncate"
        }
        title={message}
      >
        {message}
      </span>
      {needsSettings(status) ? (
        <Link
          className="shrink-0 text-foreground underline underline-offset-4"
          to="/settings"
        >
          {t("status.openModels")}
        </Link>
      ) : null}
      {status.phase === "transcribing" || status.phase === "completing" ? (
        <progress
          aria-label={t("status.progress")}
          className="h-1.5 w-40 shrink-0 accent-primary"
          max={100}
          value={status.percent ?? undefined}
        />
      ) : null}
    </footer>
  );
}
