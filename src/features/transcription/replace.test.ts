import { expect, test } from "bun:test";
import {
  replaceDescription,
  transcribeNeedsConfirm,
} from "@/features/transcription/replace";

const bino = "C:SbobinoRegistrazione.bino";

test("Trascrivi chiede conferma con testo nell'area, e su un Bino sempre", () => {
  expect(transcribeNeedsConfirm("", "D:/a.mp3")).toBe(false);
  expect(transcribeNeedsConfirm("  \n", "D:/a.mp3")).toBe(false);
  expect(transcribeNeedsConfirm("Ciao.", "D:/a.mp3")).toBe(true);
  // La ritrascrizione sostituisce il testo dentro il Bino, anche se l'area è vuota.
  expect(transcribeNeedsConfirm("", bino)).toBe(true);
  expect(transcribeNeedsConfirm("", null)).toBe(false);
});

test("la conferma dice cosa viene sostituito", () => {
  expect(replaceDescription("open", "D:/a.mp3")).toBe(
    "transcription.replace.descriptionOpen"
  );
  expect(replaceDescription("transcribe", bino)).toBe(
    "transcription.replace.descriptionBino"
  );
  expect(replaceDescription("transcribe", "D:/a.mp3")).toBe(
    "transcription.replace.description"
  );
  // La Registrazione crea un Bino nuovo: quello aperto non cambia.
  expect(replaceDescription("record", bino)).toBe(
    "transcription.replace.description"
  );
});
