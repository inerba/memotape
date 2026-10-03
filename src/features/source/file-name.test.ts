import { expect, test } from "bun:test";
import { fileName, isBino } from "@/features/source/file-name";

test("il nome della Sorgente è l'ultimo segmento del percorso Windows", () => {
  expect(fileName("C:\\Users\\me\\Lezione 1.mp3")).toBe("Lezione 1.mp3");
  expect(fileName("D:/audio/intervista.wav")).toBe("intervista.wav");
});

test("un Bino si riconosce dall'estensione, maiuscole comprese", () => {
  expect(isBino("C:\\Sbobino\\Registrazione.bino")).toBe(true);
  expect(isBino("D:/a/RIUNIONE.BINO")).toBe(true);
  expect(isBino("D:/a/Registrazione.ogg")).toBe(false);
  expect(isBino("D:/a.bino/intervista.wav")).toBe(false);
});
