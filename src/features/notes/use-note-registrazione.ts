import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands } from "@/bindings";
import type { Session } from "@/features/activity/activity";
import {
  bozzaNote,
  type NotaAperta,
  nomeNuovo,
  nomeRifiutato,
  noteIniziali,
  salvataggio,
  senzaNota,
} from "./notes";
import type { NoteDelTape } from "./use-note";

interface Stato {
  aperta: string;
  note: NotaAperta[];
}

/**
 * Le Note scritte durante la Registrazione della `session` (ADR-0032), con le stesse operazioni di
 * quelle di un Tape. Vivono qui finché la sessione dura, e ogni modifica manda tutte le Note scritte
 * al backend, che le tiene nella bozza accanto all'Ogg e allo Stop le mette nel Tape: si salvano
 * sempre da sole. Gli invii vanno uno dopo l'altro e mandano le Note di quel momento. Una sessione
 * nuova riparte con una sola Nota, nuova.
 */
export function useNoteRegistrazione({
  onError,
  session,
}: {
  onError: (error: AppError) => void;
  session: Session;
}): NoteDelTape {
  const sessionId = "id" in session ? session.id : null;
  const { t } = useTranslation();
  const predefinito = t("notes.defaultName");
  const nuove = useCallback(
    (): Stato => ({
      aperta: predefinito,
      note: noteIniziali([], predefinito),
    }),
    [predefinito]
  );
  const [stato, setStato] = useState<Stato>(nuove);
  const attuale = useRef(stato);
  const coda = useRef<Promise<unknown>>(Promise.resolve());

  // biome-ignore lint/correctness/useExhaustiveDependencies: si riparte solo con una sessione nuova
  useEffect(() => {
    attuale.current = nuove();
    setStato(attuale.current);
  }, [sessionId]);

  const aggiorna = useCallback((cambia: (stato: Stato) => Stato) => {
    attuale.current = cambia(attuale.current);
    setStato(attuale.current);
  }, []);
  /** Manda le Note scritte: `true` se ora sono nella bozza. */
  const invia = useCallback(() => {
    const fatto = coda.current.then(async () => {
      if (sessionId === null) {
        return false;
      }
      const result = await commands.saveRecordingNotes(
        sessionId,
        bozzaNote(attuale.current.note)
      );
      if (result.status === "error") {
        onError(result.error);
        return false;
      }
      return result.data;
    });
    coda.current = fatto.catch(() => undefined);
    return fatto;
  }, [onError, sessionId]);

  const apri = useCallback(
    (aperta: string) => aggiorna((s) => ({ ...s, aperta })),
    [aggiorna]
  );

  const crea = useCallback(() => {
    const nomi = attuale.current.note.map((n) => n.nome);
    const nome = nomeNuovo(nomi, (n) => t("notes.newName", { n }));
    aggiorna((s) => ({
      aperta: nome,
      note: [...s.note, { nome, salvato: null }],
    }));
    return nome;
  }, [aggiorna, t]);

  const salva = useCallback(
    (nome: string, bozza: string) => {
      const nota = attuale.current.note.find((n) => n.nome === nome);
      // Rinominata o eliminata nel frattempo: la bozza era già salvata uscendo dal campo.
      if (!nota) {
        return Promise.resolve(false);
      }
      if (!salvataggio(nome, nota.salvato, bozza)) {
        return Promise.resolve(nota.salvato !== null);
      }
      aggiorna((s) => ({
        ...s,
        note: s.note.map((n) =>
          n.nome === nome ? { ...n, salvato: bozza } : n
        ),
      }));
      return invia();
    },
    [aggiorna, invia]
  );

  const rinomina = useCallback(
    (nome: string, nuovo: string) => {
      const { note } = attuale.current;
      const nota = note.find((n) => n.nome === nome);
      if (!nota || nuovo === nome) {
        return;
      }
      const rifiuto = nomeRifiutato(
        nuovo,
        note.filter((n) => n !== nota).map((n) => n.nome)
      );
      if (rifiuto) {
        onError(rifiuto);
        return;
      }
      aggiorna((s) => ({
        aperta: s.aperta === nome ? nuovo : s.aperta,
        note: s.note.map((n) => (n.nome === nome ? { ...n, nome: nuovo } : n)),
      }));
      if (nota.salvato !== null) {
        invia();
      }
    },
    [aggiorna, invia, onError]
  );

  const elimina = useCallback(
    (nome: string) => {
      const nota = attuale.current.note.find((n) => n.nome === nome);
      if (!nota) {
        return;
      }
      aggiorna((s) => {
        const resto = senzaNota(s.note, nome, predefinito);
        return {
          aperta: s.aperta === nome ? resto.aperta : s.aperta,
          note: resto.note,
        };
      });
      if (nota.salvato !== null) {
        invia();
      }
    },
    [aggiorna, invia, predefinito]
  );

  return useMemo(
    () => ({ ...stato, apri, crea, elimina, rinomina, salva }),
    [apri, crea, elimina, rinomina, salva, stato]
  );
}
