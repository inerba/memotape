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
  expect(folderOf("C:\\Memotape\\Acme\\Call.bino")).toBe("C:\\Memotape\\Acme");
  expect(folderOf("D:/audio/intervista.wav")).toBe("D:/audio");
});

test("un Tape si riconosce dall'estensione, maiuscole comprese", () => {
  expect(isTape("C:\\Memotape\\Registrazione.bino")).toBe(true);
  expect(isTape("D:/a/RIUNIONE.BINO")).toBe(true);
  expect(isTape("D:/a/Registrazione.ogg")).toBe(false);
  expect(isTape("D:/a.bino/intervista.wav")).toBe(false);
});

test("un percorso segue il Tape o la cartella spostati o rinominati", () => {
  const call = "C:\\Memotape\\Acme\\Call.bino";
  expect(movedPath(call, call, "C:\\Memotape\\Call.bino")).toBe(
    "C:\\Memotape\\Call.bino"
  );
  // La Raccolta rinominata, scritta anche con altre maiuscole.
  expect(movedPath(call, "c:\\memotape\\acme", "C:\\Memotape\\Acme Srl")).toBe(
    "C:\\Memotape\\Acme Srl\\Call.bino"
  );
  expect(movedPath(call, "C:\\Memotape\\Ac", "C:\\Memotape\\X")).toBe(call);
  expect(movedPath(call, "C:\\Memotape\\Altro.bino", "C:\\X.bino")).toBe(call);
});
