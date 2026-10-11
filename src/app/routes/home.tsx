import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { TFunction } from "i18next";
import { Check, CircleAlert, Copy, X } from "lucide-react";
import {
  createContext,
  type ReactNode,
  type RefObject,
  use,
  useCallback,
  useEffect,
  useEffectEvent,
  useMemo,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useOutlet } from "react-router";
import {
  centerView,
  selection,
  tapeMostrato,
  useHomeState,
} from "@/app/routes/home-state";
import {
  type AppError,
  commands,
  events,
  type LibraryList,
  type OpenedTape,
  type RecordingCleaningFailed,
  type RecordingCleaningPreparing,
  type TapeEntry,
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
import {
  WINDOW_CONTROLS_PADDING,
  WindowControls,
} from "@/components/window-controls";
import { AllTapes } from "@/features/library/all-tapes";
import { useBarraLaterale } from "@/features/library/barra-laterale";
import { chosenRaccolta, raccoltaLabel } from "@/features/library/library";
import { LibraryHome } from "@/features/library/library-home";
import { recentTitle } from "@/features/library/recent-tapes";
import {
  SIDEBAR_TOGGLE_PADDING,
  Sidebar,
  SidebarToggle,
} from "@/features/library/sidebar";
import { TapeMenu } from "@/features/library/tape-actions";
import type { TapeOperations } from "@/features/library/tape-context-menu";
import {
  DocumentHeader,
  entryOf,
  TapeHeader,
  titleOf,
} from "@/features/library/tape-header";
import { useLastTape } from "@/features/library/use-last-tape";
import { useLibrary } from "@/features/library/use-library";
import { useTapeOperations } from "@/features/library/use-tape-operations";
import { type BozzeAction, bozzeDi } from "@/features/notes/bozze";
import { NotesTab } from "@/features/notes/notes-tab";
import {
  type NoteDelTape,
  quanteNote,
  useNote,
} from "@/features/notes/use-note";
import { useNoteRegistrazione } from "@/features/notes/use-note-registrazione";
import { useUscitaProtetta } from "@/features/notes/use-uscita-protetta";
import { Player, type PlayerState, usePlayer } from "@/features/player/player";
import { PreparationPanel } from "@/features/recording/preparation-panel";
import { RecordMenu } from "@/features/recording/record-menu";
import {
  activitySummary,
  afterRecording,
} from "@/features/recording/recording";
import { RecordingPanel } from "@/features/recording/recording-panel";
import { recordAfterSettings } from "@/features/recording/start";
import { nomeMicrofonoRegistrazione } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { ariaTasti, conTasti } from "@/features/shortcuts/shortcuts";
import {
  ScorciatoieProvider,
  useScorciatoia,
} from "@/features/shortcuts/shortcuts-provider";
import { dropVerdict } from "@/features/source/drop";
import { DropVeil } from "@/features/source/drop-veil";
import { fileName, isTape } from "@/features/source/file-name";
import {
  alRitorno,
  opening,
  pendingTape,
  type TapeView,
  type Trovato,
  trovato,
} from "@/features/source/view";
import {
  afterDiarization,
  afterTranscription,
  type Banner,
  bannerOf,
  errorText,
  isBusy,
  type Status,
  statusText,
  transcribesSource,
} from "@/features/status/status";
import type { RichiestaTaglio } from "@/features/taglio/taglio";
import { useTaglio } from "@/features/taglio/use-taglio";
import {
  diarizationNeedsConfirmation,
  wasDiarized,
} from "@/features/transcription/diarization";
import { ParlantiTab } from "@/features/transcription/parlanti-tab";
import {
  type Conversation,
  type Parlante,
  type PhraseRef,
  parlantiOf,
  type Turn,
  visiblePhrases,
  withNome,
} from "@/features/transcription/phrases";
import {
  TranscribeButton,
  TranscribeDialog,
  type TranscribeTarget,
} from "@/features/transcription/transcribe-menu";
import { TranscriptView } from "@/features/transcription/transcript-view";

const COPIED_MS = 2000;
/** Quanto resta l'avviso di un esito, o di un errore di un'operazione sulla Libreria. */
const NOTICE_MS = 6000;

function internalError(e: unknown): AppError {
  return { code: "internal", detail: String(e) };
}

export function HomePage() {
  const { t } = useTranslation();
  const pendingEdits = useRef(new Map<string, Promise<boolean>>());
  const { loadError, save, flush, settings } = useSettings();
  const recordingInFlight = useRef<boolean>(false);
  const pendingRecording = useRef<{
    sessionId: string;
    dispatched: boolean;
    controller: AbortController;
    started: boolean;
  } | null>(null);
  const recordingStarted = useCallback((sessionId: string) => {
    if (pendingRecording.current?.sessionId === sessionId) {
      pendingRecording.current.started = true;
    }
  }, []);
  // Lo Status, le Frasi e le informazioni della Sorgente (del Tape aperto o dell'Attività in
  // corso), il timer, la pulizia e il Tape consultato. Impostazioni illeggibili all'avvio: l'avviso
  // lo dice finché non c'è altro.
  const { dispatch, ready, state } = useHomeState(
    loadError
      ? { error: loadError, phase: "failed" }
      : { phase: "idle", source: null },
    recordingStarted
  );
  const {
    cleaningDismissed,
    cleaningFailures,
    cleaningPreparing,
    conversation,
    elapsedMs,
    info,
    session,
    status,
  } = state.activity;
  const { highlight, revision, source } = state.view;
  // L'errore di un'operazione su un Tape o sulla Libreria, o un avviso (il Markdown esportato):
  // l'avviso lo mostra per un po' sopra la fase, che durante un'Attività non deve cambiare.
  const [notice, setNotice] = useState<AppError | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  // Le Note della Registrazione restano lasciando la sua vista e tornandoci.
  const noteRegistrazione = useNoteRegistrazione({
    onError: setNotice,
    session,
  });
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [copied, setCopied] = useState(false);
  // L'esito o l'errore chiuso dall'utente: l'avviso torna con la fase successiva.
  const [dismissed, setDismissed] = useState<Status | null>(null);
  const [cancelling, setCancelling] = useState(false);
  // Trascrivi passa sempre dal dialog delle scelte; su un Tape con testo avvisa che si sostituisce.
  const [confirmTranscribe, setConfirmTranscribe] = useState(false);
  const [confirmDiarize, setConfirmDiarize] = useState(false);
  const diarizationInFlight = useRef(false);
  const confirmationOpen = confirmTranscribe || confirmDiarize;
  // I file trascinati da Esplora file sopra la finestra.
  const [dragged, setDragged] = useState<string[] | null>(null);
  const { lastPath, remember, move: moveLastTape, forget } = useLastTape();
  const barra = useBarraLaterale();
  const running = status.phase === "transcribing";
  const recording = status.phase === "recording";
  const paused = status.phase === "recording" && status.paused;
  const preparing = status.phase === "preparingRecording";
  // Una Attività alla volta: durante l'una, l'altra e Apri file sono disabilitate.
  const busy = isBusy(status);
  // Impostazioni, aperta sopra questa finestra.
  const settingsPage = useOutlet();
  const navigate = useNavigate();
  const { list: library, loading: libraryLoading } = useLibrary(setNotice);
  const raccolta = chosenRaccolta(settings.raccolta, library.raccolte);

  // Le bozze delle Note e l'uscita protetta dal Tape mostrato.
  const shown = centerView(state);
  const {
    bozze: bozzeNote,
    dialog: noteDialog,
    dispatch: bozzeDispatch,
    proteggi,
    salvaNote,
    scarti,
  } = useUscitaProtetta(tapeMostrato(shown));

  // Il Tape dell'avvio, e quelli del doppio clic con l'app aperta. Si prende dopo aver registrato
  // il listener, così uno arrivato nel frattempo non si perde.
  useEffect(() => {
    const takeTape = () =>
      commands
        .takePendingTape()
        .then((path) => path && dispatch({ path, type: "tapeRequested" }));
    const tapeRequested = events.tapeRequested.listen(takeTape);
    tapeRequested.then(takeTape);
    return () => {
      tapeRequested.then((stop) => stop());
    };
  }, [dispatch]);

  // Spenta la pulizia di un Ingresso, la sua preparazione non si mostra più.
  const microfonoPulito = settings.audioMicrofono?.pulizia;
  const sistemaPulito = settings.audioSistema?.pulizia;
  useEffect(() => {
    if (!microfonoPulito) {
      dispatch({ ingresso: "microfono", type: "cleaningOff" });
    }
    if (!sistemaPulito) {
      dispatch({ ingresso: "sistema", type: "cleaningOff" });
    }
  }, [dispatch, microfonoPulito, sistemaPulito]);

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
  // ricerca con la Frase trovata evidenziata; `restart`: la sua vista riparte dall'inizio.
  // Restituisce l'errore, se non si apre.
  const loadTape = useCallback(
    async (path: string, phrase?: Trovato, restart = false) => {
      const result = await commands.openTape(path);
      if (result.status === "error") {
        return result.error;
      }
      remember(path);
      dispatch({
        path,
        phrase,
        restart,
        tape: result.data,
        type: "sourceOpened",
      });
      return null;
    },
    [dispatch, remember]
  );

  const openTape = useCallback(
    async (path: string, phrase?: Trovato) => {
      const error = await loadTape(path, phrase);
      if (error) {
        dispatch({ status: { error, phase: "failed" }, type: "status" });
      }
    },
    [dispatch, loadTape]
  );

  // Durante un'Attività un Tape della Libreria si consulta accanto, senza toccarla.
  const consult = useCallback(
    async (path: string, phrase?: Trovato) => {
      const result = await commands.openTape(path);
      if (result.status === "error") {
        setNotice(result.error);
        return;
      }
      remember(path);
      dispatch({ path, phrase, tape: result.data, type: "tapeConsulted" });
    },
    [dispatch, remember]
  );

  // Un Tape della barra laterale, della Libreria o della ricerca (`fromLibrary`, con la Frase
  // trovata), o un percorso di Apri file, del doppio clic o rilasciato: la vista della Sorgente
  // dice cosa leggere, qui lo si legge.
  const requestOpen = useCallback(
    (path: string, phrase?: Trovato, fromLibrary = false) => {
      const how = opening(state.view, status, path, fromLibrary);
      dispatch({ opening: how, path, phrase, type: "openRequested" });
      if (how === "restart") {
        remember(path);
      } else if (how === "consult") {
        consult(path, phrase);
      } else if (how === "source" && isTape(path)) {
        openTape(path, phrase);
      } else if (how === "source") {
        dispatch({ path, type: "sourceOpened" });
      }
    },
    [consult, dispatch, openTape, remember, state.view, status]
  );

  const pickFile = useCallback(
    () =>
      proteggi(async () => {
        const picked = await commands.pickSource(t("source.filter"));
        if (picked) {
          requestOpen(picked);
        }
      }),
    [proteggi, requestOpen, t]
  );

  const openFromLibrary = useCallback(
    (path: string, phrase?: Trovato) =>
      proteggi(() => requestOpen(path, phrase, true), path),
    [proteggi, requestOpen]
  );

  const showActivity = useCallback(
    () => proteggi(() => dispatch({ type: "activityShown" })),
    [dispatch, proteggi]
  );
  const showAll = useCallback(
    () => proteggi(() => dispatch({ type: "libraryShown" })),
    [dispatch, proteggi]
  );
  const showHome = useCallback(
    () => proteggi(() => dispatch({ type: "homeShown" })),
    [dispatch, proteggi]
  );

  const chooseRaccolta = useCallback(
    async (value: string | null) => {
      const error = await save((current) => ({ ...current, raccolta: value }));
      if (error) {
        setNotice(error);
      }
    },
    [save]
  );

  // Un Tape aperto e spostato o rinominato resta aperto, con il percorso nuovo.
  const moved = useCallback(
    (from: string, to: string) => {
      moveLastTape(from, to);
      dispatch({ from, to, type: "tapeMoved" });
      bozzeDispatch({ from, to, type: "tapeMoved" });
    },
    [bozzeDispatch, dispatch, moveLastTape]
  );

  // Il Tape che non c'è più (nel Cestino o sparito dal disco), se era aperto, si chiude; la vista
  // di un'Attività però resta.
  const gone = useCallback(
    (path: string) => {
      forget(path);
      dispatch({ path, type: "tapeGone" });
      bozzeDispatch({ path, type: "tapeGone" });
    },
    [bozzeDispatch, dispatch, forget]
  );

  const { dialog, moveTape, renameTape, requestTrash, reveal } =
    useTapeOperations({
      onError: setNotice,
      onMoved: moved,
      onTrashed: gone,
    });

  // Un Tape arrivato con il doppio clic in Esplora file aspetta che finisca l'Attività (Apri file
  // intanto è disabilitata) e che si chiuda la conferma di Trascrivi. Si apre sulla finestra
  // principale, anche se c'era Impostazioni sopra.
  // Con le bozze delle Note chiede una volta: Annulla lo lascia in attesa.
  const tapeToOpen = pendingTape(state.view, status, confirmationOpen);
  const openPending = useEffectEvent((path: string) =>
    proteggi(() => {
      navigate("/");
      requestOpen(path);
    }, path)
  );
  useEffect(() => {
    if (tapeToOpen) {
      openPending(tapeToOpen);
    }
  }, [tapeToOpen]);

  // Un file rilasciato si apre come con Apri file, anche da Impostazioni; durante un'Attività solo
  // un Tape, in consultazione. Con la conferma di Trascrivi aperta il rilascio non conta.
  const dropped = useEffectEvent((paths: string[]) => {
    const verdict = dropVerdict(paths, busy);
    if (verdict.accepted && !confirmationOpen) {
      const { path } = verdict;
      proteggi(() => {
        navigate("/");
        requestOpen(path);
      }, path);
    }
  });

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
        dropped(payload.paths);
      }
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  const open = useCallback(async () => {
    if (!source) {
      return;
    }
    const result = await commands.openSource(source);
    if (result.status === "error") {
      dispatch({
        status: { error: result.error, phase: "failed" },
        type: "status",
      });
    }
  }, [dispatch, source]);

  // Un file diventa un Tape nella Raccolta scelta; un Tape si ritrascrive. Finita (o annullata) la
  // Trascrizione, il Tape si rilegge dal disco.
  const transcribe = useCallback(async () => {
    if (!source) {
      return;
    }
    // Le scelte appena fatte nel dialog devono essere salvate prima che il backend le legga.
    const unsaved = await flush();
    if (unsaved) {
      setNotice(unsaved);
      return;
    }
    dispatch({ kind: "transcription", type: "start" });
    setCancelling(false);
    try {
      const result = await commands.transcribe(source, raccolta);
      let opened: string | null = isTape(source) ? source : null;
      if (result.status === "ok" && result.data.outcome === "saved") {
        opened = result.data.path;
      }
      const error = opened ? await loadTape(opened) : null;
      dispatch({
        status: error ? { error, phase: "failed" } : afterTranscription(result),
        type: "outcome",
      });
    } catch (e) {
      // `typedError` rilancia gli `Error` di IPC: la Trascrizione non deve restare "in corso".
      dispatch({
        status: { error: internalError(e), phase: "failed" },
        type: "outcome",
      });
    }
  }, [dispatch, flush, loadTape, raccolta, source]);

  // La nuova Diarizzazione lascia in vista il risultato precedente fino al salvataggio riuscito.
  const diarize = useCallback(async () => {
    if (!source || busy || diarizationInFlight.current) {
      return;
    }
    diarizationInFlight.current = true;
    setConfirmDiarize(false);
    dispatch({ kind: "diarization", type: "start" });
    setCancelling(false);
    try {
      const result = await commands.diarize(source);
      // Riuscita, la vista del Tape riparte dall'inizio.
      const error =
        result.status === "ok" ? await loadTape(source, undefined, true) : null;
      dispatch({
        status: error
          ? { error, phase: "failed" }
          : afterDiarization(result, source),
        type: "outcome",
      });
    } catch (e) {
      dispatch({
        status: { error: internalError(e), phase: "failed" },
        type: "outcome",
      });
    } finally {
      diarizationInFlight.current = false;
      setCancelling(false);
    }
  }, [busy, dispatch, loadTape, source]);

  const requestDiarization = useCallback(() => {
    if (busy || !conversation.phrases.length) {
      return;
    }
    proteggi(() => {
      if (diarizationNeedsConfirmation(info, conversation.parlanti)) {
        setConfirmDiarize(true);
      } else {
        diarize();
      }
    });
  }, [
    busy,
    conversation.parlanti,
    conversation.phrases.length,
    diarize,
    info,
    proteggi,
  ]);

  const closeDiarizationConfirm = useCallback((opened: boolean) => {
    if (!opened) {
      setConfirmDiarize(false);
    }
  }, []);

  const finishRecording = useCallback(
    async (
      result: Awaited<ReturnType<typeof commands.record>>,
      before: string | null
    ) => {
      const after = afterRecording(result);
      const opened = after.source ?? before;
      let error: AppError | null = null;
      if (opened && isTape(opened)) {
        error = await loadTape(opened);
      } else if (after.source) {
        // L'Ogg tenuto: come Sorgente, senza chiudere Home o Libreria aperte intanto.
        dispatch({ keepView: true, path: after.source, type: "sourceOpened" });
      }
      dispatch({
        status:
          error && after.status.phase !== "failed"
            ? { error, phase: "failed" }
            : after.status,
        type: "outcome",
      });
    },
    [dispatch, loadTape]
  );

  // Il Tape della Registrazione diventa la Sorgente; se non è partita torna quella di prima.
  const record = useCallback(async () => {
    if (recordingInFlight.current) {
      return;
    }
    recordingInFlight.current = true;
    const before = source;
    const request = {
      controller: new AbortController(),
      dispatched: false,
      sessionId: crypto.randomUUID(),
      started: false,
    };
    pendingRecording.current = request;
    // Il Microfono persona sola ha già il nome predefinito, come nel Tape che nascerà.
    dispatch({
      kind: "recording",
      nomeMicrofono: nomeMicrofonoRegistrazione(settings),
      partials: settings.trascrizioneDalVivo ?? false,
      sessionId: request.sessionId,
      type: "start",
    });
    setCancelling(false);
    const restore = (error?: AppError) => {
      dispatch({ type: "restore" });
      if (error && error.code !== "cancelled") {
        setNotice(error);
      }
    };
    try {
      const result = await recordAfterSettings(
        flush,
        () => {
          request.dispatched = true;
          dispatch({ type: "recordingRequested" });
          return commands.record(
            request.sessionId,
            t("recording.prefix"),
            raccolta
          );
        },
        request.controller.signal,
        ready()
      );
      if (result.status === "error" && !request.started) {
        restore(result.error);
        return;
      }
      await finishRecording(result, before);
    } catch (e) {
      const error = internalError(e);
      if (request.started) {
        dispatch({ status: { error, phase: "failed" }, type: "outcome" });
      } else {
        restore(error);
      }
    } finally {
      pendingRecording.current = null;
      recordingInFlight.current = false;
      setCancelling(false);
    }
  }, [dispatch, finishRecording, flush, raccolta, ready, settings, source, t]);

  const cancelRecordingStart = useCallback(async () => {
    const request = pendingRecording.current;
    if (!request) {
      return;
    }
    request.controller.abort();
    setCancelling(true);
    if (request.dispatched) {
      try {
        await commands.cancelRecordingStart(request.sessionId);
      } catch (error) {
        setCancelling(false);
        setNotice(internalError(error));
      }
    }
  }, []);

  const setPaused = useCallback(
    (value: boolean) => dispatch({ paused: value, type: "pause" }),
    [dispatch]
  );

  const requestTranscription = useCallback(
    () => proteggi(() => setConfirmTranscribe(true)),
    [proteggi]
  );
  const newRecording = useCallback(() => proteggi(record), [proteggi, record]);
  let transcribeTarget: TranscribeTarget = { kind: "tape" };
  if (source && !isTape(source)) {
    transcribeTarget = { kind: "file", name: fileName(source) };
  } else if (conversation.phrases.length > 0) {
    transcribeTarget = {
      kind: "replace",
      manual: info?.correttoAMano ?? false,
    };
  }

  const closeConfirm = useCallback((opened: boolean) => {
    if (!opened) {
      setConfirmTranscribe(false);
    }
  }, []);

  const failed = useCallback(
    (error: AppError) =>
      dispatch({ status: { error, phase: "failed" }, type: "status" }),
    [dispatch]
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
  const tapeText = useCallback(async (path: string) => {
    // Prima le correzioni ancora in scrittura; se non si salvano, niente copia.
    if ((await pendingEdits.current.get(path)) === false) {
      return null;
    }
    const result = await commands.tapeText(path);
    if (result.status === "error") {
      setNotice(result.error);
      return null;
    }
    return result.data;
  }, []);
  const copy = useCallback(
    async (path: string | null) => {
      try {
        const text = path
          ? await tapeText(path)
          : ((await commands.transcriptText(visiblePhrases(conversation))) ??
            "");
        if (text !== null) {
          await navigator.clipboard.writeText(text);
          setCopied(true);
        }
      } catch (e) {
        setNotice(internalError(e));
      }
    },
    [conversation, tapeText]
  );
  // Copia testo dal menu di un Tape della Libreria o dei Recenti: l'avviso dice il titolo mostrato.
  const copyTape = useCallback(
    async (tape: TapeEntry) => {
      try {
        const text = await tapeText(tape.path);
        if (text !== null) {
          await navigator.clipboard.writeText(text);
          const titolo = recentTitle(tape, library.tapes, t);
          setMessage(t("library.textCopied", { titolo }));
        }
      } catch (e) {
        setNotice(internalError(e));
      }
    },
    [library.tapes, t, tapeText]
  );

  // Il menu e i tasti dei Tape della Libreria e dei Recenti. Il Tape di Trascrivi su un Tape non
  // si rinomina, sposta né cestina (il backend risponderebbe `activityInProgress`).
  const tapeDellAttivita =
    transcribesSource(status) && source && isTape(source) ? source : null;
  const tapeOperations = useMemo<TapeOperations>(
    () => ({
      onCopy: copyTape,
      onMove: (tape, destinazione) => moveTape(tape.path, destinazione),
      onOpen: (tape) => openFromLibrary(tape.path),
      onRename: (tape, titolo) => renameTape(tape.path, titolo),
      onReveal: (tape) => reveal(tape.path),
      onTrash: requestTrash,
      tapeDellAttivita,
    }),
    [
      copyTape,
      moveTape,
      openFromLibrary,
      renameTape,
      requestTrash,
      reveal,
      tapeDellAttivita,
    ]
  );

  const exported = useCallback(
    (path: string) => setMessage(t("transcription.exported", { path })),
    [t]
  );

  // Il testo e le informazioni scritti nel Tape `path` valgono per ogni sua vista.
  const applyOpenedTape = useCallback(
    (path: string, { info: updated, parlanti, phrases }: OpenedTape) =>
      dispatch({
        change: (c) => ({ ...c, parlanti, phrases }),
        info: () => updated,
        path,
        type: "tapeChanged",
      }),
    [dispatch]
  );

  // Al ritorno in primo piano il backend riallinea l'indice; qui si ricarica il Tape mostrato se
  // un Assistente o Esplora file l'hanno cambiato, o si chiude se non c'è più. Una bozza dell'editor
  // del Turno resta: il suo salvataggio fallisce sul testo atteso.
  const refocused = useEffectEvent(async () => {
    const center = centerView(state);
    if (center.kind !== "tape") {
      return;
    }
    const { path } = center.tape;
    const suDisco = await commands.tapeModificato(path);
    if (suDisco.status === "error") {
      setNotice(suDisco.error);
      return;
    }
    const azione = alRitorno(
      center.tape.info,
      suDisco.data,
      center.isSource && busy
    );
    if (azione === "chiudi") {
      gone(path);
    } else if (azione === "ricarica") {
      const opened = await commands.openTape(path);
      if (opened.status === "error") {
        setNotice(opened.error);
      } else {
        applyOpenedTape(path, opened.data);
      }
    }
  });
  useEffect(() => {
    const unlisten = getCurrentWindow().onFocusChanged(({ payload }) => {
      if (payload) {
        refocused();
      }
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  const merge = useCallback(
    async (path: string, turn: Turn, target: PhraseRef) => {
      if ((await pendingEdits.current.get(path)) === false) {
        return false;
      }
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
        applyOpenedTape(path, opened.data);
        return true;
      } catch (e) {
        setNotice(internalError(e));
        return false;
      }
    },
    [applyOpenedTape]
  );

  const rename = useCallback(
    async (path: string, voce: Parlante, nome: string) => {
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
      // Il dato manuale viene mostrato soltanto dopo una scrittura riuscita.
      dispatch({
        change: (c) => withNome(c, voce.ingresso, voce.parlante, nome),
        info:
          nome.trim() === voce.nome
            ? undefined
            : (current) => current && { ...current, correttoAMano: true },
        path,
        type: "tapeChanged",
      });
    },
    [dispatch]
  );

  // La risposta contiene la proiezione realmente scritta; una bozza fallita resta nell'editor.
  const edit = useCallback(
    async (path: string, turn: Turn, original: string, text: string) => {
      const saving = (async () => {
        try {
          const result = await commands.editTurno(
            path,
            turn.ingresso,
            turn.items.map((item) => item.phraseId),
            original,
            text
          );
          if (result.status === "error") {
            setNotice(result.error);
            return false;
          }
          applyOpenedTape(path, result.data);
          return true;
        } catch (e) {
          setNotice(internalError(e));
          return false;
        }
      })();
      pendingEdits.current.set(path, saving);
      try {
        return await saving;
      } finally {
        if (pendingEdits.current.get(path) === saving) {
          pendingEdits.current.delete(path);
        }
      }
    },
    [applyOpenedTape]
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
      dispatch({
        info: (current) => current && { ...current, creato },
        path,
        type: "tapeChanged",
      });
    },
    [dispatch]
  );

  // Una Nota salvata riscrive il Tape: la data nuova non lo fa ricaricare al ritorno in primo piano.
  const noteSalvate = useCallback(
    (path: string, modificato: number | null) =>
      dispatch({
        info: (current) => current && { ...current, modificato },
        path,
        type: "tapeChanged",
      }),
    [dispatch]
  );

  const copyActivity = useCallback(() => copy(null), [copy]);

  // La Raccolta di un Tape dal suo percorso nella barra in alto: apre la Libreria lì.
  const openRaccolta = useCallback(
    (value: string) =>
      proteggi(() => {
        chooseRaccolta(value);
        dispatch({ type: "libraryShown" });
      }),
    [chooseRaccolta, dispatch, proteggi]
  );

  // L'avviso in cima: l'errore di un'operazione, un messaggio o l'esito dell'Attività.
  const statusBanner: Banner | null =
    dismissed === status ? null : bannerOf(status, t);
  const banner = shownBanner(
    notice,
    message,
    cleaningFailures.length && !cleaningDismissed
      ? {
          settings: false,
          text: cleaningFailures
            .map((failure) =>
              t("recording.cleaningFailed", {
                input: t(
                  `settings.recording.inputs.${failure.ingresso === "microfono" ? "mic" : "system"}`
                ),
              })
            )
            .join(" "),
          tone: "error",
        }
      : statusBanner,
    t
  );
  const dismiss = useCallback(() => {
    setNotice(null);
    setMessage(null);
    dispatch({ type: "cleaningDismissed" });
    setDismissed(status);
  }, [dispatch, status]);
  const dismissUpdate = useCallback(() => setUpdate(null), []);
  // Gli esiti spariscono da soli dopo un po'; gli errori restano finché non si chiudono.
  useEffect(() => {
    if (statusBanner?.tone !== "info") {
      return;
    }
    const timer = setTimeout(() => setDismissed(status), NOTICE_MS);
    return () => clearTimeout(timer);
  }, [status, statusBanner?.tone]);

  // La vista di un Tape; `isSource`: è la Sorgente, che Trascrivi trascrive.
  const tapePane = (view: TapeView, isSource: boolean) => (
    <TapePane
      bozze={bozzeDi(bozzeNote, view.path)}
      busy={busy}
      copied={copied}
      // Non ci lavora l'Attività in corso.
      editable={!(isSource && busy)}
      highlight={highlight}
      isSource={isSource}
      key={`${view.path}:${revision}`}
      library={library}
      onBozze={bozzeDispatch}
      onCopy={copy}
      onCreato={changeCreato}
      onDiarize={requestDiarization}
      onEdit={edit}
      onError={setNotice}
      onExported={exported}
      onMerge={merge}
      onMessage={setMessage}
      onMove={moveTape}
      onNoteSalvate={noteSalvate}
      onProteggi={proteggi}
      onRaccolta={openRaccolta}
      onRenameParlante={rename}
      onRenameTitle={renameTape}
      onReveal={reveal}
      onTagliato={applyOpenedTape}
      onTranscribe={requestTranscription}
      onTrash={requestTrash}
      processing={isSource && transcribesSource(status)}
      recording={recording}
      running={running}
      salvaNote={salvaNote}
      scarti={scarti}
      view={view}
    />
  );

  const bannerView = banner ? (
    <BannerView banner={banner} key={banner.text} onDismiss={dismiss} />
  ) : null;

  const center = shown;
  const selected = selection(state);
  const renderMainView = () => {
    switch (center.kind) {
      case "preparation":
        return (
          <>
            <TopBar crumbs={<Crumb current>{t("recording.title")}</Crumb>} />
            <div className="flex-1" />
            <Dock>
              <PreparationPanel
                cancelling={cancelling}
                onCancel={cancelRecordingStart}
                stage={center.stage}
              />
            </Dock>
          </>
        );
      case "library":
        return (
          <>
            <TopBar crumbs={<Crumb current>{t("library.title")}</Crumb>} />
            <AllTapes
              list={library}
              onError={setNotice}
              onMoved={moved}
              onRaccolta={chooseRaccolta}
              operations={tapeOperations}
              raccolta={raccolta}
            />
          </>
        );
      case "home":
        return (
          <LibraryHome
            banner={bannerView}
            busy={busy}
            lastPath={lastPath}
            loading={libraryLoading}
            onImport={pickFile}
            onOpen={openFromLibrary}
            onRecord={newRecording}
            onShowAll={showAll}
            tapes={library.tapes}
          />
        );
      case "live":
        return (
          <LiveView
            cleaningFailures={cleaningFailures}
            cleaningPreparing={cleaningPreparing}
            conversation={conversation}
            copied={copied}
            elapsedMs={elapsedMs}
            note={noteRegistrazione}
            onCopy={copyActivity}
            onError={setNotice}
            onMessage={setMessage}
            onPausedChange={setPaused}
            paused={paused}
            sessionId={"id" in session ? session.id : null}
            status={status}
          />
        );
      case "tape":
        return tapePane(center.tape, center.isSource);
      case "file":
        return (
          <FileView
            busy={busy}
            conversation={conversation}
            copied={copied}
            onCopy={copyActivity}
            onOpen={open}
            onTranscribe={requestTranscription}
            running={running}
            source={center.path}
          />
        );
      default:
        return center satisfies never;
    }
  };

  let recordingAnnouncement = "";
  if (preparing) {
    recordingAnnouncement = statusText(status, t);
  } else if (recording) {
    recordingAnnouncement = t(
      paused ? "status.recordingPaused" : "recording.started"
    );
  }

  return (
    <ScorciatoieProvider
      attivita={busy}
      coperta={settingsPage !== null}
      registrazione={recording}
    >
      <div className="flex h-screen" inert={settingsPage !== null}>
        <Sidebar
          activity={activitySummary(status, elapsedMs, t)}
          aperta={barra.aperta}
          busy={busy}
          cancelling={cancelling}
          list={library}
          onActivity={showActivity}
          onCancel={cancel}
          onError={setNotice}
          onHome={showHome}
          onImport={pickFile}
          onOpen={openFromLibrary}
          onRecord={newRecording}
          onShowAll={showAll}
          onToggle={barra.alterna}
          operations={tapeOperations}
          record={
            <RecordMenu
              disabled={busy}
              onError={failed}
              onRecord={newRecording}
            />
          }
          selected={selected.tape}
          showingAll={selected.library}
          showingHome={selected.home}
          update={
            update ? (
              <UpdateNotice onDismiss={dismissUpdate} update={update} />
            ) : null
          }
        />
        <main className="relative flex min-w-0 flex-1 flex-col">
          <SidebarToggle aperta={barra.aperta} onToggle={barra.alterna} />
          <BannerSlot value={bannerView}>{renderMainView()}</BannerSlot>
          <p aria-atomic="true" className="sr-only" role="status">
            {recordingAnnouncement}
          </p>
        </main>
      </div>
      <TranscribeDialog
        onError={failed}
        onOpenChange={closeConfirm}
        onTranscribe={transcribe}
        open={confirmTranscribe}
        target={transcribeTarget}
      />
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
      {noteDialog}
      {dialog}
      {settingsPage}
      <WindowControls />
      {dragged && !confirmationOpen ? (
        <DropVeil verdict={dropVerdict(dragged, busy)} />
      ) : null}
    </ScorciatoieProvider>
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

/**
 * La Registrazione in corso, o il suo completamento dopo Stop, con il testo dal vivo e le Note:
 * si scrivono fino allo Stop, e Ctrl+T vi mette il tempo della Registrazione (`elapsedMs`).
 */
function LiveView({
  cleaningFailures,
  cleaningPreparing,
  conversation,
  copied,
  elapsedMs,
  note,
  onCopy,
  onError,
  onMessage,
  onPausedChange,
  paused,
  sessionId,
  status,
}: {
  cleaningFailures: RecordingCleaningFailed[];
  cleaningPreparing: RecordingCleaningPreparing[];
  conversation: Conversation;
  copied: boolean;
  elapsedMs: number;
  note: NoteDelTape;
  onCopy: () => void;
  onError: (error: AppError) => void;
  onMessage: (text: string) => void;
  onPausedChange: (paused: boolean) => void;
  paused: boolean;
  sessionId: string | null;
  status: Status;
}) {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const [tab, setTab] = useState<DocumentTab>("transcript");
  const live = settings.trascrizioneDalVivo ?? false;
  const recording = status.phase === "recording";
  const header = (
    <>
      <DocumentHeader
        meta={live ? t("live.on") : null}
        title={t("live.title")}
      />
      <DocumentToolbar
        hasText={false}
        note={quanteNote(note)}
        onTab={setTab}
        parlanti={0}
        tab={tab}
      />
    </>
  );
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
      {tab === "note" ? (
        <section className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
          <div className="mx-auto flex w-full max-w-[46rem] flex-col px-10 pb-12">
            {header}
            {/* I tempi restano testo finché la Registrazione non è un Tape. */}
            <NotesTab
              bozza
              durataMs={0}
              note={note}
              onError={onError}
              onMessage={onMessage}
              player={{ positionMs: elapsedMs }}
              readOnly={!recording}
              scarti={0}
            />
          </div>
        </section>
      ) : (
        <TranscriptView
          conversation={conversation}
          empty={
            <p className="max-w-[34rem] py-6 text-muted-foreground leading-relaxed">
              {live ? t("live.waiting") : t("live.off")}
            </p>
          }
          header={header}
          parlanti={NO_PARLANTI}
        />
      )}
      {recording ? (
        <Dock>
          <RecordingPanel
            cleaningFailures={cleaningFailures}
            cleaningPreparing={cleaningPreparing}
            key={sessionId}
            onError={onError}
            onPausedChange={onPausedChange}
            paused={paused}
            sessionId={sessionId ?? undefined}
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
  onOpen,
  onTranscribe,
  running,
  source,
}: {
  busy: boolean;
  conversation: Conversation;
  copied: boolean;
  onCopy: () => void;
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
                <TranscribeButton disabled={busy} onClick={onTranscribe} />
              </div>
            )}
          </>
        }
        parlanti={NO_PARLANTI}
      />
    </>
  );
}

/**
 * La barra in alto del pannello centrale: si trascina come la barra del titolo di Windows. A
 * sinistra dove si è, a destra le azioni, prima dei pulsanti della finestra. Alta come quella della Home.
 */
function TopBar({
  actions,
  crumbs,
}: {
  actions?: React.ReactNode;
  crumbs?: React.ReactNode;
}) {
  const { t } = useTranslation();
  const banner = use(BannerSlot);
  return (
    <>
      <header
        className={`flex h-12 shrink-0 items-center gap-4 border-b ${SIDEBAR_TOGGLE_PADDING} ${WINDOW_CONTROLS_PADDING}`}
        data-tauri-drag-region
      >
        <nav
          aria-label={t("app.whereAmI")}
          className="flex min-w-0 flex-1 items-center gap-2 text-muted-foreground text-sm"
          data-tauri-drag-region
        >
          {crumbs}
        </nav>
        {actions ? (
          <div className="flex items-center gap-1.5">{actions}</div>
        ) : null}
      </header>
      {banner}
    </>
  );
}

/**
 * Un aggiornamento disponibile, in fondo alla barra laterale: resta finché non si chiude, ma non
 * riguarda la vista e non le sta sopra.
 */
function UpdateNotice({
  onDismiss,
  update,
}: {
  onDismiss: () => void;
  update: UpdateInfo;
}) {
  const { t } = useTranslation();
  const download = useCallback(
    () => commands.openUpdate(update.url),
    [update.url]
  );
  return (
    <div className="mx-3 mb-2 flex items-start gap-2 rounded-lg border border-sidebar-border bg-background/60 py-2 pr-2 pl-3 text-sm">
      <p className="min-w-0 flex-1 leading-relaxed" role="status">
        {t("status.updateAvailable", { version: update.version })}{" "}
        <button
          className="-my-0.5 inline-flex min-h-6 items-center whitespace-nowrap rounded-sm font-medium underline underline-offset-4 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
          onClick={download}
          type="button"
        >
          {t("status.updateDownload")}
        </button>
      </p>
      <button
        aria-label={t("app.dismiss")}
        className="flex size-6 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
        onClick={onDismiss}
        title={t("app.dismiss")}
        type="button"
      >
        <X className="size-4" />
      </button>
    </div>
  );
}

/** Senza Parlanti da rinominare: sempre lo stesso array, così i turni memoizzati non cambiano. */
const NO_PARLANTI: Parlante[] = [];

/** L'avviso della finestra, che ogni vista mette sotto la sua barra in alto. */
const BannerSlot = createContext<ReactNode>(null);

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

/**
 * L'avviso sotto la barra in alto, nel flusso: spinge giù il contenuto invece di coprirlo. Ha il link
 * alle Impostazioni quando serve.
 */
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
      className={`motion-safe:fade-in mx-auto mt-4 flex w-[min(46rem,calc(100%-4rem))] shrink-0 items-start gap-3 rounded-xl border bg-card px-4 py-3 text-sm motion-safe:animate-in ${
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

/**
 * La vista di un Tape: la barra in alto con il percorso, Copia testo e "…"; il documento con
 * testata e schede Trascrizione e Parlanti; in fondo il player con Segui l'audio, nascosto
 * mentre lo si trascrive. `editable`: non ci lavora l'Attività in corso; `isSource`: è la Sorgente, che
 * Trascrivi trascrive; `recording`: il player è disabilitato.
 */
function TapePane({
  bozze: bozzeDelTape,
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
  onMessage,
  onBozze,
  onMove,
  onNoteSalvate,
  onProteggi,
  onRaccolta,
  onRenameParlante,
  onRenameTitle,
  onReveal,
  onTagliato,
  onTranscribe,
  onTrash,
  isSource,
  processing,
  recording,
  running,
  salvaNote,
  scarti,
  view: { conversation, info, path },
}: {
  /** Le bozze delle Note del Tape, per nome. */
  bozze: Record<string, string>;
  busy: boolean;
  copied: boolean;
  editable: boolean;
  highlight: Trovato | null;
  library: LibraryList;
  onCopy: (path: string) => void;
  onCreato: (path: string, local: string) => void;
  onDiarize: () => void;
  onEdit: (
    path: string,
    turn: Turn,
    original: string,
    text: string
  ) => Promise<boolean>;
  onMerge: (path: string, turn: Turn, target: PhraseRef) => Promise<boolean>;
  onError: (error: AppError) => void;
  onExported: (path: string) => void;
  onMessage: (text: string) => void;
  onBozze: (action: BozzeAction) => void;
  onMove: (path: string, raccolta: string) => void;
  onNoteSalvate: (path: string, modificato: number | null) => void;
  /** Fa `vai` dopo aver salvato o scartato le bozze delle Note, se ce ne sono. */
  onProteggi: (vai: () => void) => void;
  onRaccolta: (raccolta: string) => void;
  onRenameParlante: (path: string, voce: Parlante, nome: string) => void;
  onRenameTitle: (path: string, titolo: string) => void;
  onReveal: (path: string) => void;
  /** Il documento del Tape `path` appena tagliato. */
  onTagliato: (path: string, opened: OpenedTape) => void;
  onTranscribe: () => void;
  onTrash: (tape: { path: string; titolo: string }) => void;
  isSource: boolean;
  processing: boolean;
  recording: boolean;
  running: boolean;
  /** Qui la vista registra come salvare le bozze delle sue Note. */
  salvaNote: RefObject<(() => Promise<boolean>) | null>;
  scarti: number;
  view: TapeView;
}) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<DocumentTab>("transcript");
  // Il Parlante di cui si sta scrivendo il nome nuovo; si annulla quando ci lavora un'Attività.
  const [renaming, setRenaming] = useState<Parlante | null>(null);
  if (renaming && !editable) {
    setRenaming(null);
  }
  // TapePane si ridisegna a ogni aggiornamento del player: i Parlanti restano gli stessi oggetti.
  const allParlanti = useMemo(
    () => parlantiOf(conversation, t),
    [conversation, t]
  );
  const parlanti = editable ? allParlanti : NO_PARLANTI;
  const transcribing = isSource && running;
  const durataMs = info?.durataMs ?? 0;
  const player = usePlayer(durataMs);
  // La Frase di un risultato della ricerca porta lì il player, senza avviarlo.
  const phrases = useRef(conversation.phrases);
  useEffect(() => {
    phrases.current = conversation.phrases;
  }, [conversation.phrases]);
  const { audio, move } = player;
  // Un risultato in una Nota apre la scheda Note su quella Nota.
  const { frase, nota } = trovato(highlight);
  useEffect(() => {
    if (nota !== undefined) {
      setTab("note");
      return;
    }
    const found = phrases.current.find(
      (p) => p.ingresso === frase?.ingresso && p.phraseId === frase.phraseId
    );
    if (found) {
      setTab("transcript");
      audio.current?.pause();
      move(found.inizioMs, "jump");
    }
  }, [audio, frase, move, nota]);
  const copy = useCallback(() => onCopy(path), [onCopy, path]);
  const rename = useCallback(
    (voce: Parlante, nome: string) => {
      setRenaming(null);
      onRenameParlante(path, voce, nome);
    },
    [onRenameParlante, path]
  );
  const edit = useCallback(
    (turn: Turn, original: string, text: string) =>
      onEdit(path, turn, original, text),
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
  const taglio = useTaglio({
    attiva: editable,
    info,
    onError,
    onTagliato,
    path,
    player,
  });
  const { chiedi } = taglio;
  const forbici = useMemo(
    () =>
      chiedi
        ? {
            durataMs: taglio.durataMs,
            onTaglia: (r: RichiestaTaglio) => onProteggi(() => chiedi(r)),
          }
        : null,
    [chiedi, onProteggi, taglio.durataMs]
  );
  const editingActions = editable ? { onEdit: edit, onMerge: merge } : {};
  const note = useNote({
    bozze: bozzeDelTape,
    onBozze,
    onError,
    onSaved: onNoteSalvate,
    path,
    scelta: nota,
  });
  const salvaTutte = note?.salvaTutte;
  useEffect(() => {
    if (!salvaTutte) {
      return;
    }
    salvaNote.current = salvaTutte;
    return () => {
      if (salvaNote.current === salvaTutte) {
        salvaNote.current = null;
      }
    };
  }, [salvaNote, salvaTutte]);

  const header = (
    <>
      <TapeHeader
        disabled={!editable}
        info={info}
        library={library}
        onCreato={onCreato}
        onRename={onRenameTitle}
        parlanti={allParlanti.length}
        path={path}
      />
      <DocumentToolbar
        hasText={hasText}
        note={quanteNote(note)}
        onTab={setTab}
        parlanti={parlanti.length}
        tab={tab}
      />
    </>
  );

  const empty = (
    <TapeEmpty
      busy={busy}
      onTranscribe={isSource ? onTranscribe : undefined}
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
              onDiarize={isSource ? onDiarize : undefined}
              onError={onError}
              onExported={onExported}
              onMove={onMove}
              onReveal={onReveal}
              onTranscribe={
                isSource && !busy && hasText ? onTranscribe : undefined
              }
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
      {tab === "transcript" ? (
        <TranscriptView
          conversation={conversation}
          empty={empty}
          header={header}
          highlight={frase}
          {...editingActions}
          onRename={rename}
          onRenaming={setRenaming}
          parlanti={parlanti}
          player={listening}
          renaming={renaming}
          taglio={forbici}
        />
      ) : (
        <section className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
          <div className="mx-auto flex w-full max-w-[46rem] flex-col px-10 pb-12">
            {header}
            {tab === "note" ? (
              <NotesTab
                durataMs={durataMs}
                note={note}
                onError={onError}
                onMessage={onMessage}
                player={listening}
                readOnly={!editable}
                scarti={scarti}
              />
            ) : (
              <ParlantiTab
                conversation={conversation}
                list={parlanti}
                onPlay={listening?.playFrom}
                onRename={rename}
                onRenaming={setRenaming}
                renaming={renaming}
              />
            )}
          </div>
        </section>
      )}
      <TapePlayer
        disabled={recording}
        key={taglio.versione}
        path={path}
        player={player}
        processing={processing}
        versione={taglio.versione}
      />
      {taglio.dialog}
    </>
  );
}

function TapePlayer({
  disabled,
  path,
  player,
  processing,
  versione,
}: {
  disabled: boolean;
  path: string;
  player: PlayerState;
  processing: boolean;
  /** Cresce quando il mix cambia (un Taglio): il player lo rilegge. */
  versione: number;
}) {
  return processing ? null : (
    <Dock>
      <Player
        disabled={disabled}
        path={path}
        player={player}
        versione={versione}
      />
    </Dock>
  );
}

/** Il documento di un Tape senza Frasi: in Trascrizione, o da trascrivere (se `onTranscribe`). */
function TapeEmpty({
  busy,
  onTranscribe,
  transcribing,
}: {
  busy: boolean;
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
        <TranscribeButton disabled={busy} onClick={onTranscribe} />
      ) : null}
    </div>
  );
}

type DocumentTab = "transcript" | "parlanti" | "note";

/** Le schede Trascrizione, Parlanti (se il Tape ha Frasi) e Note, con il numero delle Note. */
function DocumentToolbar({
  hasText,
  note,
  onTab,
  parlanti,
  tab,
}: {
  hasText: boolean;
  note: number;
  onTab: (tab: DocumentTab) => void;
  parlanti: number;
  tab: DocumentTab;
}) {
  const { t } = useTranslation();
  const toTranscript = useCallback(() => onTab("transcript"), [onTab]);
  const toParlanti = useCallback(() => onTab("parlanti"), [onTab]);
  const toNote = useCallback(() => onTab("note"), [onTab]);
  return (
    <div className="sticky top-0 z-10 mb-3 flex items-center gap-6 border-b bg-background">
      <div aria-label={t("tabs.label")} className="flex gap-6" role="tablist">
        <TabButton onClick={toTranscript} selected={tab === "transcript"}>
          {t("tabs.transcript")}
        </TabButton>
        {hasText ? (
          <TabButton onClick={toParlanti} selected={tab === "parlanti"}>
            {t("tabs.parlanti")}
            {parlanti > 0 ? (
              <span className="ml-1.5 text-muted-foreground tabular-nums">
                {parlanti}
              </span>
            ) : null}
          </TabButton>
        ) : null}
        <TabButton onClick={toNote} selected={tab === "note"}>
          {t("tabs.note")}
          {note > 0 ? (
            <span className="ml-1.5 text-muted-foreground tabular-nums">
              {note}
            </span>
          ) : null}
        </TabButton>
      </div>
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
  useScorciatoia("copiaTesto", disabled ? null : onCopy);
  return (
    <Button
      aria-keyshortcuts={ariaTasti("copiaTesto")}
      className="h-8 gap-2 bg-card"
      disabled={disabled}
      onClick={onCopy}
      title={conTasti(t, t("shortcuts.lines.copiaTesto"), "copiaTesto")}
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
