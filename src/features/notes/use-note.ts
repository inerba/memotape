import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands } from "@/bindings";
import {
  apertaIniziale,
  type NotaAperta,
  nomeNuovo,
  nomeRifiutato,
  noteIniziali,
  salvataggio,
  senzaNota,
} from "./notes";

interface Stato {
  aperta: string;
  note: NotaAperta[];
}

/** Le Note del Tape aperto e le operazioni su di loro. */
export interface NoteDelTape extends Stato {
  apri: (nome: string) => void;
  /** Una Nota nuova, «Nota 2», «Nota 3»…, aperta: nasce su disco con il primo carattere. */
  crea: () => string;
  elimina: (nome: string) => void;
  /** Rinomina la Nota; un nome rifiutato va a `onError`. */
  rinomina: (nome: string, nuovo: string) => void;
  /** Salva la bozza della Nota: `true` se ora su disco c'è la bozza. */
  salva: (nome: string, bozza: string) => Promise<boolean>;
}

/** Quante Note ha il Tape su disco, per la scheda: le nuove ancora vuote non contano. */
export function quanteNote(note: NoteDelTape | null): number {
  return note?.note.filter((n) => n.salvato !== null).length ?? 0;
}

/**
 * Le Note del Tape `path` (ADR-0032), lette al montaggio, con la Nota aperta. Vive quanto la vista
 * del Tape, non quanto la scheda: le Note nuove e ancora vuote restano passando da una scheda
 * all'altra e spariscono uscendo dal Tape, perché su disco non ci sono. Salvataggi, rinomine ed
 * eliminazioni vanno uno dopo l'altro in una coda, e ognuno parte dalle Note lasciate dal
 * precedente; `onSaved` riceve la data di modifica nuova del Tape. `scelta` (la Nota di un risultato
 * della ricerca) si apre appena c'è. `null` finché non sono lette.
 */
export function useNote({
  onError,
  onSaved,
  path,
  scelta,
}: {
  onError: (error: AppError) => void;
  onSaved: (path: string, modificato: number | null) => void;
  path: string;
  scelta?: string;
}): NoteDelTape | null {
  const { t } = useTranslation();
  const [stato, setStato] = useState<Stato | null>(null);
  // Lo stato vero, aggiornato subito: la coda non aspetta che React ridisegni.
  const attuale = useRef<Stato | null>(null);
  const coda = useRef<Promise<unknown>>(Promise.resolve());
  const predefinito = t("notes.defaultName");
  const sceltaAllaLettura = useRef(scelta);
  sceltaAllaLettura.current = scelta;

  const aggiorna = useCallback((cambia: (stato: Stato) => Stato) => {
    if (attuale.current) {
      attuale.current = cambia(attuale.current);
      setStato(attuale.current);
    }
  }, []);
  const inCoda = useCallback(<T>(lavoro: () => Promise<T>): Promise<T> => {
    const fatto = coda.current.then(lavoro);
    coda.current = fatto.catch(() => undefined);
    return fatto;
  }, []);

  useEffect(() => {
    let vivo = true;
    commands.tapeNote(path).then((result) => {
      if (!vivo) {
        return;
      }
      if (result.status === "error") {
        onError(result.error);
        return;
      }
      const note = noteIniziali(result.data, predefinito);
      attuale.current = {
        aperta: apertaIniziale(note, sceltaAllaLettura.current),
        note,
      };
      setStato(attuale.current);
    });
    return () => {
      vivo = false;
    };
  }, [onError, path, predefinito]);

  const apri = useCallback(
    (aperta: string) => aggiorna((s) => ({ ...s, aperta })),
    [aggiorna]
  );
  useEffect(() => {
    if (scelta && attuale.current?.note.some((n) => n.nome === scelta)) {
      apri(scelta);
    }
  }, [apri, scelta]);

  const crea = useCallback(() => {
    const nomi = attuale.current?.note.map((n) => n.nome) ?? [];
    const nome = nomeNuovo(nomi, (n) => t("notes.newName", { n }));
    aggiorna((s) => ({
      aperta: nome,
      note: [...s.note, { nome, salvato: null }],
    }));
    return nome;
  }, [aggiorna, t]);

  const salva = useCallback(
    (nome: string, bozza: string) =>
      inCoda(async () => {
        const nota = attuale.current?.note.find((n) => n.nome === nome);
        // Rinominata o eliminata nel frattempo: la bozza era già salvata uscendo dal campo.
        if (!nota) {
          return false;
        }
        const da = salvataggio(nome, nota.salvato, bozza);
        if (!da) {
          return nota.salvato !== null;
        }
        const result = await commands.saveNota(path, nome, da.testo, da.atteso);
        if (result.status === "error") {
          onError(result.error);
          return false;
        }
        aggiorna((s) => ({
          ...s,
          note: s.note.map((n) =>
            n.nome === nome ? { ...n, salvato: da.testo } : n
          ),
        }));
        onSaved(path, result.data);
        return true;
      }),
    [aggiorna, inCoda, onError, onSaved, path]
  );

  const rinomina = useCallback(
    (nome: string, nuovo: string) =>
      inCoda(async () => {
        const note = attuale.current?.note ?? [];
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
        // Una Nota nuova non è ancora su disco: cambia solo il nome con cui nascerà.
        if (nota.salvato !== null) {
          const result = await commands.renameNota(path, nome, nuovo);
          if (result.status === "error") {
            onError(result.error);
            return;
          }
          onSaved(path, result.data);
        }
        aggiorna((s) => ({
          aperta: s.aperta === nome ? nuovo : s.aperta,
          note: s.note.map((n) =>
            n.nome === nome ? { ...n, nome: nuovo } : n
          ),
        }));
      }),
    [aggiorna, inCoda, onError, onSaved, path]
  );

  const elimina = useCallback(
    (nome: string) =>
      inCoda(async () => {
        const nota = attuale.current?.note.find((n) => n.nome === nome);
        if (!nota) {
          return;
        }
        if (nota.salvato !== null) {
          const result = await commands.deleteNota(path, nome);
          if (result.status === "error") {
            onError(result.error);
            return;
          }
          onSaved(path, result.data);
        }
        aggiorna((s) => {
          const resto = senzaNota(s.note, nome, predefinito);
          return {
            aperta: s.aperta === nome ? resto.aperta : s.aperta,
            note: resto.note,
          };
        });
      }),
    [aggiorna, inCoda, onError, onSaved, path, predefinito]
  );

  return useMemo(
    () => stato && { ...stato, apri, crea, elimina, rinomina, salva },
    [apri, crea, elimina, rinomina, salva, stato]
  );
}
