import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import {
  afterDiarization,
  afterTranscription,
  bannerOf,
  needsSettings,
  type Status,
  statusText,
  withDiarizing,
  withLiveDiarizationError,
  withLiveError,
  withMovedSource,
  withProgress,
  withRecordingPhase,
} from "@/features/status/status";

const t = i18n.t.bind(i18n);

test("la Registrazione resta in preparazione fino alla conferma audio e conserva i guasti anticipati", () => {
  const preparing: Status = { phase: "preparingRecording", stage: "saving" };
  const error = {
    code: "liveTranscriptionUnavailable",
    detail: "ASR",
  } as const;
  const waiting = withLiveError(preparing, error);
  expect(waiting).toEqual({ ...preparing, liveError: error });
  expect(bannerOf(waiting, t)).toEqual({
    settings: true,
    text: t("errors.codes.liveTranscriptionUnavailable", { detail: "ASR" }),
    tone: "error",
  });
  expect(withRecordingPhase(waiting, "cleaning")).toEqual({
    liveError: error,
    phase: "preparingRecording",
    stage: "cleaning",
  });
  expect(withRecordingPhase(waiting, "recording")).toEqual({
    liveError: error,
    paused: false,
    phase: "recording",
  });
  const stopped: Status = { percent: null, phase: "completing" };
  expect(withRecordingPhase(stopped, "recording")).toBe(stopped);
  const cancelled: Status = { phase: "cancelled" };
  expect(withRecordingPhase(cancelled, "devices")).toBe(cancelled);
});

test("la Diarizzazione autonoma ha esiti propri e conserva il collegamento alle Impostazioni", () => {
  expect(statusText({ phase: "diarizing" }, t)).toBe(
    "Riconoscimento dei parlanti…"
  );
  expect(bannerOf({ phase: "diarizing" }, t)).toBeNull();
  expect(afterDiarization({ data: null, status: "ok" }, "Call.tape")).toEqual({
    path: "Call.tape",
    phase: "diarized",
  });
  const cancelled = afterDiarization(
    { error: { code: "cancelled" }, status: "error" },
    "Call.tape"
  );
  expect(statusText(cancelled, t)).toBe(
    "Diarizzazione annullata. Il Tape non è stato modificato."
  );
  expect(bannerOf(cancelled, t)?.tone).toBe("info");
  const missing = afterDiarization(
    { error: { code: "localDiarizerMissing" }, status: "error" },
    "Call.tape"
  );
  expect(bannerOf(missing, t)?.settings).toBe(true);
  expect(bannerOf(missing, t)?.tone).toBe("error");
});

test("il guasto dei Parlanti resta visibile dopo Stop e non maschera un guasto ASR", () => {
  const lag = { code: "liveDiarizationLagging" } as const;
  const stopped = withLiveDiarizationError(
    { percent: 30, phase: "completing" },
    lag
  );
  expect(statusText(stopped, t)).toContain(
    "parlanti dal vivo non tiene il passo"
  );
  expect(statusText(withProgress(stopped, 70), t)).toContain(
    "parlanti dal vivo non tiene il passo"
  );
  expect(statusText(withDiarizing(stopped), t)).toContain(
    "parlanti dal vivo non tiene il passo"
  );
  const failedAsr = withLiveError(
    { paused: false, phase: "recording" },
    { code: "liveTranscriptionUnavailable", detail: "ASR" }
  );
  expect(withLiveDiarizationError(failedAsr, lag)).toBe(failedAsr);
});

test("il modello locale assente o incompatibile mostra un errore esplicito con il link alle Impostazioni", () => {
  const absent: Status = {
    error: { code: "localDiarizerMissing" },
    phase: "failed",
  };
  expect(needsSettings(absent)).toBe(true);
  expect(statusText(absent, t)).toBe(
    "Scegli il modello locale Nemotron Diarization in Impostazioni → Trascrizione"
  );
  const invalid: Status = {
    error: { code: "localDiarizerIncompatible", detail: "D:\\modello.gguf" },
    phase: "failed",
  };
  expect(needsSettings(invalid)).toBe(true);
  expect(statusText(invalid, t)).toContain(
    "GGUF Nemotron Diarization verificato"
  );
  expect(statusText(invalid, t)).toContain("D:\\modello.gguf");
});

test("a riposo la status bar invita a scegliere un file o mostra il percorso della Sorgente", () => {
  expect(statusText({ phase: "idle", source: null }, t)).toBe(
    "Apri un file audio, video o Tape, o scegli un Tape della Libreria"
  );
  expect(
    statusText({ phase: "idle", source: "C:\\Lezioni\\Lezione 1.mp4" }, t)
  ).toBe("C:\\Lezioni\\Lezione 1.mp4");
});

test("durante la Trascrizione mostra la percentuale solo se la durata è nota", () => {
  expect(statusText({ percent: 42, phase: "transcribing" }, t)).toBe(
    "Trascrizione in corso… 42%"
  );
  expect(statusText({ percent: null, phase: "transcribing" }, t)).toBe(
    "Trascrizione in corso…"
  );
});

test("dopo la Trascrizione la status bar dice che riconosce i parlanti, senza percentuale", () => {
  const diarizing = withDiarizing({ percent: 100, phase: "transcribing" });
  expect(diarizing).toEqual({
    diarizing: true,
    percent: null,
    phase: "transcribing",
  });
  expect(statusText(diarizing, t)).toBe("Riconoscimento dei parlanti…");
  // Dopo Stop, finita la coda della Trascrizione dal vivo, anche la Registrazione.
  const completing = withDiarizing({ percent: 100, phase: "completing" });
  expect(completing).toEqual({
    diarizing: true,
    percent: null,
    phase: "completing",
  });
  expect(statusText(completing, t)).toBe("Analisi finale dei parlanti…");
  expect(withProgress(completing, 100)).toBe(completing);
  // Un evento in ritardo non riapre un'Attività finita.
  const cancelled: Status = { phase: "cancelled" };
  expect(withDiarizing(cancelled)).toBe(cancelled);
});

test("a fine Trascrizione mostra il Tape in cui è il testo", () => {
  const path = String.raw`C:\Memotape\Acme\Lezione 1.tape`;
  expect(statusText({ path, phase: "finished" }, t)).toBe(
    `Trascrizione salvata in ${path}`
  );
});

test("gli errori hanno un messaggio tradotto per codice", () => {
  expect(
    statusText(
      {
        error: { code: "unsupportedCodec", detail: "ac-3" },
        phase: "failed",
      },
      t
    )
  ).toBe("Il formato audio del file non è supportato: ac-3");
  expect(
    statusText(
      {
        error: {
          code: "unwritableFolder",
          detail: "D:\\x.txt: accesso negato",
        },
        phase: "failed",
      },
      t
    )
  ).toBe("Impossibile salvare nella cartella: D:\\x.txt: accesso negato");
});

test("il progresso aggiorna solo una Trascrizione in corso", () => {
  const running: Status = { percent: 10, phase: "transcribing" };
  expect(withProgress(running, 11)).toEqual({
    percent: 11,
    phase: "transcribing",
  });
  // Un evento in ritardo non riporta indietro una Trascrizione già finita.
  const finished: Status = { path: "a.tape", phase: "finished" };
  expect(withProgress(finished, 100)).toBe(finished);
});

test("una Trascrizione senza Frasi dice che non c'è parlato", () => {
  const status = afterTranscription({
    data: { outcome: "noSpeech" },
    status: "ok",
  });
  expect(status).toEqual({ phase: "noSpeech" });
  expect(statusText(status, t)).toBe("Nessun parlato rilevato");
});

test("una Trascrizione salvata porta il percorso del Tape", () => {
  expect(
    afterTranscription({
      data: { outcome: "saved", path: "a.tape" },
      status: "ok",
    })
  ).toEqual({ path: "a.tape", phase: "finished" });
});

test("Annulla non è un errore", () => {
  const status = afterTranscription({
    error: { code: "cancelled" },
    status: "error",
  });
  expect(status).toEqual({ phase: "cancelled" });
  expect(statusText(status, t)).toBe("Trascrizione annullata");
});

test("una seconda Attività è rifiutata con un messaggio dedicato", () => {
  const status = afterTranscription({
    error: { code: "activityInProgress" },
    status: "error",
  });
  expect(status.phase).toBe("failed");
  expect(statusText(status, t)).toBe(
    "Un'altra Attività è in corso: aspetta che finisca o annullala"
  );
});

test("dopo Stop il progresso della Trascrizione dal vivo diventa il completamento della trascrizione", () => {
  const completing = withProgress({ paused: false, phase: "recording" }, 40);
  expect(completing).toEqual({ percent: 40, phase: "completing" });
  expect(statusText(completing, t)).toBe(
    "Completamento della trascrizione… 40%"
  );
  expect(withProgress(completing, 41)).toEqual({
    percent: 41,
    phase: "completing",
  });
  // A Stop arriva subito un progresso indeterminato, anche col motore a metà Frase.
  const stopped = withProgress({ paused: false, phase: "recording" }, null);
  expect(statusText(stopped, t)).toBe("Completamento della trascrizione…");
});

test("senza modello la Registrazione continua e la status bar lo dice con il link alle Impostazioni", () => {
  const error = {
    code: "liveTranscriptionUnavailable",
    detail: "Nemotron",
  } as const;
  const recording = withLiveError({ paused: false, phase: "recording" }, error);
  expect(recording).toEqual({
    liveError: error,
    paused: false,
    phase: "recording",
  });
  expect(statusText(recording, t)).toBe(
    "Trascrizione dal vivo non disponibile: il modello Nemotron non è scaricato o non si carica (la Registrazione si salva comunque)"
  );
  expect(needsSettings(recording)).toBe(true);
  expect(needsSettings({ error, phase: "failed" })).toBe(true);
  expect(
    needsSettings({
      error: { code: "modelMissing", detail: "x" },
      phase: "failed",
    })
  ).toBe(true);
  expect(needsSettings({ paused: false, phase: "recording" })).toBe(false);
  // Riconosci i parlanti senza Sortformer: si scarica dalle Impostazioni.
  const diarizerMissing = {
    error: { code: "diarizerMissing", detail: "Sortformer 4spk v2.1" },
    phase: "failed",
  } as const;
  expect(needsSettings(diarizerMissing)).toBe(true);
  expect(statusText(diarizerMissing, t)).toBe(
    "Il modello per Riconosci i parlanti, Sortformer 4spk v2.1, non è scaricato"
  );
  // Finita la Registrazione l'avviso arriva con il suo esito, non con l'evento.
  const recorded: Status = { path: "a.ogg", phase: "recorded" };
  expect(withLiveError(recorded, error)).toBe(recorded);
});

test("un Tape aperto spostato o rinominato resta nella status bar con il percorso nuovo", () => {
  const from = "C:\\Memotape\\Call.tape";
  const to = "C:\\Memotape\\Acme\\Call.tape";
  expect(withMovedSource({ phase: "idle", source: from }, from, to)).toEqual({
    phase: "idle",
    source: to,
  });
  const other = { phase: "idle", source: "C:\\Altro.tape" } as const;
  expect(withMovedSource(other, from, to)).toEqual(other);
  expect(withMovedSource({ phase: "noSpeech" }, from, to)).toEqual({
    phase: "noSpeech",
  });
});

test("l'avviso dice errori ed esiti, non la fase di un'Attività in corso", () => {
  expect(bannerOf({ phase: "idle", source: null }, t)).toBeNull();
  expect(bannerOf({ percent: 10, phase: "transcribing" }, t)).toBeNull();
  expect(bannerOf({ paused: false, phase: "recording" }, t)).toBeNull();
  expect(bannerOf({ phase: "noSpeech" }, t)).toEqual({
    settings: false,
    text: "Nessun parlato rilevato",
    tone: "info",
  });
  const missing = bannerOf(
    { error: { code: "modelMissing", detail: "Nemotron" }, phase: "failed" },
    t
  );
  expect(missing?.tone).toBe("error");
  expect(missing?.settings).toBe(true);
  // La Trascrizione dal vivo si ferma, la Registrazione no: l'avviso porta alle Impostazioni.
  const live = bannerOf(
    {
      liveError: { code: "liveTranscriptionUnavailable", detail: "Nemotron" },
      paused: false,
      phase: "recording",
    },
    t
  );
  expect(live?.tone).toBe("error");
  expect(live?.settings).toBe(true);
});

test("gli avvisi dei due Ingressi restano distinti attraverso Stop e il guasto ASR ha priorità", () => {
  const lag = { code: "liveDiarizationLagging" } as const;
  let status: Status = { paused: false, phase: "recording" };
  status = withLiveDiarizationError(status, lag, "sistema");
  expect(statusText(status, t)).toContain("Audio di sistema:");
  expect(statusText(status, t)).not.toContain("Microfono:");
  status = withLiveDiarizationError(
    status,
    { code: "localDiarizerMissing" },
    "microfono"
  );
  expect(statusText(status, t)).toContain("Microfono:");
  expect(needsSettings(status)).toBe(true);
  status = withDiarizing(withProgress(status, null));
  expect(statusText(status, t)).toContain("Microfono:");
  expect(statusText(status, t)).toContain("Audio di sistema:");
  expect(bannerOf(status, t)?.settings).toBe(true);
  status = withLiveError(status, { code: "internal", detail: "ASR in errore" });
  expect(statusText(status, t)).toContain("ASR in errore");
});
