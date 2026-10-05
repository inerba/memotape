import { expect, test } from "bun:test";
import { dropVerdict } from "@/features/source/drop";

test("un solo file audio, video o Tape si apre", () => {
  expect(dropVerdict(["C:\\audio\\Lezione 1.MP3"], false)).toEqual({
    accepted: true,
    path: "C:\\audio\\Lezione 1.MP3",
  });
  expect(dropVerdict(["D:\\video\\call.mkv"], false).accepted).toBe(true);
  expect(dropVerdict(["D:\\Sbobino\\Call.bino"], false).accepted).toBe(true);
});

test("più di un file si rifiuta, anche se tutti accettati", () => {
  expect(dropVerdict(["C:\\a.wav", "C:\\b.wav"], false)).toEqual({
    accepted: false,
    reason: "many",
  });
});

test("un formato non accettato o una cartella si rifiutano", () => {
  expect(dropVerdict(["C:\\note.docx"], false)).toEqual({
    accepted: false,
    reason: "format",
  });
  expect(dropVerdict(["C:\\Users\\me\\Musica"], false).accepted).toBe(false);
  expect(dropVerdict(["C:\\cartella.wav\\file"], false).accepted).toBe(false);
  expect(dropVerdict([], false).accepted).toBe(false);
});

test("durante un'Attività si apre solo un Tape", () => {
  expect(dropVerdict(["C:\\Call.bino"], true).accepted).toBe(true);
  expect(dropVerdict(["C:\\a.wav"], true)).toEqual({
    accepted: false,
    reason: "busy",
  });
  // Il formato sbagliato vale più dell'Attività: con l'Attività finita resterebbe sbagliato.
  expect(dropVerdict(["C:\\note.docx"], true)).toEqual({
    accepted: false,
    reason: "format",
  });
});
