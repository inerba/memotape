import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import "../../src/app/global.css";
import i18n from "i18next";
import "../../src/lib/i18n";
import { TranscriptView } from "../../src/features/transcription/transcript-view";
import { TapeHeader } from "../../src/features/library/tape-header";
import { parlantiOf, type Parlante } from "../../src/features/transcription/phrases";

await i18n.changeLanguage("it");
function Smoke() {
  const [corrected, setCorrected] = useState(false);
  const [renaming, setRenaming] = useState<Parlante | null>(null);
  const [conversation, setConversation] = useState({
    parlanti: { "mix:1": "Mario", "mix:2": "Anna" },
    partials: [],
    phrases: Array.from({ length: 24 }, (_, phraseId) => ({
      phraseId, ingresso: "mix" as const, parlante: phraseId % 2 + 1,
      inizioMs: phraseId * 2000, fineMs: phraseId * 2000 + 1800,
      text: `Intervento ${phraseId + 1}. Questo testo si corregge sul posto e conserva il suo intervallo audio.`,
    })),
  });
  return <main className="flex h-screen flex-col bg-background text-foreground">
    <TranscriptView
      header={<TapeHeader disabled={false} info={{ correttoAMano: corrected, creato: "2026-10-06T10:00:00+02:00", durataMs: 48000, completa: true, modello: "Nemotron", linguaParlato: "it", origine: null, diarizzazione: null, ingressiSeparati: false }} library={{ tapes: [], raccolte: [], folder: "" } as any} path="Smoke.tape" parlanti={2} onRename={() => {}} onCreato={() => {}} />}
      conversation={conversation}
      parlanti={parlantiOf(conversation, i18n.t)}
      onEdit={async (item, text) => { setConversation(c => ({...c, phrases: c.phrases.map(p => p.phraseId === item.phraseId ? {...p, text} : p)})); setCorrected(true); return true; }}
      onMerge={async (turn, target) => { setConversation(c => ({...c, phrases: c.phrases.map(p => turn.items.some(item => item.phraseId === p.phraseId) ? {...p, parlante: c.phrases.find(p => p.phraseId === target.phraseId)!.parlante} : p)})); setCorrected(true); return true; }}
      onRename={(voce, nome) => { setConversation(c => ({...c, parlanti: {...c.parlanti, [`${voce.ingresso}:${voce.parlante}`]: nome}})); setRenaming(null); setCorrected(true); }}
      onRenaming={setRenaming}
      renaming={renaming}
    />
  </main>;
}
createRoot(document.getElementById("root")!).render(<Smoke />);
