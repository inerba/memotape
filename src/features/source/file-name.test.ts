import { expect, test } from "bun:test";
import {
  fileName,
  folderOf,
  isBino,
  movedPath,
} from "@/features/source/file-name";

test("il nome della Sorgente è l'ultimo segmento del percorso Windows", () => {
  expect(fileName("C:\\Users\\me\\Lezione 1.mp3")).toBe("Lezione 1.mp3");
  expect(fileName("D:/audio/intervista.wav")).toBe("intervista.wav");
  expect(folderOf("C:\\Sbobino\\Acme\\Call.bino")).toBe("C:\\Sbobino\\Acme");
  expect(folderOf("D:/audio/intervista.wav")).toBe("D:/audio");
});

test("un Bino si riconosce dall'estensione, maiuscole comprese", () => {
  expect(isBino("C:\\Sbobino\\Registrazione.bino")).toBe(true);
  expect(isBino("D:/a/RIUNIONE.BINO")).toBe(true);
  expect(isBino("D:/a/Registrazione.ogg")).toBe(false);
  expect(isBino("D:/a.bino/intervista.wav")).toBe(false);
});

test("un percorso segue il Bino o la cartella spostati o rinominati", () => {
  const call = "C:\\Sbobino\\Acme\\Call.bino";
  expect(movedPath(call, call, "C:\\Sbobino\\Call.bino")).toBe(
    "C:\\Sbobino\\Call.bino"
  );
  // La Raccolta rinominata, scritta anche con altre maiuscole.
  expect(movedPath(call, "c:\\sbobino\\acme", "C:\\Sbobino\\Acme Srl")).toBe(
    "C:\\Sbobino\\Acme Srl\\Call.bino"
  );
  expect(movedPath(call, "C:\\Sbobino\\Ac", "C:\\Sbobino\\X")).toBe(call);
  expect(movedPath(call, "C:\\Sbobino\\Altro.bino", "C:\\X.bino")).toBe(call);
});
