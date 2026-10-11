import { expect, test } from "bun:test";
import type { Ingresso } from "@/bindings";
import {
  autoScroll,
  type Follow,
  type FollowEvent,
  nextFollow,
  playingAt,
} from "@/features/player/sync";

const phrase = (
  phraseId: number,
  inizioMs: number,
  fineMs: number,
  ingresso: Ingresso = "mix"
) => ({ fineMs, ingresso, inizioMs, parlante: null, phraseId, text: "" });

const ids = (list: { ingresso: Ingresso; phraseId: number }[]) =>
  list.map((p) => `${p.ingresso}:${p.phraseId}`);

const mix = [
  phrase(0, 1000, 3000),
  phrase(1, 3000, 5000),
  phrase(2, 8000, 9000),
];

test("si evidenzia la Frase che contiene la posizione, fine esclusa", () => {
  expect(ids(playingAt(mix, 1500))).toEqual(["mix:0"]);
  expect(ids(playingAt(mix, 1000))).toEqual(["mix:0"]);
  expect(ids(playingAt(mix, 3000))).toEqual(["mix:1"]);
  expect(ids(playingAt(mix, 8999))).toEqual(["mix:2"]);
});

test("nel silenzio e dopo l'ultima resta la precedente, prima della prima nessuna", () => {
  expect(ids(playingAt(mix, 6000))).toEqual(["mix:1"]);
  expect(ids(playingAt(mix, 9000))).toEqual(["mix:2"]);
  expect(ids(playingAt(mix, 60_000))).toEqual(["mix:2"]);
  expect(playingAt(mix, 999)).toEqual([]);
  expect(playingAt([], 5000)).toEqual([]);
});

test("le Frasi sovrapposte si evidenziano tutte, la prima è quella iniziata prima", () => {
  // In ordine di inizio, come nella Conversation.
  const separati = [
    phrase(0, 1000, 6000, "microfono"),
    phrase(0, 2000, 4000, "sistema"),
    phrase(1, 7000, 8000, "sistema"),
  ];
  expect(ids(playingAt(separati, 3000))).toEqual(["microfono:0", "sistema:0"]);
  expect(ids(playingAt(separati, 5000))).toEqual(["microfono:0"]);
  // Nel silenzio, quella finita per ultima.
  expect(ids(playingAt(separati, 6500))).toEqual(["microfono:0"]);
});

test("Segui l'audio: lo scorrimento a mano lo sospende, salto, barra e comando lo riprendono", () => {
  const sequence: [FollowEvent, Follow][] = [
    ["scroll", "free"],
    ["scroll", "free"],
    ["seek", "following"],
    ["scroll", "free"],
    ["follow", "following"],
    ["scroll", "free"],
    ["jump", "following"],
  ];
  let state: Follow = "following";
  for (const [event, expected] of sequence) {
    state = nextFollow(event);
    expect(state).toBe(expected);
  }
});

test("durante la correzione di una Frase il testo non scorre da solo", () => {
  expect(autoScroll("following", false)).toBe(true);
  expect(autoScroll("following", true)).toBe(false);
  expect(autoScroll("free", false)).toBe(false);
});
