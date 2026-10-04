import type { TFunction } from "i18next";
import { Check, CircleAlert, Copy, FileUp, Mic, X } from "lucide-react";
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
import { WindowControls } from "@/components/window-controls";
import { AllBini } from "@/features/library/all-bini";
import { BinoMenu } from "@/features/library/bino-actions";
import {
  BinoHeader,
  DocumentHeader,
  entryOf,
  titleOf,
} from "@/features/library/bino-header";
import { chosenRaccolta, raccoltaLabel } from "@/features/library/library";
import { Sidebar } from "@/features/library/sidebar";
import { useBinoOperations } from "@/features/library/use-bino-operations";
import { useLibrary } from "@/features/library/use-library";
import {
  keepFocus,
  Player,
  type PlayerState,
  usePlayer,
} from "@/features/player/player";
import { RecordMenu } from "@/features/recording/record-menu";
import {
  activitySummary,
  afterRecording,
} from "@/features/recording/recording";
import { RecordingPanel } from "@/features/recording/recording-panel";
import { useSettings } from "@/features/settings/settings-context";
import { fileName, isBino, movedPath } from "@/features/source/file-name";
import {
  afterTranscription,
  type Banner,
  bannerOf,
  errorText,
  progressPercent,
  type Status,
  statusText,
  withDiarizing,
  withLiveError,
  withMovedSource,
  withProgress,
} from "@/features/status/status";
import { ParlantiTab } from "@/features/transcription/parlanti-tab";
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
/** Quanto resta l'avviso di un esito, o di un errore di un'operazione sulla Libreria. */
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
  // L'esito o l'errore chiuso dall'utente: l'avviso torna con la fase successiva.
  const [dismissed, setDismissed] = useState<Status | null>(null);
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

  // La data e l'ora nuove del Bino `path`, nell'ora locale del campo.
  const changeCreato = useCallback(
    async (path: string, local: string) => {
      const result = await commands.setCreato(path, local);
      if (result.status === "error") {
        setNotice(result.error);
        return;
      }
      const creato = result.data;
      setBrowsed((current) =>
        current?.path === path && current.info
          ? { ...current, info: { ...current.info, creato } }
          : current
      );
      if (path === source) {
        setInfo((current) => current && { ...current, creato });
      }
    },
    [source]
  );

  const copyActivity = useCallback(() => copy(null), [copy]);

  // La Raccolta di un Bino dal suo percorso nella barra in alto: apre la Libreria lì.
  const openRaccolta = useCallback(
    (value: string) => {
      chooseRaccolta(value);
      setListOpen(true);
    },
    [chooseRaccolta]
  );

  // L'avviso in cima: l'errore di un'operazione, un messaggio o l'esito dell'Attività.
  const statusBanner = dismissed === status ? null : bannerOf(status, t);
  const banner = shownBanner(notice, message, statusBanner, t);
  const dismiss = useCallback(() => {
    setNotice(null);
    setMessage(null);
    setDismissed(status);
  }, [status]);
  // Gli esiti spariscono da soli dopo un po'; gli errori restano finché non si chiudono.
  useEffect(() => {
    if (statusBanner?.tone !== "info") {
      return;
    }
    const timer = setTimeout(() => setDismissed(status), NOTICE_MS);
    return () => clearTimeout(timer);
  }, [status, statusBanner?.tone]);

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
      onCreato={changeCreato}
      onEdit={edit}
      onError={setNotice}
      onExported={exported}
      onMove={moveBino}
      onRaccolta={openRaccolta}
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
      status={status}
      view={view}
    />
  );

  let mainView: React.ReactNode;
  if (listOpen) {
    mainView = (
      <>
        <TopBar crumbs={<Crumb current>{t("library.title")}</Crumb>} />
        <AllBini
          list={library}
          onError={setNotice}
          onMove={moveBino}
          onMoved={moved}
          onOpen={openFromLibrary}
          onRaccolta={chooseRaccolta}
          onTrash={requestTrash}
          raccolta={raccolta}
        />
      </>
    );
  } else if (browsed) {
    mainView = binoPane(browsed, false);
  } else if (recording || completing) {
    mainView = (
      <LiveView
        cancelling={cancelling}
        conversation={conversation}
        copied={copied}
        onCancel={cancel}
        onCopy={copyActivity}
        onPausedChange={setPaused}
        paused={paused}
        status={status}
      />
    );
  } else if (source && isBino(source)) {
    mainView = binoPane({ conversation, info, path: source }, true);
  } else if (source) {
    mainView = (
      <FileView
        busy={busy}
        cancelling={cancelling}
        conversation={conversation}
        copied={copied}
        onCancel={cancel}
        onCopy={copyActivity}
        onError={failed}
        onOpen={open}
        onTranscribe={requestTranscription}
        running={running}
        source={source}
        status={status}
      />
    );
  } else {
    mainView = <Welcome busy={busy} onImport={pickFile} onRecord={record} />;
  }

  return (
    <>
      <div className="flex h-screen" inert={settingsPage !== null}>
        <Sidebar
          activity={activitySummary(status, elapsedMs, t)}
          busy={busy}
          list={library}
          onActivity={showActivity}
          onError={setNotice}
          onImport={pickFile}
          onOpen={openFromLibrary}
          onShowAll={showAll}
          record={
            <RecordMenu disabled={busy} onError={failed} onRecord={record} />
          }
          selected={selectedBino(
            listOpen,
            browsed,
            recording || completing,
            source
          )}
          showingAll={listOpen}
        />
        <main className="relative flex min-w-0 flex-1 flex-col">
          {mainView}
          {banner ? (
            <BannerView banner={banner} key={banner.text} onDismiss={dismiss} />
          ) : null}
        </main>
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
      <WindowControls />
    </>
  );
}

/** L'avviso da mostrare: l'errore di un'operazione, poi un messaggio, poi quello della fase. */
function shownBanner(
  notice: AppError | null,
  message: string | null,
  status: Banner | null,
  t: TFunction
): Banner | null {
  if (notice) {
    return { settings: false, text: errorText(notice, t), tone: "error" };
  }
  return message ? { settings: false, text: message, tone: "info" } : status;
}

/** La Registrazione in corso, o il suo completamento dopo Stop, con il testo dal vivo. */
function LiveView({
  cancelling,
  conversation,
  copied,
  onCancel,
  onCopy,
  onPausedChange,
  paused,
  status,
}: {
  cancelling: boolean;
  conversation: Conversation;
  copied: boolean;
  onCancel: () => void;
  onCopy: () => void;
  onPausedChange: (paused: boolean) => void;
  paused: boolean;
  status: Status;
}) {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const live = settings.trascrizioneDalVivo ?? false;
  const recording = status.phase === "recording";
  return (
    <>
      <TopBar
        actions={
          <CopyButton
            copied={copied}
            disabled={conversation.phrases.length === 0}
            onCopy={onCopy}
          />
        }
        crumbs={
          <Crumb current>
            {recording ? t("live.recording") : t("status.completing")}
          </Crumb>
        }
      />
      <TranscriptView
        conversation={conversation}
        empty={
          <p className="max-w-[34rem] py-6 text-muted-foreground leading-relaxed">
            {live ? t("live.waiting") : t("live.off")}
          </p>
        }
        header={
          <DocumentHeader
            meta={live ? t("live.on") : null}
            title={t("live.title")}
          />
        }
        parlanti={[]}
      />
      <Dock>
        {recording ? (
          <RecordingPanel onPausedChange={onPausedChange} paused={paused} />
        ) : (
          <ProgressDock
            cancelling={cancelling}
            onCancel={onCancel}
            status={status}
          />
        )}
      </Dock>
    </>
  );
}

/**
 * Un file audio o video: il nome, che un clic apre, e Trascrivi; dopo una Trascrizione annullata,
 * le sue Frasi.
 */
function FileView({
  busy,
  cancelling,
  conversation,
  copied,
  onCancel,
  onCopy,
  onError,
  onOpen,
  onTranscribe,
  running,
  source,
  status,
}: {
  busy: boolean;
  cancelling: boolean;
  conversation: Conversation;
  copied: boolean;
  onCancel: () => void;
  onCopy: () => void;
  onError: (error: AppError) => void;
  onOpen: () => void;
  onTranscribe: () => void;
  running: boolean;
  source: string;
  status: Status;
}) {
  const { t } = useTranslation();
  return (
    <>
      <TopBar
        actions={
          conversation.phrases.length > 0 ? (
            <CopyButton copied={copied} disabled={running} onCopy={onCopy} />
          ) : null
        }
        crumbs={
          <Crumb current title={source}>
            {fileName(source)}
          </Crumb>
        }
      />
      <TranscriptView
        conversation={conversation}
        header={
          <>
            <DocumentHeader
              meta={running ? null : t("source.ready")}
              title={<SourceTitle onOpen={onOpen} source={source} />}
            />
            {running ? null : (
              <div className="pb-8">
                <TranscribeMenu
                  busy={busy}
                  canTranscribe={!busy}
                  onError={onError}
                  onTranscribe={onTranscribe}
                />
              </div>
            )}
          </>
        }
        parlanti={[]}
      />
      {running ? (
        <Dock>
          <ProgressDock
            cancelling={cancelling}
            onCancel={onCancel}
            status={status}
          />
        </Dock>
      ) : null}
    </>
  );
}

/**
 * La barra in alto del pannello centrale: si trascina come la barra del titolo di Windows. A
 * sinistra dove si è, a destra le azioni, sotto i pulsanti della finestra.
 */
function TopBar({
  actions,
  crumbs,
}: {
  actions?: React.ReactNode;
  crumbs?: React.ReactNode;
}) {
  const { t } = useTranslation();
  return (
    <header
      className="flex h-20 shrink-0 items-end gap-4 border-b px-8 pb-3"
      data-tauri-drag-region
    >
      <nav
        aria-label={t("app.whereAmI")}
        className="flex min-w-0 flex-1 items-center gap-2 pb-1.5 text-muted-foreground text-sm"
        data-tauri-drag-region
      >
        {crumbs}
      </nav>
      {actions ? (
        <div className="flex items-center gap-1.5">{actions}</div>
      ) : null}
    </header>
  );
}

/** Un passo del percorso nella barra in alto: il corrente, o un link (`onClick`). */
function Crumb({
  children,
  current = false,
  onClick,
  title,
}: {
  children: React.ReactNode;
  current?: boolean;
  onClick?: () => void;
  /** Il testo intero, per esempio il percorso del file. */
  title?: string;
}) {
  if (onClick) {
    return (
      <>
        <button
          className="shrink-0 truncate rounded-sm transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
          onClick={onClick}
          type="button"
        >
          {children}
        </button>
        <span aria-hidden className="text-muted-foreground/60">
          /
        </span>
      </>
    );
  }
  return (
    <span
      aria-current={current ? "page" : undefined}
      className="truncate text-foreground"
      title={title}
    >
      {children}
    </span>
  );
}

/** Il posto in fondo al pannello centrale per il player, la Registrazione o l'avanzamento. */
function Dock({ children }: { children: React.ReactNode }) {
  return <div className="shrink-0 px-8 pt-1 pb-5">{children}</div>;
}

/** L'avanzamento di una Trascrizione o del suo completamento dopo Stop, con Annulla. */
function ProgressDock({
  cancelling,
  onCancel,
  status,
}: {
  cancelling: boolean;
  onCancel: () => void;
  status: Status;
}) {
  const { t } = useTranslation();
  const percent = progressPercent(status);
  return (
    <section
      aria-live="polite"
      className="mx-auto flex w-full max-w-[52rem] items-center gap-5 rounded-2xl border bg-card px-5 py-3.5 shadow-float"
    >
      <div className="flex min-w-0 flex-1 flex-col gap-2">
        <span className="truncate text-sm tabular-nums">
          {statusText(status, t)}
        </span>
        <span
          aria-label={t("status.progress")}
          aria-valuemax={100}
          aria-valuemin={0}
          aria-valuenow={percent ?? undefined}
          className="relative h-1.5 overflow-hidden rounded-full bg-play-soft"
          role="progressbar"
        >
          {percent === null ? (
            <span className="absolute inset-y-0 w-1/3 animate-[indeterminate_1.4s_ease-in-out_infinite] rounded-full bg-play" />
          ) : (
            <span
              className="absolute inset-y-0 left-0 rounded-full bg-play transition-[width] duration-300"
              style={{ width: `${percent}%` }}
            />
          )}
        </span>
      </div>
      <Button
        className="h-10"
        disabled={cancelling}
        onClick={onCancel}
        variant="outline"
      >
        {cancelling ? t("transcription.cancelling") : t("transcription.cancel")}
      </Button>
    </section>
  );
}

/** L'avviso sospeso sotto la barra in alto, con il link alle Impostazioni quando serve. */
function BannerView({
  banner,
  onDismiss,
}: {
  banner: Banner;
  onDismiss: () => void;
}) {
  const { t } = useTranslation();
  const error = banner.tone === "error";
  return (
    <div
      className={`motion-safe:fade-in motion-safe:slide-in-from-top-2 absolute top-24 left-1/2 z-20 flex w-[min(40rem,calc(100%-4rem))] -translate-x-1/2 items-start gap-3 rounded-xl border bg-card px-4 py-3 text-sm shadow-float motion-safe:animate-in ${
        error ? "border-destructive/35" : ""
      }`}
      role={error ? "alert" : "status"}
    >
      {error ? (
        <CircleAlert className="mt-0.5 size-4 shrink-0 text-destructive" />
      ) : (
        <Check className="mt-0.5 size-4 shrink-0 text-play" />
      )}
      <span className="min-w-0 flex-1 break-words leading-relaxed">
        {banner.text}
        {banner.settings ? (
          <>
            {" "}
            <Link
              className="whitespace-nowrap font-medium underline underline-offset-4"
              to="/settings"
            >
              {t("status.openModels")}
            </Link>
          </>
        ) : null}
      </span>
      <button
        aria-label={t("app.dismiss")}
        className="-m-1 flex size-6 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
        onClick={onDismiss}
        title={t("app.dismiss")}
        type="button"
      >
        <X className="size-4" />
      </button>
    </div>
  );
}

/** Senza Sorgente: le due strade, e la promessa che tutto resta sul PC. */
function Welcome({
  busy,
  onImport,
  onRecord,
}: {
  busy: boolean;
  onImport: () => void;
  onRecord: () => void;
}) {
  const { t } = useTranslation();
  return (
    <>
      <div className="h-20 shrink-0 border-b" data-tauri-drag-region />
      {/* Sulla stessa colonna e alla stessa altezza del titolo di un Bino: lo stesso foglio, vuoto. */}
      <div className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
        <section className="mx-auto flex w-full max-w-[46rem] flex-col items-start px-10 pt-12">
          <h1 className="max-w-[20ch] text-balance font-display font-medium text-[2.75rem] leading-[1.1] tracking-[-0.015em]">
            {t("welcome.title")}
          </h1>
          <p className="mt-4 max-w-[38rem] text-[1.0625rem] text-muted-foreground leading-relaxed">
            {t("welcome.description")}
          </p>
          <div className="mt-8 flex items-center gap-3">
            <Button className="h-10 px-4" disabled={busy} onClick={onRecord}>
              <Mic />
              {t("sidebar.newRecording")}
            </Button>
            <Button
              className="h-10"
              disabled={busy}
              onClick={onImport}
              variant="outline"
            >
              <FileUp />
              {t("sidebar.importFile")}
            </Button>
          </div>
        </section>
      </div>
    </>
  );
}

/**
 * La vista di un Bino: la barra in alto con il percorso, Copia testo e "…"; il documento con
 * testata, schede Trascrizione e Parlanti e Segui l'audio; in fondo il player, o l'avanzamento
 * mentre lo si trascrive. `editable`: non ci lavora l'Attività in corso; `own`: è la Sorgente, che
 * Trascrivi trascrive; `recording`: il player è disabilitato.
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
  onCreato,
  onEdit,
  onError,
  onExported,
  onMove,
  onRaccolta,
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
  status,
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
  onCreato: (path: string, local: string) => void;
  onEdit: (path: string, phrase: PhraseRef, text: string) => Promise<boolean>;
  onError: (error: AppError) => void;
  onExported: (path: string) => void;
  onMove: (path: string, raccolta: string) => void;
  onRaccolta: (raccolta: string) => void;
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
  status: Status;
  view: BinoView;
}) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<"transcript" | "parlanti">("transcript");
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
      setTab("transcript");
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
  const entry = entryOf(library, path);
  const raccolta = entry ? (entry.raccolta ?? "") : null;
  const toRaccolta = useCallback(
    () => raccolta !== null && onRaccolta(raccolta),
    [onRaccolta, raccolta]
  );
  // Durante una Registrazione niente salti né evidenziazione: il player è disabilitato.
  const listening = transcribing || recording ? undefined : player;
  const hasText = conversation.phrases.length > 0;

  const header = (
    <>
      <BinoHeader
        disabled={!editable}
        info={info}
        library={library}
        onCreato={onCreato}
        onRename={onRenameTitle}
        parlanti={parlantiOf(conversation, t).length}
        path={path}
      />
      {hasText ? (
        <DocumentToolbar
          follow={listening}
          onTab={setTab}
          parlanti={parlanti.length}
          tab={tab}
        />
      ) : null}
    </>
  );

  const empty = (
    <BinoEmpty
      busy={busy}
      onError={onError}
      onTranscribe={own ? onTranscribe : undefined}
      transcribing={transcribing}
    />
  );

  return (
    <>
      <TopBar
        actions={
          <>
            {hasText ? (
              <CopyButton
                copied={copied}
                disabled={transcribing}
                onCopy={copy}
              />
            ) : null}
            <BinoMenu
              disabled={!editable}
              library={library}
              onError={onError}
              onExported={onExported}
              onMove={onMove}
              onReveal={onReveal}
              onTranscribe={own && !busy && hasText ? onTranscribe : undefined}
              onTrash={onTrash}
              path={path}
            />
          </>
        }
        crumbs={
          <>
            {raccolta === null ? null : (
              <Crumb onClick={toRaccolta}>{raccoltaLabel(raccolta, t)}</Crumb>
            )}
            <Crumb current title={path}>
              {titleOf(library, path)}
            </Crumb>
          </>
        }
      />
      {tab === "parlanti" && hasText ? (
        <section className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
          <div className="mx-auto flex w-full max-w-[46rem] flex-col px-10 pb-12">
            {header}
            <ParlantiTab
              conversation={conversation}
              list={parlanti}
              onPlay={listening?.playFrom}
              onRename={rename}
              onRenaming={onRenaming}
              renaming={renaming}
            />
          </div>
        </section>
      ) : (
        <TranscriptView
          conversation={conversation}
          empty={empty}
          header={header}
          highlight={highlight}
          onEdit={editable ? edit : undefined}
          onRename={rename}
          onRenaming={onRenaming}
          parlanti={parlanti}
          player={listening}
          renaming={renaming}
        />
      )}
      <Dock>
        {transcribing ? (
          <ProgressDock
            cancelling={cancelling}
            onCancel={onCancel}
            status={status}
          />
        ) : (
          <Player
            disabled={recording}
            label={
              info?.origine
                ? t("player.file", { name: info.origine })
                : t("player.recording")
            }
            path={path}
            player={player}
          />
        )}
      </Dock>
    </>
  );
}

/** Il documento di un Bino senza Frasi: in Trascrizione, o da trascrivere (se `onTranscribe`). */
function BinoEmpty({
  busy,
  onError,
  onTranscribe,
  transcribing,
}: {
  busy: boolean;
  onError: (error: AppError) => void;
  onTranscribe?: () => void;
  transcribing: boolean;
}) {
  const { t } = useTranslation();
  if (transcribing) {
    return (
      <p className="py-6 text-muted-foreground">{t("transcription.running")}</p>
    );
  }
  return (
    <div className="flex flex-col items-start gap-5 py-6">
      <div className="flex flex-col gap-1.5">
        <p className="font-medium">{t("bino.noText")}</p>
        <p className="max-w-[34rem] text-muted-foreground text-sm leading-relaxed">
          {t("bino.noTextDescription")}
        </p>
      </div>
      {onTranscribe ? (
        <TranscribeMenu
          busy={busy}
          canTranscribe={!busy}
          onError={onError}
          onTranscribe={onTranscribe}
        />
      ) : null}
    </div>
  );
}

/** Le schede Trascrizione e Parlanti e, con il player, l'interruttore Segui l'audio. */
function DocumentToolbar({
  follow,
  onTab,
  parlanti,
  tab,
}: {
  follow?: PlayerState;
  onTab: (tab: "transcript" | "parlanti") => void;
  parlanti: number;
  tab: "transcript" | "parlanti";
}) {
  const { t } = useTranslation();
  const toTranscript = useCallback(() => onTab("transcript"), [onTab]);
  const toParlanti = useCallback(() => onTab("parlanti"), [onTab]);
  const following = follow?.follow === "following";
  const onFollow = follow?.onFollow;
  const toggleFollow = useCallback(
    () => onFollow?.(following ? "scroll" : "follow"),
    [following, onFollow]
  );
  return (
    <div className="sticky top-0 z-10 mb-3 flex items-center gap-6 border-b bg-background">
      <div aria-label={t("tabs.label")} className="flex gap-6" role="tablist">
        <TabButton onClick={toTranscript} selected={tab === "transcript"}>
          {t("tabs.transcript")}
        </TabButton>
        <TabButton onClick={toParlanti} selected={tab === "parlanti"}>
          {t("tabs.parlanti")}
          {parlanti > 0 ? (
            <span className="ml-1.5 text-muted-foreground tabular-nums">
              {parlanti}
            </span>
          ) : null}
        </TabButton>
      </div>
      <span className="flex-1" />
      {follow && tab === "transcript" ? (
        <button
          aria-checked={following}
          className="group flex items-center gap-2.5 rounded-md py-1 text-sm focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
          onClick={toggleFollow}
          onPointerDown={keepFocus}
          role="switch"
          type="button"
        >
          <span className="relative h-5 w-9 rounded-full bg-input transition-colors duration-200 group-aria-checked:bg-play">
            <span className="absolute top-0.5 left-0.5 size-4 rounded-full bg-card shadow-sm transition-transform duration-200 ease-out group-aria-checked:translate-x-4" />
          </span>
          {t("player.follow")}
        </button>
      ) : null}
    </div>
  );
}

function TabButton({
  children,
  onClick,
  selected,
}: {
  children: React.ReactNode;
  onClick: () => void;
  selected: boolean;
}) {
  return (
    <button
      aria-selected={selected}
      className="relative h-11 font-medium text-muted-foreground text-sm transition-colors after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:rounded-full hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-selected:text-foreground aria-selected:after:bg-foreground"
      onClick={onClick}
      role="tab"
      type="button"
    >
      {children}
    </button>
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
      className="h-8 gap-2 bg-card"
      disabled={disabled}
      onClick={onCopy}
      variant="outline"
    >
      {copied ? <Check className="text-play" /> : <Copy />}
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
  source: string;
}) {
  const { t } = useTranslation();
  return (
    <button
      className="block max-w-full cursor-pointer text-balance rounded-md text-left decoration-2 decoration-muted-foreground/30 underline-offset-8 hover:underline focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
      onClick={onOpen}
      title={t("source.open", { path: source })}
      type="button"
    >
      {fileName(source)}
    </button>
  );
}
