import { expect, test } from "bun:test";
import {
  fileName,
  folderOf,
  isTape,
  movedPath,
} from "@/features/source/file-name";

test("il nome della Sorgente è l'ultimo segmento del percorso Windows", () => {
  expect(fileName("C:\\Users\\me\\Lezione 1.mp3")).toBe("Lezione 1.mp3");
  expect(fileName("D:/audio/intervista.wav")).toBe("intervista.wav");
  expect(folderOf("C:\\Memotape\\Acme\\Call.tape")).toBe("C:\\Memotape\\Acme");
  expect(folderOf("D:/audio/intervista.wav")).toBe("D:/audio");
});

test("un Tape si riconosce dall'estensione, maiuscole comprese", () => {
  expect(isTape("C:\\Memotape\\Registrazione.tape")).toBe(true);
  expect(isTape("D:/a/RIUNIONE.TAPE")).toBe(true);
  expect(isTape("D:/a/Registrazione.ogg")).toBe(false);
  expect(isTape("D:/a.tape/intervista.wav")).toBe(false);
});

test("un percorso segue il Tape o la cartella spostati o rinominati", () => {
  const call = "C:\\Memotape\\Acme\\Call.tape";
  expect(movedPath(call, call, "C:\\Memotape\\Call.tape")).toBe(
    "C:\\Memotape\\Call.tape"
  );
  // La Raccolta rinominata, scritta anche con altre maiuscole.
  expect(movedPath(call, "c:\\memotape\\acme", "C:\\Memotape\\Acme Srl")).toBe(
    "C:\\Memotape\\Acme Srl\\Call.tape"
  );
  expect(movedPath(call, "C:\\Memotape\\Ac", "C:\\Memotape\\X")).toBe(call);
  expect(movedPath(call, "C:\\Memotape\\Altro.tape", "C:\\X.tape")).toBe(call);
});
