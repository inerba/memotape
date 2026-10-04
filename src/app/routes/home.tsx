import { FolderOpen } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useOutlet } from "react-router";
import {
  type AppError,
  type BinoInfo,
  commands,
  events,
  type LibraryList,
} from "@/bindings";
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
import { AllBini } from "@/features/library/all-bini";
import { BinoActions } from "@/features/library/bino-actions";
import { BinoHeader } from "@/features/library/bino-header";
import {
  biniOf,
  chosenRaccolta,
  raccoltaLabel,
} from "@/features/library/library";
import { Sidebar } from "@/features/library/sidebar";
import { useBinoOperations } from "@/features/library/use-bino-operations";
import { useLibrary } from "@/features/library/use-library";
import { Player, usePlayer } from "@/features/player/player";
import { RecordMenu } from "@/features/recording/record-menu";
import { activityText, afterRecording } from "@/features/recording/recording";
import { RecordingPanel } from "@/features/recording/recording-panel";
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
  EMPTY_CONVERSATION,
  type Parlante,
  type PhraseRef,
  parlantiOf,
  withNome,
  withoutPartials,
  withParlanti,
  withPartial,
  withPhrase,
  withTesto,
} from "@/features/transcription/phrases";
import { TranscribeMenu } from "@/features/transcription/transcribe-menu";
import { TranscriptView } from "@/features/transcription/transcript-view";

const COPIED_MS = 2000;
/** Quanto resta nella status bar un errore di un'operazione sulla Libreria, o un avviso. */
const NOTICE_MS = 6000;

/** Un Bino aperto: le Frasi e le informazioni. */
interface BinoView {
  conversation: Conversation;
  info: BinoInfo | null;
  path: string;
}

/** Il Bino evidenziato nella barra laterale: quello mostrato nell'area principale. */
function selectedBino(
  listOpen: boolean,
  browsed: BinoView | null,
  recording: boolean,
  source: string | null
): string | null {
  if (listOpen) {
    return null;
  }
  return browsed?.path ?? (recording ? null : source);
}

function internalError(e: unknown): AppError {
  return { code: "internal", detail: String(e) };
}

export function HomePage() {
  const { t } = useTranslation();
  const [source, setSource] = useState<string | null>(null);
  // Le Frasi della Sorgente: quelle del Bino aperto, o quelle che arrivano da un'Attività con i
  // Parziali in corso (Nemotron), uno per Ingresso.
  const [conversation, setConversation] =
    useState<Conversation>(EMPTY_CONVERSATION);
  // Le informazioni del Bino aperto come Sorgente.
  const [info, setInfo] = useState<BinoInfo | null>(null);
  // Gli eventi possono arrivare dopo la risposta di `record`: un Parziale tardivo si ignora.
  const acceptPartials = useRef<boolean>(false);
  const { loadError, save, settings } = useSettings();
  // Impostazioni illeggibili all'avvio: la status bar lo dice finché non c'è altro da mostrare.
  const [status, setStatus] = useState<Status>(() =>
    loadError
      ? { error: loadError, phase: "failed" }
      : { phase: "idle", source: null }
  );
  // L'errore di un'operazione su un Bino o sulla Libreria, o un avviso (il Markdown esportato): la
  // status bar lo mostra per un po' sopra la fase, che durante un'Attività non deve cambiare.
  const [notice, setNotice] = useState<AppError | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  // Trascrivi su un Bino aspetta la conferma: il testo e le correzioni si sostituiscono.
  const [confirmTranscribe, setConfirmTranscribe] = useState(false);
  // La Frase di un risultato della ricerca, evidenziata nel Bino aperto.
  const [highlight, setHighlight] = useState<PhraseRef | null>(null);
  // Cambia per riaprire il Bino già aperto dall'inizio del testo.
  const [revision, setRevision] = useState(0);
  // Il Bino del doppio clic in Esplora file, finché non si può aprire.
  const [pendingBino, setPendingBino] = useState<string | null>(null);
  // Il Parlante di cui si sta scrivendo il nome nuovo.
  const [renaming, setRenaming] = useState<Parlante | null>(null);
  // Il Bino aperto dalla barra laterale durante un'Attività.
  const [browsed, setBrowsed] = useState<BinoView | null>(null);
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
  // Impostazioni, aperta sopra questa finestra.
  const settingsPage = useOutlet();
  const navigate = useNavigate();
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

  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = setTimeout(() => setCopied(false), COPIED_MS);
    return () => clearTimeout(timer);
  }, [copied]);

  useEffect(() => {
    if (!(notice || message)) {
      return;
    }
    const timer = setTimeout(() => {
      setNotice(null);
      setMessage(null);
    }, NOTICE_MS);
    return () => clearTimeout(timer);
  }, [notice, message]);

  // Il Bino diventa la Sorgente, con il suo testo, senza ritrascrivere; da un risultato della
  // ricerca con la Frase trovata evidenziata. Restituisce l'errore, se non si apre.
  const loadBino = useCallback(async (path: string, phrase?: PhraseRef) => {
    const result = await commands.openBino(path);
    if (result.status === "error") {
      return result.error;
    }
    const { info: opened, parlanti, phrases } = result.data;
    setSource(path);
    setConversation({ ...EMPTY_CONVERSATION, parlanti, phrases });
    setInfo(opened);
    setHighlight(phrase ?? null);
    setRenaming(null);
    return null;
  }, []);

  const openBino = useCallback(
    async (path: string, phrase?: PhraseRef) => {
      const error = await loadBino(path, phrase);
      setStatus(
        error ? { error, phase: "failed" } : { phase: "idle", source: path }
      );
    },
    [loadBino]
  );

  // Durante un'Attività un Bino della Libreria si consulta accanto, senza toccarla; quello su cui
  // lavora l'Attività riporta alla sua vista.
  const browse = useCallback(
    async (path: string, phrase?: PhraseRef) => {
      setHighlight(phrase ?? null);
      setRenaming(null);
      // Solo Trascrivi lavora sulla Sorgente; durante una Registrazione è un Bino come gli altri.
      if (path === source && running) {
        setBrowsed(null);
        return;
      }
      const result = await commands.openBino(path);
      if (result.status === "error") {
        setNotice(result.error);
        return;
      }
      const { info: opened, parlanti, phrases } = result.data;
      setBrowsed({
        conversation: { ...EMPTY_CONVERSATION, parlanti, phrases },
        info: opened,
        path,
      });
    },
    [running, source]
  );

  // Finita l'Attività torna la sua vista, con il suo esito aperto (il Bino di una Registrazione).
  useEffect(() => {
    if (!busy) {
      setBrowsed(null);
    }
  }, [busy]);

  // Una Sorgente scelta con Apri file o con il doppio clic su un Bino in Esplora file.
  const openPath = useCallback(
    (path: string, phrase?: PhraseRef) => {
      if (isBino(path)) {
        openBino(path, phrase);
        return;
      }
      setSource(path);
      setConversation(EMPTY_CONVERSATION);
      setInfo(null);
      setHighlight(null);
      setStatus({ phase: "idle", source: path });
    },
    [openBino]
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
      if (busy) {
        browse(path, phrase);
      } else if (path === source) {
        // Il Bino già aperto: torna alla Frase trovata, o all'inizio del testo.
        setHighlight(phrase ?? null);
        setRevision((n) => n + 1);
      } else {
        openPath(path, phrase);
      }
    },
    [browse, busy, openPath, source]
  );

  // "Attività in corso" riporta alla sua vista.
  const showActivity = useCallback(() => {
    setBrowsed(null);
    // La Frase trovata era del Bino consultato.
    setHighlight(null);
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
          setConversation(EMPTY_CONVERSATION);
          setInfo(null);
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
  // intanto è disabilitata) e che si chiuda la conferma di Trascrivi. Si apre sulla finestra
  // principale, anche se c'era Impostazioni sopra.
  useEffect(() => {
    if (pendingBino && !busy && !confirmTranscribe) {
      setPendingBino(null);
      setListOpen(false);
      navigate("/");
      openPath(pendingBino);
    }
  }, [busy, confirmTranscribe, navigate, openPath, pendingBino]);

  const open = useCallback(async () => {
    if (!source) {
      return;
    }
    const result = await commands.openSource(source);
    if (result.status === "error") {
      setStatus({ error: result.error, phase: "failed" });
    }
  }, [source]);

  // Un file diventa un Bino nella Raccolta scelta; un Bino si ritrascrive. Finita (o annullata) la
  // Trascrizione, il Bino si rilegge dal disco.
  const transcribe = useCallback(async () => {
    if (!source) {
      return;
    }
    setConversation(EMPTY_CONVERSATION);
    setHighlight(null);
    setRenaming(null);
    setCancelling(false);
    setStatus({ percent: null, phase: "transcribing" });
    try {
      const result = await commands.transcribe(source, raccolta);
      let opened: string | null = isBino(source) ? source : null;
      if (result.status === "ok" && result.data.outcome === "saved") {
        opened = result.data.path;
      }
      const error = opened ? await loadBino(opened) : null;
      setStatus(
        error ? { error, phase: "failed" } : afterTranscription(result)
      );
    } catch (e) {
      // `typedError` rilancia gli `Error` di IPC: la Trascrizione non deve restare "in corso".
      setStatus({ error: internalError(e), phase: "failed" });
    }
  }, [loadBino, raccolta, source]);

  // Il Bino della Registrazione diventa la Sorgente; se non è partita torna quella di prima.
  const record = useCallback(async () => {
    const before = source;
    setConversation(EMPTY_CONVERSATION);
    setInfo(null);
    setHighlight(null);
    setRenaming(null);
    acceptPartials.current = settings.trascrizioneDalVivo ?? false;
    setCancelling(false);
    setElapsedMs(0);
    setListOpen(false);
    setStatus({ paused: false, phase: "recording" });
    try {
      const after = afterRecording(
        await commands.record(t("recording.prefix"), raccolta)
      );
      acceptPartials.current = false;
      setConversation(withoutPartials);
      const opened = after.source ?? before;
      let error: AppError | null = null;
      if (opened && isBino(opened)) {
        error = await loadBino(opened);
      } else if (after.source) {
        // Il Bino non si è scritto: la Sorgente è l'Ogg, con il testo dal vivo.
        setSource(after.source);
      }
      setStatus(
        error && after.status.phase !== "failed"
          ? { error, phase: "failed" }
          : after.status
      );
    } catch (e) {
      setStatus({ error: internalError(e), phase: "failed" });
    } finally {
      acceptPartials.current = false;
    }
  }, [loadBino, raccolta, settings.trascrizioneDalVivo, source, t]);

  const setPaused = useCallback((value: boolean) => {
    setStatus((current) =>
      current.phase === "recording" ? { ...current, paused: value } : current
    );
  }, []);

  // Ritrascrivere un Bino ne sostituisce il testo, correzioni comprese: prima si conferma.
  const requestTranscription = useCallback(() => {
    if (source && isBino(source)) {
      setConfirmTranscribe(true);
    } else {
      transcribe();
    }
  }, [source, transcribe]);

  const closeConfirm = useCallback((opened: boolean) => {
    if (!opened) {
      setConfirmTranscribe(false);
    }
  }, []);

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

  // Il documento del Bino `path`, o senza Bino quello della Trascrizione in corso o appena finita,
  // in testo semplice o Markdown secondo le impostazioni.
  const copy = useCallback(async (path: string | null) => {
    try {
      let text: string | null;
      if (path) {
        const result = await commands.binoText(path);
        if (result.status === "error") {
          setNotice(result.error);
          return;
        }
        text = result.data;
      } else {
        text = await commands.transcriptText();
      }
      await navigator.clipboard.writeText(text ?? "");
      setCopied(true);
    } catch (e) {
      setNotice(internalError(e));
    }
  }, []);

  const exported = useCallback(
    (path: string) => setMessage(t("transcription.exported", { path })),
    [t]
  );

  // Una correzione o un nome salvati nel Bino `path` valgono per la sua vista.
  const updateView = useCallback(
    (path: string, change: (c: Conversation) => Conversation) => {
      setBrowsed((current) =>
        current?.path === path
          ? { ...current, conversation: change(current.conversation) }
          : current
      );
      if (path === source) {
        setConversation(change);
      }
    },
    [source]
  );

  const rename = useCallback(
    async (path: string, voce: Parlante, nome: string) => {
      setRenaming(null);
      const result = await commands.renameParlante(
        path,
        voce.ingresso,
        voce.parlante,
        nome
      );
      if (result.status === "error") {
        setNotice(result.error);
        return;
      }
      updateView(path, (c) => withNome(c, voce.ingresso, voce.parlante, nome));
    },
    [updateView]
  );

  // Una correzione non salvata resta scritta nella Frase, con l'errore nella status bar.
  const edit = useCallback(
    async (path: string, phrase: PhraseRef, text: string) => {
      const result = await commands.editFrase(
        path,
        phrase.ingresso,
        phrase.phraseId,
        text
      );
      if (result.status === "error") {
        setNotice(result.error);
        return false;
      }
      updateView(path, (c) => withTesto(c, phrase, text));
      return true;
    },
    [updateView]
  );

  const copyActivity = useCallback(() => copy(null), [copy]);

  // La vista di un Bino; `own`: è la Sorgente, che Trascrivi trascrive.
  const binoPane = (view: BinoView, own: boolean) => (
    <BinoPane
      busy={busy}
      cancelling={cancelling}
      copied={copied}
      // Non ci lavora l'Attività in corso.
      editable={!(own && busy)}
      highlight={highlight}
      key={`${view.path}:${revision}`}
      library={library}
      onCancel={cancel}
      onCopy={copy}
      onEdit={edit}
      onError={setNotice}
      onExported={exported}
      onFailed={failed}
      onMove={moveBino}
      onRenameParlante={rename}
      onRenameTitle={renameBino}
      onRenaming={setRenaming}
      onReveal={reveal}
      onTranscribe={requestTranscription}
      onTrash={requestTrash}
      own={own}
      recording={recording}
      renaming={renaming}
      running={running}
      view={view}
    />
  );

  let mainView: React.ReactNode;
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
    mainView = binoPane(browsed, false);
  } else if (recording || completing) {
    // La Registrazione, con il testo dal vivo.
    mainView = (
      <>
        {recording ? (
          <RecordingPanel onPausedChange={setPaused} paused={paused} />
        ) : null}
        <div className="flex items-center gap-2">
          <CopyButton
            copied={copied}
            disabled={conversation.phrases.length === 0}
            onCopy={copyActivity}
          />
        </div>
        <TranscriptView conversation={conversation} parlanti={[]} />
      </>
    );
  } else if (source && isBino(source)) {
    mainView = binoPane({ conversation, info, path: source }, true);
  } else {
    // Un file audio o video: il nome e Trascrivi; dopo una Trascrizione annullata, le sue Frasi.
    mainView = (
      <>
        <SourceTitle onOpen={open} source={source} />
        <div className="flex items-center gap-2">
          <TranscribeMenu
            busy={busy}
            cancelling={cancelling}
            canTranscribe={source !== null && !busy}
            onCancel={running ? cancel : null}
            onError={failed}
            onTranscribe={requestTranscription}
            running={running}
          />
          <CopyButton
            copied={copied}
            disabled={running || conversation.phrases.length === 0}
            onCopy={copyActivity}
          />
        </div>
        {running ? null : (
          <TranscriptView conversation={conversation} parlanti={[]} />
        )}
      </>
    );
  }

  return (
    <>
      <div className="flex h-screen" inert={settingsPage !== null}>
        <Sidebar
          actions={
            <div className="flex gap-2">
              <RecordMenu disabled={busy} onError={failed} onRecord={record} />
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
          selected={selectedBino(
            listOpen,
            browsed,
            recording || completing,
            source
          )}
        />
        <div className="flex min-w-0 flex-1 flex-col">
          <main className="flex min-h-0 flex-1 flex-col gap-4 p-6">
            {mainView}
          </main>
          <StatusBar message={message} notice={notice} status={status} />
        </div>
      </div>
      <AlertDialog onOpenChange={closeConfirm} open={confirmTranscribe}>
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
      {dialog}
      {settingsPage}
    </>
  );
}

/**
 * La vista di un Bino: titolo, informazioni, azioni, Parlanti, trascrizione a turni e player.
 * `editable`: non ci lavora l'Attività in corso; `own`: è la Sorgente, che Trascrivi trascrive;
 * `recording`: il player è disabilitato.
 */
function BinoPane({
  busy,
  cancelling,
  copied,
  editable,
  highlight,
  library,
  onCancel,
  onCopy,
  onEdit,
  onError,
  onExported,
  onFailed,
  onMove,
  onRenameParlante,
  onRenameTitle,
  onRenaming,
  onReveal,
  onTranscribe,
  onTrash,
  own,
  recording,
  renaming,
  running,
  view: { conversation, info, path },
}: {
  busy: boolean;
  cancelling: boolean;
  copied: boolean;
  editable: boolean;
  highlight: PhraseRef | null;
  library: LibraryList;
  onCancel: () => void;
  onCopy: (path: string) => void;
  onEdit: (path: string, phrase: PhraseRef, text: string) => Promise<boolean>;
  onError: (error: AppError) => void;
  onExported: (path: string) => void;
  onFailed: (error: AppError) => void;
  onMove: (path: string, raccolta: string) => void;
  onRenameParlante: (path: string, voce: Parlante, nome: string) => void;
  onRenameTitle: (path: string, titolo: string) => void;
  onRenaming: (voce: Parlante | null) => void;
  onReveal: (path: string) => void;
  onTranscribe: () => void;
  onTrash: (bino: { path: string; titolo: string }) => void;
  own: boolean;
  recording: boolean;
  renaming: Parlante | null;
  running: boolean;
  view: BinoView;
}) {
  const { t } = useTranslation();
  const parlanti = editable ? parlantiOf(conversation, t) : [];
  const transcribing = own && running;
  const player = usePlayer(info?.durataMs ?? 0);
  // La Frase di un risultato della ricerca porta lì il player, senza avviarlo.
  const phrases = useRef(conversation.phrases);
  useEffect(() => {
    phrases.current = conversation.phrases;
  }, [conversation.phrases]);
  const { audio, move } = player;
  useEffect(() => {
    const found = phrases.current.find(
      (p) =>
        p.ingresso === highlight?.ingresso && p.phraseId === highlight.phraseId
    );
    if (found) {
      audio.current?.pause();
      move(found.inizioMs, "jump");
    }
  }, [audio, highlight, move]);
  const copy = useCallback(() => onCopy(path), [onCopy, path]);
  const rename = useCallback(
    (voce: Parlante, nome: string) => onRenameParlante(path, voce, nome),
    [onRenameParlante, path]
  );
  const edit = useCallback(
    (phrase: PhraseRef, text: string) => onEdit(path, phrase, text),
    [onEdit, path]
  );
  return (
    <>
      <BinoHeader
        disabled={!editable}
        info={info}
        library={library}
        onRename={onRenameTitle}
        path={path}
      />
      <div className="flex flex-wrap items-center gap-2">
        <TranscribeMenu
          busy={busy}
          cancelling={cancelling}
          canTranscribe={own && !busy}
          onCancel={transcribing ? onCancel : null}
          onError={onFailed}
          onTranscribe={onTranscribe}
          running={transcribing}
        />
        <CopyButton copied={copied} disabled={transcribing} onCopy={copy} />
        <BinoActions
          disabled={!editable}
          library={library}
          onError={onError}
          onExported={onExported}
          onMove={onMove}
          onReveal={onReveal}
          onTrash={onTrash}
          path={path}
        />
      </div>
      <ParlantiBar
        editing={renaming}
        list={parlanti}
        onEdit={onRenaming}
        onRename={rename}
      />
      {transcribing ? null : (
        <>
          <TranscriptView
            conversation={conversation}
            highlight={highlight}
            onEdit={editable ? edit : undefined}
            onRename={onRenaming}
            parlanti={parlanti}
            // Durante una Registrazione niente salti né evidenziazione: il player è disabilitato.
            player={recording ? undefined : player}
          />
          <Player disabled={recording} path={path} player={player} />
        </>
      )}
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
    <Button disabled={disabled} onClick={onCopy} variant="outline">
      {copied ? t("transcription.copied") : t("transcription.copy")}
    </Button>
  );
}

/** Il nome del file audio o video, che un clic apre con il programma associato. */
function SourceTitle({
  onOpen,
  source,
}: {
  onOpen: () => void;
  source: string | null;
}) {
  const { t } = useTranslation();
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

/**
 * La status bar: il messaggio (o per un po' l'errore di un'operazione su un Bino o un avviso), il
 * link alle Impostazioni quando serve e l'avanzamento.
 */
function StatusBar({
  message,
  notice,
  status,
}: {
  message: string | null;
  notice: AppError | null;
  status: Status;
}) {
  const { t } = useTranslation();
  let text = message ?? statusText(status, t);
  if (notice) {
    text = errorText(notice, t);
  }
  const failed = notice !== null || (!message && status.phase === "failed");
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
        title={text}
      >
        {text}
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
