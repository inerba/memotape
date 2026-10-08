import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { TFunction } from "i18next";
import { Check, CircleAlert, Copy, X } from "lucide-react";
import {
  useCallback,
  useEffect,
  useEffectEvent,
  useId,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useOutlet } from "react-router";
import { centerView, selection, useHomeState } from "@/app/routes/home-state";
import {
  type AppError,
  commands,
  events,
  type LibraryList,
  type OpenedTape,
  type RecordingCleaningFailed,
  type RecordingCleaningPreparing,
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
import { Switch } from "@/components/ui/switch";
import { WindowControls } from "@/components/window-controls";
import { AllTapes } from "@/features/library/all-tapes";
import { chosenRaccolta, raccoltaLabel } from "@/features/library/library";
import { LibraryHome } from "@/features/library/library-home";
import { Sidebar } from "@/features/library/sidebar";
import { TapeMenu } from "@/features/library/tape-actions";
import {
  DocumentHeader,
  entryOf,
  TapeHeader,
  titleOf,
} from "@/features/library/tape-header";
import { useLastTape } from "@/features/library/use-last-tape";
import { useLibrary } from "@/features/library/use-library";
import { useTapeOperations } from "@/features/library/use-tape-operations";
import {
  keepFocus,
  Player,
  type PlayerState,
  usePlayer,
} from "@/features/player/player";
import { PreparationPanel } from "@/features/recording/preparation-panel";
import { RecordMenu } from "@/features/recording/record-menu";
import {
  activitySummary,
  afterRecording,
} from "@/features/recording/recording";
import { RecordingPanel } from "@/features/recording/recording-panel";
import { recordAfterSettings } from "@/features/recording/start";
import { SWITCH_CLASS } from "@/features/settings/setting-switch";
import { nomeMicrofonoRegistrazione } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { dropVerdict } from "@/features/source/drop";
import { DropVeil } from "@/features/source/drop-veil";
import { fileName, isTape } from "@/features/source/file-name";
import { opening, pendingTape, type TapeView } from "@/features/source/view";
import {
  afterDiarization,
  afterTranscription,
  type Banner,
  bannerOf,
  errorText,
  isBusy,
  type Status,
  statusText,
} from "@/features/status/status";
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
import { TranscribeMenu } from "@/features/transcription/transcribe-menu";
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
  const {
    cleaningDismissed,
    cleaningFailures,
    cleaningPreparing,
    conversation,
    dispatch,
    elapsedMs,
    highlight,
    info,
    ready,
    revision,
    session,
    source,
    state,
    status,
  } = useHomeState(
    loadError
      ? { error: loadError, phase: "failed" }
      : { phase: "idle", source: null },
    recordingStarted
  );
  // L'errore di un'operazione su un Tape o sulla Libreria, o un avviso (il Markdown esportato):
  // l'avviso lo mostra per un po' sopra la fase, che durante un'Attività non deve cambiare.
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
  // I file trascinati da Esplora file sopra la finestra.
  const [dragged, setDragged] = useState<string[] | null>(null);
  const { lastPath, remember, move: moveLastTape, forget } = useLastTape();
  const running = status.phase === "transcribing";
  const diarizing = status.phase === "diarizing";
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
    async (path: string, phrase?: PhraseRef, restart = false) => {
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
    async (path: string, phrase?: PhraseRef) => {
      const error = await loadTape(path, phrase);
      if (error) {
        dispatch({ status: { error, phase: "failed" }, type: "status" });
      }
    },
    [dispatch, loadTape]
  );

  // Durante un'Attività un Tape della Libreria si consulta accanto, senza toccarla.
  const browse = useCallback(
    async (path: string, phrase?: PhraseRef) => {
      const result = await commands.openTape(path);
      if (result.status === "error") {
        setNotice(result.error);
        return;
      }
      remember(path);
      dispatch({ path, phrase, tape: result.data, type: "tapeBrowsed" });
    },
    [dispatch, remember]
  );

  // Un Tape della barra laterale, della Libreria o della ricerca (`fromLibrary`, con la Frase
  // trovata), o un percorso di Apri file, del doppio clic o rilasciato: la vista della Sorgente
  // dice cosa leggere, qui lo si legge.
  const requestOpen = useCallback(
    (path: string, phrase?: PhraseRef, fromLibrary = false) => {
      const how = opening(state.view, status, path, fromLibrary);
      dispatch({ fromLibrary, path, phrase, type: "openRequested" });
      if (how === "restart") {
        remember(path);
      } else if (how === "browse") {
        browse(path, phrase);
      } else if (how === "source" && isTape(path)) {
        openTape(path, phrase);
      } else if (how === "source") {
        dispatch({ path, type: "sourceOpened" });
      }
    },
    [browse, dispatch, openTape, remember, state.view, status]
  );

  const pickFile = useCallback(async () => {
    const picked = await commands.pickSource(t("source.filter"));
    if (picked) {
      requestOpen(picked);
    }
  }, [requestOpen, t]);

  const openFromLibrary = useCallback(
    (path: string, phrase?: PhraseRef) => requestOpen(path, phrase, true),
    [requestOpen]
  );

  const showActivity = useCallback(
    () => dispatch({ type: "activityShown" }),
    [dispatch]
  );
  const showAll = useCallback(
    () => dispatch({ type: "libraryShown" }),
    [dispatch]
  );
  const showHome = useCallback(
    () => dispatch({ type: "homeShown" }),
    [dispatch]
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
    },
    [dispatch, moveLastTape]
  );

  // Il Tape nel Cestino, se era aperto, non lo è più; la vista di un'Attività però resta.
  const trashed = useCallback(
    (path: string) => {
      forget(path);
      dispatch({ path, type: "tapeTrashed" });
    },
    [dispatch, forget]
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
  const tapeToOpen = pendingTape(state.view, status, confirmationOpen);
  useEffect(() => {
    if (tapeToOpen) {
      navigate("/");
      requestOpen(tapeToOpen);
    }
  }, [navigate, requestOpen, tapeToOpen]);

  // Un file rilasciato si apre come con Apri file, anche da Impostazioni; durante un'Attività solo
  // un Tape, in consultazione. Con la conferma di Trascrivi aperta il rilascio non conta.
  const dropped = useEffectEvent((paths: string[]) => {
    const verdict = dropVerdict(paths, busy);
    if (verdict.accepted && !confirmationOpen) {
      navigate("/");
      requestOpen(verdict.path);
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
  }, [dispatch, loadTape, raccolta, source]);

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
        dispatch({ path: after.source, type: "sourceOpened" });
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
  const copy = useCallback(
    async (path: string | null) => {
      try {
        let text: string | null;
        if (path) {
          if ((await pendingEdits.current.get(path)) === false) {
            return;
          }
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

  const copyActivity = useCallback(() => copy(null), [copy]);

  // La Raccolta di un Tape dal suo percorso nella barra in alto: apre la Libreria lì.
  const openRaccolta = useCallback(
    (value: string) => {
      chooseRaccolta(value);
      dispatch({ type: "libraryShown" });
    },
    [chooseRaccolta, dispatch]
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
      : (statusBanner ?? updateBanner(update, t)),
    t
  );
  const dismiss = useCallback(() => {
    setNotice(null);
    setMessage(null);
    dispatch({ type: "cleaningDismissed" });
    setDismissed(status);
    if (banner?.updateUrl) {
      setUpdate(null);
    }
  }, [banner, dispatch, status]);
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
      onReveal={reveal}
      onTranscribe={requestTranscription}
      onTrash={requestTrash}
      own={own}
      processing={own && (running || diarizing)}
      recording={recording}
      running={running}
      view={view}
    />
  );

  const center = centerView(state);
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
              onMove={moveTape}
              onMoved={moved}
              onOpen={openFromLibrary}
              onRaccolta={chooseRaccolta}
              onTrash={requestTrash}
              raccolta={raccolta}
            />
          </>
        );
      case "home":
        return (
          <LibraryHome
            busy={busy}
            lastPath={lastPath}
            loading={libraryLoading}
            onImport={pickFile}
            onOpen={openFromLibrary}
            onRecord={record}
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
            onCopy={copyActivity}
            onError={setNotice}
            onPausedChange={setPaused}
            paused={paused}
            sessionId={"id" in session ? session.id : null}
            status={status}
          />
        );
      case "tape":
        return tapePane(center.tape, center.own);
      case "file":
        return (
          <FileView
            busy={busy}
            conversation={conversation}
            copied={copied}
            onCopy={copyActivity}
            onError={failed}
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
          onHome={showHome}
          onImport={pickFile}
          onOpen={openFromLibrary}
          onShowAll={showAll}
          record={
            <RecordMenu disabled={busy} onError={failed} onRecord={record} />
          }
          selected={selected.tape}
          showingAll={selected.library}
          showingHome={selected.home}
        />
        <main className="relative flex min-w-0 flex-1 flex-col">
          {renderMainView()}
          <p aria-atomic="true" className="sr-only" role="status">
            {recordingAnnouncement}
          </p>
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
  cleaningFailures,
  cleaningPreparing,
  conversation,
  copied,
  onCopy,
  onError,
  onPausedChange,
  paused,
  sessionId,
  status,
}: {
  cleaningFailures: RecordingCleaningFailed[];
  cleaningPreparing: RecordingCleaningPreparing[];
  conversation: Conversation;
  copied: boolean;
  onCopy: () => void;
  onError: (error: AppError) => void;
  onPausedChange: (paused: boolean) => void;
  paused: boolean;
  sessionId: string | null;
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
  onReveal,
  onTranscribe,
  onTrash,
  own,
  processing,
  recording,
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
  onEdit: (
    path: string,
    turn: Turn,
    original: string,
    text: string
  ) => Promise<boolean>;
  onMerge: (path: string, turn: Turn, target: PhraseRef) => Promise<boolean>;
  onError: (error: AppError) => void;
  onExported: (path: string) => void;
  onMove: (path: string, raccolta: string) => void;
  onRaccolta: (raccolta: string) => void;
  onRenameParlante: (path: string, voce: Parlante, nome: string) => void;
  onRenameTitle: (path: string, titolo: string) => void;
  onReveal: (path: string) => void;
  onTranscribe: () => void;
  onTrash: (tape: { path: string; titolo: string }) => void;
  own: boolean;
  processing: boolean;
  recording: boolean;
  running: boolean;
  view: TapeView;
}) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<"transcript" | "parlanti">("transcript");
  // Il Parlante di cui si sta scrivendo il nome nuovo; si annulla quando ci lavora un'Attività.
  const [renaming, setRenaming] = useState<Parlante | null>(null);
  if (renaming && !editable) {
    setRenaming(null);
  }
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
              onRenaming={setRenaming}
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
          onRenaming={setRenaming}
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
  const followId = useId();
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
        <label
          className="flex cursor-pointer items-center gap-2.5 py-1 text-sm"
          htmlFor={followId}
        >
          <Switch
            checked={following}
            className={SWITCH_CLASS}
            id={followId}
            onCheckedChange={toggleFollow}
            onPointerDown={keepFocus}
          />
          {t("player.follow")}
        </label>
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
