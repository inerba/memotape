import { expect, test } from "bun:test";
import { fileName } from "@/features/source/file-name";

test("il nome della Sorgente è l'ultimo segmento del percorso Windows", () => {
  expect(fileName("C:\\Users\\me\\Lezione 1.mp3")).toBe("Lezione 1.mp3");
  expect(fileName("D:/audio/intervista.wav")).toBe("intervista.wav");
});
