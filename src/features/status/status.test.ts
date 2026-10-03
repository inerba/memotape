import { expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import {
  afterTranscription,
  type Status,
  statusText,
  withProgress,
} from "@/features/status/status";

const t = i18n.t.bind(i18n);

test("a riposo la status bar invita a scegliere un file o mostra il percorso della Sorgente", () => {
  expect(statusText({ phase: "idle", source: null }, t)).toBe(
    "Scegli un file audio o video con Sfoglia"
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

test("a fine Trascrizione mostra i caratteri e il percorso del TXT", () => {
  const txtPath = "C:\\Lezioni\\Lezione 1 trascrizione 1.txt";
  expect(statusText({ chars: 12_345, phase: "finished", txtPath }, t)).toBe(
    `Trascrizione finita: 12.345 caratteri salvati in ${txtPath}`
  );
  expect(statusText({ chars: 1, phase: "finished", txtPath }, t)).toBe(
    `Trascrizione finita: 1 carattere salvato in ${txtPath}`
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
  const finished: Status = { chars: 3, phase: "finished", txtPath: "a.txt" };
  expect(withProgress(finished, 100)).toBe(finished);
});

test("una Trascrizione senza Frasi dice che non c'è parlato, senza TXT", () => {
  const status = afterTranscription({
    data: { outcome: "noSpeech" },
    status: "ok",
  });
  expect(status).toEqual({ phase: "noSpeech" });
  expect(statusText(status, t)).toBe("Nessun parlato rilevato");
});

test("una Trascrizione salvata porta caratteri e percorso del TXT", () => {
  expect(
    afterTranscription({
      data: { chars: 3, outcome: "saved", txtPath: "a.txt" },
      status: "ok",
    })
  ).toEqual({ chars: 3, phase: "finished", txtPath: "a.txt" });
});

test("Annulla non è un errore: la status bar dice che il TXT non c'è", () => {
  const status = afterTranscription({
    error: { code: "cancelled" },
    status: "error",
  });
  expect(status).toEqual({ phase: "cancelled" });
  expect(statusText(status, t)).toBe(
    "Trascrizione annullata: nessun TXT salvato"
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
