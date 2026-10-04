import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import {
  afterTranscription,
  needsSettings,
  type Status,
  statusText,
  withDiarizing,
  withLiveError,
  withMovedSource,
  withProgress,
} from "@/features/status/status";

const t = i18n.t.bind(i18n);

test("a riposo la status bar invita a scegliere un file o mostra il percorso della Sorgente", () => {
  expect(statusText({ phase: "idle", source: null }, t)).toBe(
    "Apri un file audio, video o Bino, o scegli un Bino della Libreria"
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
  expect(statusText(completing, t)).toBe("Riconoscimento dei parlanti…");
  expect(withProgress(completing, 100)).toBe(completing);
  // Un evento in ritardo non riapre un'Attività finita.
  const cancelled: Status = { phase: "cancelled" };
  expect(withDiarizing(cancelled)).toBe(cancelled);
});

test("a fine Trascrizione mostra i caratteri e il percorso del Markdown", () => {
  const mdPath = "C:\\Lezioni\\Lezione 1 trascrizione 1.md";
  expect(statusText({ chars: 12_345, mdPath, phase: "finished" }, t)).toBe(
    `Trascrizione finita: 12.345 caratteri salvati in ${mdPath}`
  );
  expect(statusText({ chars: 1, mdPath, phase: "finished" }, t)).toBe(
    `Trascrizione finita: 1 carattere salvato in ${mdPath}`
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
  const finished: Status = { chars: 3, mdPath: "a.md", phase: "finished" };
  expect(withProgress(finished, 100)).toBe(finished);
});

test("una Trascrizione senza Frasi dice che non c'è parlato, senza Markdown", () => {
  const status = afterTranscription({
    data: { outcome: "noSpeech" },
    status: "ok",
  });
  expect(status).toEqual({ phase: "noSpeech" });
  expect(statusText(status, t)).toBe("Nessun parlato rilevato");
});

test("una Trascrizione salvata porta caratteri e percorso del Markdown", () => {
  expect(
    afterTranscription({
      data: { chars: 3, mdPath: "a.md", outcome: "saved" },
      status: "ok",
    })
  ).toEqual({ chars: 3, mdPath: "a.md", phase: "finished" });
});

test("Annulla non è un errore: la status bar dice che il Markdown non c'è", () => {
  const status = afterTranscription({
    error: { code: "cancelled" },
    status: "error",
  });
  expect(status).toEqual({ phase: "cancelled" });
  expect(statusText(status, t)).toBe(
    "Trascrizione annullata: nessun Markdown salvato"
  );
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

test("un Bino aperto spostato o rinominato resta nella status bar con il percorso nuovo", () => {
  const from = "C:\\Sbobino\\Call.bino";
  const to = "C:\\Sbobino\\Acme\\Call.bino";
  expect(withMovedSource({ phase: "idle", source: from }, from, to)).toEqual({
    phase: "idle",
    source: to,
  });
  const other = { phase: "idle", source: "C:\\Altro.bino" } as const;
  expect(withMovedSource(other, from, to)).toEqual(other);
  expect(withMovedSource({ phase: "noSpeech" }, from, to)).toEqual({
    phase: "noSpeech",
  });
});
