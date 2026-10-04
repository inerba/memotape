import { Circle, FolderOpen } from "lucide-react";
import {
  type ChangeEvent,
  type MouseEvent,
  type ReactNode,
  type RefObject,
  useCallback,
  useEffect,
  useMemo,
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
import { AllBini } from "@/features/library/all-bini";
import { BinoHeader } from "@/features/library/bino-header";
import {
  biniOf,
  chosenRaccolta,
  raccoltaLabel,
} from "@/features/library/library";
import { SELECT } from "@/features/library/move-select";
import { Sidebar } from "@/features/library/sidebar";
import { useBinoOperations } from "@/features/library/use-bino-operations";
import { useLibrary } from "@/features/library/use-library";
import { useModels } from "@/features/models/use-models";
import { activityText, afterRecording } from "@/features/recording/recording";
import { RecordingPanel } from "@/features/recording/recording-panel";
import { speechLanguageChoice } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { fileName, isBino, movedPath } from "@/features/source/file-name";
import {
  afterTranscription,
  errorText,
  needsSettings,
  type Status,
  statusText,
  withDiarizing,
  withLiveError,
  withMovedSource,
  withProgress,
} from "@/features/status/status";
import { ParlantiBar } from "@/features/transcription/parlanti-bar";
import {
  type Conversation,
  conversationText,
  copyable,
  EMPTY_CONVERSATION,
  type Parlante,
  type PhraseRef,
  parlanteAt,
  parlantiOf,
  phraseRange,
  relabeled,
  shownText,
  withNome,
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
/** Quanto resta nella status bar l'errore di un'operazione sulla Libreria. */
const NOTICE_MS = 6000;

/** Il tratto dell'area da selezionare, appena l'area mostra lì `text`: la Frase di un risultato. */
interface Selection {
  end: number;
  start: number;
  text: string;
}

/** Un Bino aperto durante un'Attività: si consulta senza toccare la vista dell'Attività. */
interface Browsed {
  conversation: Conversation;
  path: string;
}

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
  // L'errore di un'operazione sulla Libreria: la status bar lo mostra per un po' sopra la fase, che
  // durante un'Attività non deve cambiare.
  const [notice, setNotice] = useState<AppError | null>(null);
  const [copied, setCopied] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  // Cosa aspetta la conferma prima di sostituire il testo nell'area: un'Attività o l'apertura di
  // un Bino.
  const [confirmReplace, setConfirmReplace] = useState<{
    action: ReplaceAction;
    path?: string;
    phrase?: PhraseRef;
  } | null>(null);
  // La Frase di un risultato della ricerca da evidenziare nell'area, appena la mostra.
  const [selection, setSelection] = useState<Selection | null>(null);
  const area = useRef<HTMLTextAreaElement>(null);
  // Il Bino del doppio clic in Esplora file, finché non si può aprire.
  const [pendingBino, setPendingBino] = useState<string | null>(null);
  // Il Parlante di cui si sta scrivendo il nome nuovo.
  const [renaming, setRenaming] = useState<Parlante | null>(null);
  // Il Bino aperto dalla barra laterale durante un'Attività.
  const [browsed, setBrowsed] = useState<Browsed | null>(null);
  // L'elenco completo dei Bini della Raccolta nell'area principale.
  const [listOpen, setListOpen] = useState(false);
  // Il timer della Registrazione per la barra laterale.
  const [elapsedMs, setElapsedMs] = useState(0);
  const running = status.phase === "transcribing";
  const recording = status.phase === "recording";
  // Dopo Stop, finché la Trascrizione dal vivo smaltisce la coda: fa ancora parte della Registrazione.
  const completing = status.phase === "completing";
  const paused = status.phase === "recording" && status.paused;
  // Una Attività alla volta: durante l'una, l'altra e Apri file sono disabilitate.
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
  const library = useLibrary(setNotice);
  const raccolta = chosenRaccolta(settings.raccolta, library.raccolte);

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
    // La Trascrizione dal vivo si è fermata (o Riconosci i parlanti non ha il modello): la
    // Registrazione continua e la status bar lo dice.
    const liveFailed = events.liveTranscriptionFailed.listen(({ payload }) => {
      setConversation(withoutPartials);
      setStatus((current) => withLiveError(current, payload.error));
    });
    const ticks = events.recordingTick.listen(({ payload }) => {
      setElapsedMs(payload.elapsedMs);
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
      ticks.then((stop) => stop());
      binoRequested.then((stop) => stop());
    };
  }, []);

  // Il testo dell'area segue le Frasi che arrivano, una per riga (con gli Ingressi separati come
  // conversazione), e i nomi dei Parlanti; dopo si può modificare, e un testo modificato a mano non
  // si rigenera.
  const { parlanti, phrases } = conversation;
  useEffect(() => {
    if (!edited.current) {
      setText(conversationText({ parlanti, partials: [], phrases }, t));
    }
  }, [parlanti, phrases, t]);

  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = setTimeout(() => setCopied(false), COPIED_MS);
    return () => clearTimeout(timer);
  }, [copied]);

  useEffect(() => {
    if (!notice) {
      return;
    }
    const timer = setTimeout(() => setNotice(null), NOTICE_MS);
    return () => clearTimeout(timer);
  }, [notice]);

  // Il tratto di `value` (il testo dell'area) con la Frase `phrase` di `of`, da evidenziare.
  const selectionOf = useCallback(
    (value: string, of: Conversation, phrase?: PhraseRef) => {
      const range = phrase && phraseRange(value, of, phrase, t);
      return range
        ? { ...range, text: value.slice(range.start, range.end) }
        : null;
    },
    [t]
  );

  // Un Bino si apre con il testo che contiene, senza ritrascrivere; da un risultato della ricerca
  // con la Frase trovata evidenziata.
  const openBino = useCallback(
    async (path: string, phrase?: PhraseRef) => {
      const result = await commands.openBino(path);
      if (result.status === "error") {
        setStatus({ error: result.error, phase: "failed" });
        return;
      }
      const opened = { ...EMPTY_CONVERSATION, ...result.data };
      setSource(path);
      edited.current = false;
      setConversation(opened);
      setSelection(selectionOf(conversationText(opened, t), opened, phrase));
      setStatus({ phase: "idle", source: path });
    },
    [selectionOf, t]
  );

  // Durante un'Attività un Bino della Libreria si consulta accanto, senza toccarla; quello su cui
  // lavora l'Attività riporta alla sua vista.
  const browse = useCallback(
    async (path: string, phrase?: PhraseRef) => {
      if (path === source) {
        setBrowsed(null);
        return;
      }
      const result = await commands.openBino(path);
      if (result.status === "error") {
        setNotice(result.error);
      } else {
        const opened = { ...EMPTY_CONVERSATION, ...result.data };
        setBrowsed({ conversation: opened, path });
        setSelection(selectionOf(conversationText(opened, t), opened, phrase));
      }
    },
    [selectionOf, source, t]
  );

  // Finita l'Attività torna la sua vista, con il suo esito aperto (il Bino di una Registrazione).
  useEffect(() => {
    if (!busy) {
      setBrowsed(null);
    }
  }, [busy]);

  // Una Sorgente scelta con Apri file o con il doppio clic su un Bino in Esplora file. Un testo
  // modificato a mano si sostituisce solo dopo conferma.
  const openPath = useCallback(
    (path: string, phrase?: PhraseRef) => {
      if (!isBino(path)) {
        setSource(path);
        setStatus({ phase: "idle", source: path });
      } else if (edited.current && text.trim()) {
        setConfirmReplace({ action: "open", path, phrase });
      } else {
        openBino(path, phrase);
      }
    },
    [openBino, text]
  );

  const pickFile = useCallback(async () => {
    const picked = await commands.pickSource(t("source.filter"));
    if (picked) {
      setListOpen(false);
      openPath(picked);
    }
  }, [openPath, t]);

  // Un Bino della barra laterale, dell'elenco completo o della ricerca, con la Frase trovata.
  const openFromLibrary = useCallback(
    (path: string, phrase?: PhraseRef) => {
      setListOpen(false);
      setSelection(null);
      if (busy) {
        browse(path, phrase);
      } else if (path !== source) {
        openPath(path, phrase);
      } else if (phrase) {
        setSelection(selectionOf(text, conversation, phrase));
      } else {
        // Il titolo del Bino già aperto: torna all'inizio del testo.
        area.current?.scrollTo({ top: 0 });
      }
    },
    [browse, busy, conversation, openPath, selectionOf, source, text]
  );

  // "Attività in corso" riporta alla sua vista.
  const showActivity = useCallback(() => {
    setBrowsed(null);
    setListOpen(false);
  }, []);

  const showAll = useCallback(() => setListOpen(true), []);

  const chooseRaccolta = useCallback(
    async (value: string | null) => {
      const error = await save({ ...settings, raccolta: value });
      if (error) {
        setNotice(error);
      }
    },
    [save, settings]
  );

  // Un Bino aperto e spostato o rinominato resta aperto, con il percorso nuovo.
  const moved = useCallback((from: string, to: string) => {
    setSource((current) => current && movedPath(current, from, to));
    setBrowsed(
      (current) =>
        current && { ...current, path: movedPath(current.path, from, to) }
    );
    setStatus((current) => withMovedSource(current, from, to));
  }, []);

  // Il Bino nel Cestino, se era aperto, non lo è più; la vista di un'Attività però resta.
  const trashed = useCallback(
    (path: string) => {
      setBrowsed((current) => (current?.path === path ? null : current));
      if (path === source) {
        setSource(null);
        if (!busy) {
          edited.current = false;
          setConversation(EMPTY_CONVERSATION);
          setStatus({ phase: "idle", source: null });
        }
      }
    },
    [busy, source]
  );

  const { dialog, moveBino, renameBino, requestTrash, reveal } =
    useBinoOperations({
      onError: setNotice,
      onMoved: moved,
      onTrashed: trashed,
    });

  // Un Bino arrivato con il doppio clic in Esplora file aspetta che finisca l'Attività (Apri file
  // intanto è disabilitata) e che si chiuda una conferma aperta, che altrimenti cambierebbe azione.
  // Si apre sulla finestra principale, anche se c'era Impostazioni sopra.
  useEffect(() => {
    if (pendingBino && !busy && !confirmReplace) {
      setPendingBino(null);
      setListOpen(false);
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
    // Le Frasi di un file arrivano senza Parziali e l'area le mostra solo alla fine.
    setText("");
    setConversation(EMPTY_CONVERSATION);
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
    setElapsedMs(0);
    setListOpen(false);
    setStatus({ paused: false, phase: "recording" });
    try {
      const after = afterRecording(
        await commands.record(t("recording.prefix"), raccolta)
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
  }, [live, raccolta, t]);

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
      openBino(confirmReplace.path, confirmReplace.phrase);
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
  // il testo modificato a mano si copia com'è, e quello di un Bino consultato durante un'Attività
  // come lo mostra l'area.
  const copy = useCallback(async () => {
    try {
      let rendered: string | null = null;
      if (browsed) {
        rendered = conversationText(browsed.conversation, t);
      } else if (!edited.current) {
        rendered = await commands.transcriptText();
      }
      await navigator.clipboard.writeText(rendered ?? text);
      setCopied(true);
    } catch (e) {
      setNotice({ code: "internal", detail: String(e) });
    }
  }, [browsed, t, text]);

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

  // I Parlanti si rinominano a Trascrizione finita.
  const parlantiList = useMemo(
    () => (busy ? [] : parlantiOf(conversation, t)),
    [busy, conversation, t]
  );

  // Il clic sull'etichetta di un Parlante nell'area apre la sua rinomina.
  const clickText = useCallback(
    (e: MouseEvent<HTMLTextAreaElement>) => {
      const { selectionEnd, selectionStart, value } = e.currentTarget;
      const voce =
        selectionStart === selectionEnd
          ? parlanteAt(value, selectionStart, parlantiList)
          : null;
      if (voce) {
        setRenaming(voce);
      }
    },
    [parlantiList]
  );

  // Il nome vale per l'area, Copia testo, il Bino e il Markdown; nel testo modificato a mano cambiano
  // solo le righe delle sue etichette.
  const rename = useCallback(async (voce: Parlante, nome: string) => {
    setRenaming(null);
    const result = await commands.renameParlante(
      voce.ingresso,
      voce.parlante,
      nome
    );
    if (result.status === "error") {
      setStatus({ error: result.error, phase: "failed" });
      return;
    }
    if (edited.current) {
      setText((current) => relabeled(current, voce, nome));
    }
    setConversation((current) =>
      withNome(current, voce.ingresso, voce.parlante, nome)
    );
  }, []);

  // Il titolo di un Bino con le sue operazioni. `active`: ci lavora la Trascrizione in corso.
  const header = useCallback(
    (path: string, active: boolean) => (
      <BinoHeader
        disabled={active}
        library={library}
        onMove={moveBino}
        onRename={renameBino}
        onReveal={reveal}
        onTrash={requestTrash}
        path={path}
      />
    ),
    [library, moveBino, renameBino, requestTrash, reveal]
  );

  // La Frase di un risultato si seleziona e si porta in vista appena l'area mostra il suo testo.
  // biome-ignore lint/correctness/useExhaustiveDependencies: si riprova a ogni cambio di vista o testo
  useEffect(() => {
    const el = area.current;
    if (
      !(selection && el) ||
      el.value.slice(selection.start, selection.end) !== selection.text
    ) {
      return;
    }
    setSelection(null);
    // Il cursore all'inizio della Frase, che `focus()` porta in vista; poi la Frase selezionata.
    el.blur();
    el.setSelectionRange(selection.start, selection.start);
    el.focus();
    el.setSelectionRange(selection.start, selection.end);
  }, [selection, text, browsed, listOpen]);

  // La sezione Trascrizione segue l'Attività: c'è durante e dopo una Trascrizione, anche dal vivo,
  // o se c'è testo.
  const showTranscription =
    writing || status.phase === "finished" || text !== "";
  const shown = shownText(running, writing, conversation, text, t);

  let mainView: ReactNode;
  if (listOpen) {
    mainView = (
      <AllBini
        bini={biniOf(library.bini, raccolta)}
        onMove={moveBino}
        onOpen={openFromLibrary}
        onTrash={requestTrash}
        raccolte={library.raccolte}
        title={raccoltaLabel(raccolta, t)}
      />
    );
  } else if (browsed) {
    mainView = (
      <BrowsedView
        area={area}
        copied={copied}
        header={header(browsed.path, false)}
        key={browsed.path}
        onCopy={copy}
        text={conversationText(browsed.conversation, t)}
      />
    );
  } else {
    mainView = (
      <>
        <SourceTitle
          header={header}
          onOpen={open}
          running={running}
          source={source}
        />
        <TranscribeBar
          busy={busy}
          cancelling={cancelling}
          canTranscribe={source !== null && !busy}
          language={language}
          languages={languages}
          onCancel={running || completing ? cancel : null}
          onError={failed}
          onLanguage={chooseLanguage}
          onTranscribe={requestTranscription}
          running={running}
        />
        {recording ? (
          <RecordingPanel onPausedChange={setPaused} paused={paused} />
        ) : null}
        {showTranscription ? (
          <section className="flex min-h-0 flex-1 flex-col gap-2">
            <Textarea
              aria-label={t("transcription.text")}
              className="flex-1 resize-none"
              // Un altro Bino riparte dall'inizio del testo.
              key={source}
              onChange={edit}
              onClick={clickText}
              readOnly={writing}
              ref={area}
              value={shown}
            />
            <div className="flex items-start justify-between gap-3">
              <ParlantiBar
                editing={renaming}
                list={parlantiList}
                onEdit={setRenaming}
                onRename={rename}
              />
              <CopyButton
                copied={copied}
                disabled={!copyable(running, text)}
                onCopy={copy}
              />
            </div>
          </section>
        ) : null}
      </>
    );
  }

  return (
    <>
      <div className="flex h-screen" inert={settingsPage !== null}>
        <Sidebar
          actions={
            <>
              <div className="flex gap-2">
                <Button
                  className="flex-1"
                  disabled={busy}
                  onClick={requestRecording}
                  variant="outline"
                >
                  <Circle className="fill-destructive text-destructive" />
                  {t("recording.start")}
                </Button>
                <Button
                  className="flex-1"
                  disabled={busy}
                  onClick={pickFile}
                  variant="outline"
                >
                  <FolderOpen />
                  {t("source.browse")}
                </Button>
              </div>
              <SettingCheckbox
                disabled={busy}
                label={t("recording.live")}
                name="trascrizioneDalVivo"
                onError={failed}
              />
            </>
          }
          activity={activityText(status, elapsedMs, t)}
          list={library}
          onActivity={showActivity}
          onError={setNotice}
          onMoved={moved}
          onOpen={openFromLibrary}
          onRaccolta={chooseRaccolta}
          onShowAll={showAll}
          raccolta={raccolta}
          selected={listOpen ? null : (browsed?.path ?? source)}
        />
        <div className="flex min-w-0 flex-1 flex-col">
          <main className="flex min-h-0 flex-1 flex-col gap-4 p-6">
            {mainView}
          </main>
          <StatusBar notice={notice} status={status} />
        </div>
      </div>
      <AlertDialog onOpenChange={closeConfirm} open={confirmReplace !== null}>
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
      {dialog}
      {settingsPage}
    </>
  );
}

/** Lingua del parlato, Riconosci i parlanti, Trascrivi e, durante l'Attività, Annulla. */
function TranscribeBar({
  busy,
  canTranscribe,
  cancelling,
  language,
  languages,
  onCancel,
  onError,
  onLanguage,
  onTranscribe,
  running,
}: {
  busy: boolean;
  canTranscribe: boolean;
  cancelling: boolean;
  language: string;
  languages: string[];
  /** `null` se non c'è niente da annullare. */
  onCancel: (() => void) | null;
  onError: (error: AppError) => void;
  onLanguage: (e: ChangeEvent<HTMLSelectElement>) => void;
  onTranscribe: () => void;
  running: boolean;
}) {
  const { t } = useTranslation();
  return (
    <section className="flex items-center justify-end gap-3">
      <select
        aria-label={t("speechLanguage.label")}
        className={SELECT}
        disabled={busy}
        onChange={onLanguage}
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
        onError={onError}
      />
      <Button disabled={!canTranscribe} onClick={onTranscribe}>
        {running ? t("transcription.running") : t("transcription.start")}
      </Button>
      {onCancel ? (
        <Button disabled={cancelling} onClick={onCancel} variant="outline">
          {cancelling
            ? t("transcription.cancelling")
            : t("transcription.cancel")}
        </Button>
      ) : null}
    </section>
  );
}

/** Il testo di un Bino consultato durante un'Attività, in sola lettura, con Copia testo. */
function BrowsedView({
  area,
  copied,
  header,
  onCopy,
  text,
}: {
  area: RefObject<HTMLTextAreaElement | null>;
  copied: boolean;
  header: ReactNode;
  onCopy: () => void;
  text: string;
}) {
  const { t } = useTranslation();
  return (
    <>
      {header}
      <section className="flex min-h-0 flex-1 flex-col gap-2">
        <Textarea
          aria-label={t("transcription.text")}
          className="flex-1 resize-none"
          readOnly
          ref={area}
          value={text}
        />
        <CopyButton copied={copied} disabled={false} onCopy={onCopy} />
      </section>
    </>
  );
}

/** Copia testo, che per un attimo dice "Copiato". */
function CopyButton({
  copied,
  disabled,
  onCopy,
}: {
  copied: boolean;
  disabled: boolean;
  onCopy: () => void;
}) {
  const { t } = useTranslation();
  return (
    <Button
      className="ml-auto shrink-0"
      disabled={disabled}
      onClick={onCopy}
      size="sm"
      variant="outline"
    >
      {copied ? t("transcription.copied") : t("transcription.copy")}
    </Button>
  );
}

/**
 * Il titolo della Sorgente: per un Bino `header`, con le sue operazioni (non sul Bino che la
 * Trascrizione in corso sta riscrivendo); per un file audio o video il nome, che un clic apre con il
 * programma associato.
 */
function SourceTitle({
  header,
  onOpen,
  running,
  source,
}: {
  header: (path: string, active: boolean) => ReactNode;
  onOpen: () => void;
  running: boolean;
  source: string | null;
}) {
  const { t } = useTranslation();
  if (source && isBino(source)) {
    return header(source, running);
  }
  return (
    <section className="min-w-0">
      {source ? (
        <button
          className="block max-w-full cursor-pointer truncate rounded-sm text-left font-medium text-lg underline decoration-muted-foreground/40 underline-offset-4 hover:decoration-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          onClick={onOpen}
          title={t("source.open", { path: source })}
          type="button"
        >
          {fileName(source)}
        </button>
      ) : (
        <span className="text-muted-foreground text-sm">
          {t("source.none")}
        </span>
      )}
    </section>
  );
}

/** Una casella che salva subito un'impostazione; un errore va nella status bar. */
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

/**
 * La status bar: il messaggio (o per un po' l'errore di un'operazione sulla Libreria), il link alle
 * Impostazioni quando serve e l'avanzamento.
 */
function StatusBar({
  notice,
  status,
}: {
  notice: AppError | null;
  status: Status;
}) {
  const { t } = useTranslation();
  const message = notice ? errorText(notice, t) : statusText(status, t);
  const failed = notice !== null || status.phase === "failed";
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
