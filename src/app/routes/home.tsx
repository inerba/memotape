import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { TFunction } from "i18next";
import { Check, CircleAlert, Copy, FileUp, Mic, X } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useOutlet } from "react-router";
import {
  type AppError,
  commands,
  events,
  type LibraryList,
  type TapeInfo,
  type UpdateInfo,
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
import { AllTapes } from "@/features/library/all-tapes";
import { chosenRaccolta, raccoltaLabel } from "@/features/library/library";
import { Sidebar } from "@/features/library/sidebar";
import { TapeMenu } from "@/features/library/tape-actions";
import {
  DocumentHeader,
  entryOf,
  TapeHeader,
  titleOf,
} from "@/features/library/tape-header";
import { useLibrary } from "@/features/library/use-library";
import { useTapeOperations } from "@/features/library/use-tape-operations";
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
import { dropVerdict } from "@/features/source/drop";
import { DropVeil } from "@/features/source/drop-veil";
import { fileName, isTape, movedPath } from "@/features/source/file-name";
import {
  afterDiarization,
  afterTranscription,
  type Banner,
  bannerOf,
  errorText,
  type Status,
  withDiarizing,
  withLiveDiarizationError,
  withLiveError,
  withMovedSource,
  withProgress,
} from "@/features/status/status";
import {
  diarizationNeedsConfirmation,
  wasDiarized,
} from "@/features/transcription/diarization";
import { ParlantiTab } from "@/features/transcription/parlanti-tab";
import {
  type Conversation,
  EMPTY_CONVERSATION,
  type Parlante,
  type PhraseRef,
  parlantiOf,
  type Turn,
  visiblePhrases,
  withLiveTranscript,
  withNome,
  withoutPartials,
  withParlanti,
  withPartial,
  withPhrase,
  withTesto,
} from "@/features/transcription/phrases";
import { TranscribeMenu } from "@/features/transcription/transcribe-menu";
import { TranscriptView } from "@/features/transcription/transcript-view";
import { inEventSession } from "@/lib/event-session";

const COPIED_MS = 2000;
/** Quanto resta l'avviso di un esito, o di un errore di un'operazione sulla Libreria. */
const NOTICE_MS = 6000;

/** Un Tape aperto: le Frasi e le informazioni. */
interface TapeView {
  conversation: Conversation;
  info: TapeInfo | null;
  path: string;
}

/** Il Tape evidenziato nella barra laterale: quello mostrato nell'area principale. */
function selectedTape(
  listOpen: boolean,
  browsed: TapeView | null,
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
  // Le Frasi della Sorgente: quelle del Tape aperto, o quelle che arrivano da un'Attività con i
  // Parziali in corso (Nemotron), uno per Ingresso.
  const [conversation, setConversation] =
    useState<Conversation>(EMPTY_CONVERSATION);
  // Le informazioni del Tape aperto come Sorgente.
  const [info, setInfo] = useState<TapeInfo | null>(null);
  // Gli eventi possono arrivare dopo la risposta di `record`: un Parziale tardivo si ignora.
  const acceptPartials = useRef<boolean>(false);
  const recordingSession = useRef<string | null>(null);
  const { loadError, save, settings } = useSettings();
  // Impostazioni illeggibili all'avvio: la status bar lo dice finché non c'è altro da mostrare.
  const [status, setStatus] = useState<Status>(() =>
    loadError
      ? { error: loadError, phase: "failed" }
      : { phase: "idle", source: null }
  );
  // L'errore di un'operazione su un Tape o sulla Libreria, o un avviso (il Markdown esportato): la
  // status bar lo mostra per un po' sopra la fase, che durante un'Attività non deve cambiare.
  const [notice, setNotice] = useState<AppError | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [copied, setCopied] = useState(false);
  // L'esito o l'errore chiuso dall'utente: l'avviso torna con la fase successiva.
  const [dismissed, setDismissed] = useState<Status | null>(null);
  const [cancelling, setCancelling] = useState(false);
  // Trascrivi su un Tape aspetta la conferma: il testo e le correzioni si sostituiscono.
  const [confirmTranscribe, setConfirmTranscribe] = useState(false);
  const [confirmDiarize, setConfirmDiarize] = useState(false);
  const diarizationInFlight = useRef(false);
  const confirmationOpen = confirmTranscribe || confirmDiarize;
  // La Frase di un risultato della ricerca, evidenziata nel Tape aperto.
  const [highlight, setHighlight] = useState<PhraseRef | null>(null);
  // Cambia per riaprire il Tape già aperto dall'inizio del testo.
  const [revision, setRevision] = useState(0);
  // Il Tape del doppio clic in Esplora file, finché non si può aprire.
  const [pendingTape, setPendingTape] = useState<string | null>(null);
  // I file trascinati da Esplora file sopra la finestra, e quelli appena rilasciati.
  const [dragged, setDragged] = useState<string[] | null>(null);
  const [dropped, setDropped] = useState<string[] | null>(null);
  // Il Parlante di cui si sta scrivendo il nome nuovo.
  const [renaming, setRenaming] = useState<Parlante | null>(null);
  // Il Tape aperto dalla barra laterale durante un'Attività.
  const [browsed, setBrowsed] = useState<TapeView | null>(null);
  // L'elenco completo dei Tape della Raccolta nell'area principale.
  const [listOpen, setListOpen] = useState(false);
  // Il timer della Registrazione per la barra laterale.
  const [elapsedMs, setElapsedMs] = useState(0);
  const running = status.phase === "transcribing";
  const diarizing = status.phase === "diarizing";
  const recording = status.phase === "recording";
  // Dopo Stop, finché la Trascrizione dal vivo smaltisce la coda: fa ancora parte della Registrazione.
  const completing = status.phase === "completing";
  const paused = status.phase === "recording" && status.paused;
  // Una Attività alla volta: durante l'una, l'altra e Apri file sono disabilitate.
  const busy = running || diarizing || recording || completing;
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
    const liveText = events.liveTranscriptUpdated.listen(({ payload }) => {
      setConversation((current) =>
        withLiveTranscript(current, payload, acceptPartials.current)
      );
    });
    const progress = events.transcriptionProgress.listen(({ payload }) => {
      if (!inEventSession(payload, recordingSession.current)) {
        return;
      }
      setStatus((current) => withProgress(current, payload.percent));
    });
    // Finita la Trascrizione, Riconosci i parlanti attribuisce le Frasi ai Parlanti.
    const diarizationStarted = events.diarizationStarted.listen(
      ({ payload }) => {
        if (!inEventSession(payload, recordingSession.current)) {
          return;
        }
        setStatus(withDiarizing);
      }
    );
    const diarizerFailed = events.liveDiarizationFailed.listen(
      ({ payload }) => {
        if (!inEventSession(payload, recordingSession.current)) {
          return;
        }
        setStatus((current) =>
          withLiveDiarizationError(current, payload.error, payload.ingresso)
        );
      }
    );
    const assigned = events.speakersAssigned.listen(({ payload }) => {
      setConversation((current) =>
        withParlanti(current, payload.speakers, payload.sessionId)
      );
    });
    // La Trascrizione dal vivo si è fermata (o Riconosci i parlanti non ha il modello): la
    // Registrazione continua e la status bar lo dice.
    const liveFailed = events.liveTranscriptionFailed.listen(({ payload }) => {
      if (!inEventSession(payload, recordingSession.current)) {
        return;
      }
      setConversation(withoutPartials);
      setStatus((current) => withLiveError(current, payload.error));
    });
    const ticks = events.recordingTick.listen(({ payload }) => {
      if (!inEventSession(payload, recordingSession.current)) {
        return;
      }
      setElapsedMs(payload.elapsedMs);
    });
    // Il Tape dell'avvio, e quelli del doppio clic con l'app aperta. Si prende dopo aver registrato
    // il listener, così uno arrivato nel frattempo non si perde.
    const takeTape = () =>
      commands.takePendingTape().then((path) => path && setPendingTape(path));
    const tapeRequested = events.tapeRequested.listen(takeTape);
    tapeRequested.then(takeTape);
    return () => {
      phrases.then((stop) => stop());
      partials.then((stop) => stop());
      liveText.then((stop) => stop());
      progress.then((stop) => stop());
      liveFailed.then((stop) => stop());
      diarizationStarted.then((stop) => stop());
      diarizerFailed.then((stop) => stop());
      assigned.then((stop) => stop());
      ticks.then((stop) => stop());
      tapeRequested.then((stop) => stop());
    };
  }, []);

  // Il drop dei file passa da Tauri, che dà i percorsi già all'ingresso (ADR-0011).
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent(({ payload }) => {
      // Un trascinamento senza file (testo da un'altra app) non ha percorsi: niente velo.
      if (payload.type === "enter") {
        setDragged(payload.paths.length > 0 ? payload.paths : null);
      } else if (payload.type === "leave") {
        setDragged(null);
      } else if (payload.type === "drop") {
        setDragged(null);
        setDropped(payload.paths);
      }
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = setTimeout(() => setCopied(false), COPIED_MS);
    return () => clearTimeout(timer);
  }, [copied]);

  // Una versione più recente: l'avviso resta finché non si chiude. Offline non dice nulla.
  useEffect(() => {
    commands.checkUpdate().then(setUpdate);
  }, []);

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

  // Il Tape diventa la Sorgente, con il suo testo, senza ritrascrivere; da un risultato della
  // ricerca con la Frase trovata evidenziata. Restituisce l'errore, se non si apre.
  const loadTape = useCallback(async (path: string, phrase?: PhraseRef) => {
    const result = await commands.openTape(path);
    if (result.status === "error") {
      return result.error;
    }
    const { info: opened, parlanti, phrases } = result.data;
    recordingSession.current = null;
    setSource(path);
    setConversation({ ...EMPTY_CONVERSATION, parlanti, phrases });
    setInfo(opened);
    setHighlight(phrase ?? null);
    setRenaming(null);
    return null;
  }, []);

  const openTape = useCallback(
    async (path: string, phrase?: PhraseRef) => {
      const error = await loadTape(path, phrase);
      setStatus(
        error ? { error, phase: "failed" } : { phase: "idle", source: path }
      );
    },
    [loadTape]
  );

  // Durante un'Attività un Tape della Libreria si consulta accanto, senza toccarla; quello su cui
  // lavora l'Attività riporta alla sua vista.
  const browse = useCallback(
    async (path: string, phrase?: PhraseRef) => {
      setHighlight(phrase ?? null);
      setRenaming(null);
      // Solo Trascrivi lavora sulla Sorgente; durante una Registrazione è un Tape come gli altri.
      if (path === source && (running || diarizing)) {
        setBrowsed(null);
        return;
      }
      const result = await commands.openTape(path);
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
    [diarizing, running, source]
  );

  // Finita l'Attività torna la sua vista, con il suo esito aperto (il Tape di una Registrazione).
  useEffect(() => {
    if (!busy) {
      setBrowsed(null);
    }
  }, [busy]);

  // Una Sorgente scelta con Apri file o con il doppio clic su un Tape in Esplora file.
  const openPath = useCallback(
    (path: string, phrase?: PhraseRef) => {
      recordingSession.current = null;
      if (isTape(path)) {
        openTape(path, phrase);
        return;
      }
      setSource(path);
      setConversation(EMPTY_CONVERSATION);
      setInfo(null);
      setHighlight(null);
      setStatus({ phase: "idle", source: path });
    },
    [openTape]
  );

  const pickFile = useCallback(async () => {
    const picked = await commands.pickSource(t("source.filter"));
    if (picked) {
      setListOpen(false);
      openPath(picked);
    }
  }, [openPath, t]);

  // Un Tape della barra laterale, dell'elenco completo o della ricerca, con la Frase trovata.
  const openFromLibrary = useCallback(
    (path: string, phrase?: PhraseRef) => {
      setListOpen(false);
      if (busy) {
        browse(path, phrase);
      } else if (path === source) {
        // Il Tape già aperto: torna alla Frase trovata, o all'inizio del testo.
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
    // La Frase trovata era del Tape consultato.
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

  // Un Tape aperto e spostato o rinominato resta aperto, con il percorso nuovo.
  const moved = useCallback((from: string, to: string) => {
    setSource((current) => current && movedPath(current, from, to));
    setBrowsed(
      (current) =>
        current && { ...current, path: movedPath(current.path, from, to) }
    );
    setStatus((current) => withMovedSource(current, from, to));
  }, []);

  // Il Tape nel Cestino, se era aperto, non lo è più; la vista di un'Attività però resta.
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

  const { dialog, moveTape, renameTape, requestTrash, reveal } =
    useTapeOperations({
      onError: setNotice,
      onMoved: moved,
      onTrashed: trashed,
    });

  // Un Tape arrivato con il doppio clic in Esplora file aspetta che finisca l'Attività (Apri file
  // intanto è disabilitata) e che si chiuda la conferma di Trascrivi. Si apre sulla finestra
  // principale, anche se c'era Impostazioni sopra.
  useEffect(() => {
    if (pendingTape && !busy && !confirmationOpen) {
      setPendingTape(null);
      setListOpen(false);
      navigate("/");
      openPath(pendingTape);
    }
  }, [busy, confirmationOpen, navigate, openPath, pendingTape]);

  // Un file rilasciato si apre come con Apri file, anche da Impostazioni; durante un'Attività solo
  // un Tape, in consultazione. Con la conferma di Trascrivi aperta il rilascio non conta.
  useEffect(() => {
    if (!dropped) {
      return;
    }
    setDropped(null);
    const verdict = dropVerdict(dropped, busy);
    if (!verdict.accepted || confirmationOpen) {
      return;
    }
    navigate("/");
    setListOpen(false);
    if (busy) {
      browse(verdict.path);
    } else {
      openPath(verdict.path);
    }
  }, [browse, busy, confirmationOpen, dropped, navigate, openPath]);

  const open = useCallback(async () => {
    if (!source) {
      return;
    }
    const result = await commands.openSource(source);
    if (result.status === "error") {
      setStatus({ error: result.error, phase: "failed" });
    }
  }, [source]);

  // Un file diventa un Tape nella Raccolta scelta; un Tape si ritrascrive. Finita (o annullata) la
  // Trascrizione, il Tape si rilegge dal disco.
  const transcribe = useCallback(async () => {
    if (!source) {
      return;
    }
    recordingSession.current = null;
    setConversation(EMPTY_CONVERSATION);
    setHighlight(null);
    setRenaming(null);
    setCancelling(false);
    setStatus({ percent: null, phase: "transcribing" });
    try {
      const result = await commands.transcribe(source, raccolta);
      let opened: string | null = isTape(source) ? source : null;
      if (result.status === "ok" && result.data.outcome === "saved") {
        opened = result.data.path;
      }
      const error = opened ? await loadTape(opened) : null;
      setStatus(
        error ? { error, phase: "failed" } : afterTranscription(result)
      );
    } catch (e) {
      // `typedError` rilancia gli `Error` di IPC: la Trascrizione non deve restare "in corso".
      setStatus({ error: internalError(e), phase: "failed" });
    }
  }, [loadTape, raccolta, source]);

  // La nuova Diarizzazione lascia in vista il risultato precedente fino al salvataggio riuscito.
  const diarize = useCallback(async () => {
    if (!source || busy || diarizationInFlight.current) {
      return;
    }
    diarizationInFlight.current = true;
    setConfirmDiarize(false);
    recordingSession.current = null;
    setRenaming(null);
    setCancelling(false);
    setStatus({ phase: "diarizing" });
    try {
      const result = await commands.diarize(source);
      const error = result.status === "ok" ? await loadTape(source) : null;
      if (result.status === "ok" && !error) {
        setRevision((current) => current + 1);
      }
      setStatus(
        error ? { error, phase: "failed" } : afterDiarization(result, source)
      );
    } catch (e) {
      setStatus({ error: internalError(e), phase: "failed" });
    } finally {
      diarizationInFlight.current = false;
      setCancelling(false);
    }
  }, [busy, loadTape, source]);

  const requestDiarization = useCallback(() => {
    if (busy || !conversation.phrases.length) {
      return;
    }
    if (diarizationNeedsConfirmation(info, conversation.parlanti)) {
      setConfirmDiarize(true);
    } else {
      diarize();
    }
  }, [busy, conversation.parlanti, conversation.phrases.length, diarize, info]);

  const closeDiarizationConfirm = useCallback((opened: boolean) => {
    if (!opened) {
      setConfirmDiarize(false);
    }
  }, []);

  // Il Tape della Registrazione diventa la Sorgente; se non è partita torna quella di prima.
  const record = useCallback(async () => {
    const before = source;
    const sessionId = crypto.randomUUID();
    recordingSession.current = sessionId;
    setConversation({ ...EMPTY_CONVERSATION, sessionId });
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
        await commands.record(sessionId, t("recording.prefix"), raccolta)
      );
      acceptPartials.current = false;
      setConversation(withoutPartials);
      const opened = after.source ?? before;
      let error: AppError | null = null;
      if (opened && isTape(opened)) {
        error = await loadTape(opened);
      } else if (after.source) {
        // Il Tape non si è scritto: la Sorgente è l'Ogg, con il testo dal vivo.
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
  }, [loadTape, raccolta, settings.trascrizioneDalVivo, source, t]);

  const setPaused = useCallback((value: boolean) => {
    setStatus((current) =>
      current.phase === "recording" ? { ...current, paused: value } : current
    );
  }, []);

  // Ritrascrivere un Tape ne sostituisce il testo, correzioni comprese: prima si conferma.
  const requestTranscription = useCallback(() => {
    if (source && isTape(source)) {
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

  // Il documento del Tape `path`, o senza Tape quello della Trascrizione in corso o appena finita,
  // in testo semplice o Markdown secondo le impostazioni.
  const copy = useCallback(
    async (path: string | null) => {
      try {
        let text: string | null;
        if (path) {
          const result = await commands.tapeText(path);
          if (result.status === "error") {
            setNotice(result.error);
            return;
          }
          text = result.data;
        } else {
          text = await commands.transcriptText(visiblePhrases(conversation));
        }
        await navigator.clipboard.writeText(text ?? "");
        setCopied(true);
      } catch (e) {
        setNotice(internalError(e));
      }
    },
    [conversation]
  );

  const exported = useCallback(
    (path: string) => setMessage(t("transcription.exported", { path })),
    [t]
  );

  // Una correzione o un nome salvati nel Tape `path` valgono per la sua vista.
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

  // Il dato manuale viene mostrato soltanto dopo una scrittura riuscita.
  const markCorrected = useCallback(
    (path: string) => {
      setBrowsed((current) =>
        current?.path === path && current.info
          ? { ...current, info: { ...current.info, correttoAMano: true } }
          : current
      );
      if (path === source) {
        setInfo((current) => current && { ...current, correttoAMano: true });
      }
    },
    [source]
  );

  const merge = useCallback(
    async (path: string, turn: Turn, target: PhraseRef) => {
      try {
        const result = await commands.unisciTurno(
          path,
          turn.ingresso,
          turn.items.map((item) => item.phraseId),
          target.phraseId
        );
        if (result.status === "error") {
          setNotice(result.error);
          return false;
        }
        const opened = await commands.openTape(path);
        if (opened.status === "error") {
          setNotice(opened.error);
          return false;
        }
        const { info: updated, parlanti, phrases } = opened.data;
        updateView(path, (c) => ({ ...c, parlanti, phrases }));
        setBrowsed((current) =>
          current?.path === path ? { ...current, info: updated } : current
        );
        if (path === source) {
          setInfo(updated);
        }
        return true;
      } catch (e) {
        setNotice(internalError(e));
        return false;
      }
    },
    [source, updateView]
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
      if (nome.trim() !== voce.nome) {
        markCorrected(path);
      }
    },
    [markCorrected, updateView]
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
      markCorrected(path);
      return true;
    },
    [markCorrected, updateView]
  );

  // La data e l'ora nuove del Tape `path`, nell'ora locale del campo.
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

  // La Raccolta di un Tape dal suo percorso nella barra in alto: apre la Libreria lì.
  const openRaccolta = useCallback(
    (value: string) => {
      chooseRaccolta(value);
      setListOpen(true);
    },
    [chooseRaccolta]
  );

  // L'avviso in cima: l'errore di un'operazione, un messaggio o l'esito dell'Attività.
  const statusBanner: Banner | null =
    dismissed === status ? null : bannerOf(status, t);
  const banner = shownBanner(
    notice,
    message,
    statusBanner ?? updateBanner(update, t),
    t
  );
  const dismiss = useCallback(() => {
    setNotice(null);
    setMessage(null);
    setDismissed(status);
    if (banner?.updateUrl) {
      setUpdate(null);
    }
  }, [banner, status]);
  // Gli esiti spariscono da soli dopo un po'; gli errori restano finché non si chiudono.
  useEffect(() => {
    if (statusBanner?.tone !== "info") {
      return;
    }
    const timer = setTimeout(() => setDismissed(status), NOTICE_MS);
    return () => clearTimeout(timer);
  }, [status, statusBanner?.tone]);

  // La vista di un Tape; `own`: è la Sorgente, che Trascrivi trascrive.
  const tapePane = (view: TapeView, own: boolean) => (
    <TapePane
      busy={busy}
      copied={copied}
      // Non ci lavora l'Attività in corso.
      editable={!(own && busy)}
      highlight={highlight}
      key={`${view.path}:${revision}`}
      library={library}
      onCopy={copy}
      onCreato={changeCreato}
      onDiarize={requestDiarization}
      onEdit={edit}
      onError={setNotice}
      onExported={exported}
      onMerge={merge}
      onMove={moveTape}
      onRaccolta={openRaccolta}
      onRenameParlante={rename}
      onRenameTitle={renameTape}
      onRenaming={setRenaming}
      onReveal={reveal}
      onTranscribe={requestTranscription}
      onTrash={requestTrash}
      own={own}
      processing={own && (running || diarizing)}
      recording={recording}
      renaming={renaming}
      running={running}
      view={view}
    />
  );

  let mainView: React.ReactNode;
  if (listOpen) {
    mainView = (
      <>
        <TopBar crumbs={<Crumb current>{t("library.title")}</Crumb>} />
        <AllTapes
          list={library}
          onError={setNotice}
          onMove={moveTape}
          onMoved={moved}
          onOpen={openFromLibrary}
          onRaccolta={chooseRaccolta}
          onTrash={requestTrash}
          raccolta={raccolta}
        />
      </>
    );
  } else if (browsed) {
    mainView = tapePane(browsed, false);
  } else if (recording || completing) {
    mainView = (
      <LiveView
        conversation={conversation}
        copied={copied}
        onCopy={copyActivity}
        onError={setNotice}
        onPausedChange={setPaused}
        paused={paused}
        status={status}
      />
    );
  } else if (source && isTape(source)) {
    mainView = tapePane({ conversation, info, path: source }, true);
  } else if (source) {
    mainView = (
      <FileView
        busy={busy}
        conversation={conversation}
        copied={copied}
        onCopy={copyActivity}
        onError={failed}
        onOpen={open}
        onTranscribe={requestTranscription}
        running={running}
        source={source}
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
          cancelling={cancelling}
          list={library}
          onActivity={showActivity}
          onCancel={cancel}
          onError={setNotice}
          onImport={pickFile}
          onOpen={openFromLibrary}
          onShowAll={showAll}
          record={
            <RecordMenu disabled={busy} onError={failed} onRecord={record} />
          }
          selected={selectedTape(
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
              {t(
                info?.correttoAMano
                  ? "transcription.replace.manualDescription"
                  : "transcription.replace.description"
              )}
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
      <AlertDialog onOpenChange={closeDiarizationConfirm} open={confirmDiarize}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t("diarization.replace.title")}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t("diarization.replace.description")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              {t("diarization.replace.keep")}
            </AlertDialogCancel>
            <AlertDialogAction onClick={diarize}>
              {t("diarization.replace.confirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      {dialog}
      {settingsPage}
      <WindowControls />
      {dragged && !confirmationOpen ? (
        <DropVeil verdict={dropVerdict(dragged, busy)} />
      ) : null}
    </>
  );
}

function updateBanner(update: UpdateInfo | null, t: TFunction): Banner | null {
  return update
    ? {
        settings: false,
        text: t("status.updateAvailable", { version: update.version }),
        tone: "info",
        updateUrl: update.url,
      }
    : null;
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
  conversation,
  copied,
  onCopy,
  onError,
  onPausedChange,
  paused,
  status,
}: {
  conversation: Conversation;
  copied: boolean;
  onCopy: () => void;
  onError: (error: AppError) => void;
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
            disabled={
              conversation.phrases.length === 0 &&
              conversation.partials.length === 0
            }
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
      {recording ? (
        <Dock>
          <RecordingPanel
            key={conversation.sessionId}
            onError={onError}
            onPausedChange={onPausedChange}
            paused={paused}
            sessionId={conversation.sessionId}
          />
        </Dock>
      ) : null}
    </>
  );
}

/**
 * Un file audio o video: il nome, che un clic apre, e Trascrivi; dopo una Trascrizione annullata,
 * le sue Frasi.
 */
function FileView({
  busy,
  conversation,
  copied,
  onCopy,
  onError,
  onOpen,
  onTranscribe,
  running,
  source,
}: {
  busy: boolean;
  conversation: Conversation;
  copied: boolean;
  onCopy: () => void;
  onError: (error: AppError) => void;
  onOpen: () => void;
  onTranscribe: () => void;
  running: boolean;
  source: string;
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

/** Il posto in fondo al pannello centrale per il player o la Registrazione. */
function Dock({ children }: { children: React.ReactNode }) {
  return <div className="shrink-0 px-8 pt-1 pb-5">{children}</div>;
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
  const { updateUrl } = banner;
  const openUpdate = useCallback(() => {
    if (updateUrl) {
      commands.openUpdate(updateUrl);
    }
  }, [updateUrl]);
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
              to="/settings?sezione=trascrizione"
            >
              {t("status.openModels")}
            </Link>
          </>
        ) : null}
        {banner.updateUrl ? (
          <>
            {" "}
            <button
              className="whitespace-nowrap font-medium underline underline-offset-4"
              onClick={openUpdate}
              type="button"
            >
              {t("status.updateDownload")}
            </button>
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
      {/* Sulla stessa colonna e alla stessa altezza del titolo di un Tape: lo stesso foglio, vuoto. */}
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
 * La vista di un Tape: la barra in alto con il percorso, Copia testo e "…"; il documento con
 * testata, schede Trascrizione e Parlanti e Segui l'audio; in fondo il player, nascosto
 * mentre lo si trascrive. `editable`: non ci lavora l'Attività in corso; `own`: è la Sorgente, che
 * Trascrivi trascrive; `recording`: il player è disabilitato.
 */
function TapePane({
  busy,
  copied,
  editable,
  highlight,
  library,
  onCopy,
  onCreato,
  onDiarize,
  onEdit,
  onMerge,
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
  processing,
  recording,
  renaming,
  running,
  view: { conversation, info, path },
}: {
  busy: boolean;
  copied: boolean;
  editable: boolean;
  highlight: PhraseRef | null;
  library: LibraryList;
  onCopy: (path: string) => void;
  onCreato: (path: string, local: string) => void;
  onDiarize: () => void;
  onEdit: (path: string, phrase: PhraseRef, text: string) => Promise<boolean>;
  onMerge: (path: string, turn: Turn, target: PhraseRef) => Promise<boolean>;
  onError: (error: AppError) => void;
  onExported: (path: string) => void;
  onMove: (path: string, raccolta: string) => void;
  onRaccolta: (raccolta: string) => void;
  onRenameParlante: (path: string, voce: Parlante, nome: string) => void;
  onRenameTitle: (path: string, titolo: string) => void;
  onRenaming: (voce: Parlante | null) => void;
  onReveal: (path: string) => void;
  onTranscribe: () => void;
  onTrash: (tape: { path: string; titolo: string }) => void;
  own: boolean;
  processing: boolean;
  recording: boolean;
  renaming: Parlante | null;
  running: boolean;
  view: TapeView;
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
  const merge = useCallback(
    (turn: Turn, target: PhraseRef) => onMerge(path, turn, target),
    [onMerge, path]
  );
  const entry = entryOf(library, path);
  const raccolta = entry ? (entry.raccolta ?? "") : null;
  const toRaccolta = useCallback(
    () => raccolta !== null && onRaccolta(raccolta),
    [onRaccolta, raccolta]
  );
  // Durante una Registrazione niente salti né evidenziazione: il player è disabilitato.
  const listening = processing || recording ? undefined : player;
  const hasText = conversation.phrases.length > 0;
  const editingActions = editable ? { onEdit: edit, onMerge: merge } : {};

  const header = (
    <>
      <TapeHeader
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
    <TapeEmpty
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
            <TapeMenu
              diarized={wasDiarized(info, conversation.phrases)}
              diarizingDisabled={busy || !hasText}
              disabled={!editable}
              library={library}
              onDiarize={own ? onDiarize : undefined}
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
          {...editingActions}
          onRename={rename}
          onRenaming={onRenaming}
          parlanti={parlanti}
          player={listening}
          renaming={renaming}
        />
      )}
      <TapePlayer
        disabled={recording}
        info={info}
        path={path}
        player={player}
        processing={processing}
      />
    </>
  );
}

function TapePlayer({
  disabled,
  info,
  path,
  player,
  processing,
}: {
  disabled: boolean;
  info: TapeInfo | null;
  path: string;
  player: PlayerState;
  processing: boolean;
}) {
  const { t } = useTranslation();
  return processing ? null : (
    <Dock>
      <Player
        disabled={disabled}
        label={
          info?.origine
            ? t("player.file", { name: info.origine })
            : t("player.recording")
        }
        path={path}
        player={player}
      />
    </Dock>
  );
}

/** Il documento di un Tape senza Frasi: in Trascrizione, o da trascrivere (se `onTranscribe`). */
function TapeEmpty({
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
        <p className="font-medium">{t("tape.noText")}</p>
        <p className="max-w-[34rem] text-muted-foreground text-sm leading-relaxed">
          {t("tape.noTextDescription")}
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
