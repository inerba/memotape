import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands, type Nota } from "@/bindings";
import type { BozzeAction } from "./bozze";
import {
  apertaIniziale,
  conNome,
  conNotaNuova,
  conTesto,
  noteIniziali,
  type RigaDelleNote,
  rilette,
  rinominaRifiutata,
  salvaOgni,
  salvataggio,
  senzaNota,
} from "./notes";

interface Stato extends RigaDelleNote {
  /** Le Note con bozza cambiate da fuori, con la Nota su disco: aspettano la scelta dell'utente. */
  conflitti: Record<string, Nota>;
  /** Cresce quando la Nota aperta prende un testo da fuori: il suo editor riparte da lì. */
  versione: number;
}

/** Le Note del Tape aperto e le operazioni su di loro. */
export interface NoteDelTape extends Stato {
  apri: (nome: string) => void;
  /** Le bozze delle Note del Tape, per nome: il testo non ancora su disco. */
  bozze: Record<string, string>;
  /** Una Nota nuova, «Nota 2», «Nota 3»…, aperta: nasce su disco con il primo carattere. */
  crea: () => string;
  elimina: (nome: string) => void;
  /** Rinomina la Nota; un nome rifiutato va a `onError`. */
  rinomina: (nome: string, nuovo: string) => void;
  /** Salva la bozza della Nota: `null` se ora non c'è niente da perdere, se no il motivo. */
  salva: (nome: string, bozza: string) => Promise<AppError | null>;
  /** Salva le bozze di tutte le Note: `null` se ora sono tutte su disco, se no il primo motivo. */
  salvaTutte: () => Promise<AppError | null>;
  /** L'editor della Nota `nome` ha `testo`: la bozza va nella route. */
  scrivi: (nome: string, testo: string) => void;
  /** «Tieni le mie»: la bozza della Nota in conflitto si salva sopra il testo nuovo. */
  tieniLeMie: (nome: string) => void;
  /** «Usa le sue»: la bozza della Nota in conflitto si butta per il testo nuovo. */
  usaLeSue: (nome: string) => void;
}

/** La firma di una scrittura dell'utente, adesso. */
function firmaDellUtente() {
  return {
    autore: { tipo: "utente" as const },
    modificata: new Date().toISOString(),
  };
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
 * precedente; `onSaved` riceve la data di modifica nuova del Tape. Le bozze stanno nella route
 * (`bozze`, `onBozze`), che le tiene se la vista si rimonta. `scelta` (la Nota di un risultato
 * della ricerca) si apre appena c'è. Quando `ricariche` cresce (il Tape è cambiato da fuori, ADR-0031)
 * le Note si rileggono con `rilette`. `null` finché non sono lette.
 */
export function useNote({
  bozze,
  onBozze,
  onError,
  onSaved,
  path,
  ricariche,
  scelta,
}: {
  bozze: Record<string, string>;
  onBozze: (action: BozzeAction) => void;
  onError: (error: AppError) => void;
  onSaved: (path: string, modificato: number | null) => void;
  path: string;
  ricariche: number;
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
  // Le bozze di adesso: Salva le legge quando lo si preme.
  const bozzeAttuali = useRef(bozze);
  bozzeAttuali.current = bozze;

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
        conflitti: {},
        note,
        versione: 0,
      };
      setStato(attuale.current);
    });
    return () => {
      vivo = false;
    };
  }, [onError, path, predefinito]);

  // Le Note cambiate da fuori, in coda dopo i salvataggi in corso: un proprio salvataggio non
  // sembra di fuori.
  const ricaricheLette = useRef(ricariche);
  useEffect(() => {
    if (ricariche === ricaricheLette.current) {
      return;
    }
    ricaricheLette.current = ricariche;
    inCoda(async () => {
      const result = await commands.tapeNote(path);
      if (result.status === "error") {
        onError(result.error);
        return;
      }
      aggiorna((s) => {
        const prima = s.note.find((n) => n.nome === s.aperta);
        const { conflitti, note } = rilette(
          s.note,
          result.data,
          bozzeAttuali.current,
          predefinito
        );
        const aperta = note.find((n) => n.nome === s.aperta) ?? note[0];
        return {
          aperta: aperta.nome,
          conflitti,
          note,
          versione: s.versione + (aperta.salvato === prima?.salvato ? 0 : 1),
        };
      });
    });
  }, [aggiorna, inCoda, onError, path, predefinito, ricariche]);

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
    aggiorna((s) => ({
      ...s,
      ...conNotaNuova(s, (n) => t("notes.newName", { n })),
    }));
    return attuale.current?.aperta ?? "";
  }, [aggiorna, t]);

  const salva = useCallback(
    (nome: string, bozza: string) =>
      inCoda(async () => {
        const nota = attuale.current?.note.find((n) => n.nome === nome);
        // Rinominata o eliminata nel frattempo: la bozza era già salvata uscendo dal campo.
        if (!nota) {
          return bozzeAttuali.current[nome] === undefined
            ? null
            : { code: "noteNotFound" as const, detail: nome };
        }
        // Una Nota in conflitto aspetta «Tieni le mie» o «Usa le sue»: lo dicono l'avviso e il
        // segno sul nome, il motivo lo riceve chi ha chiesto di salvare.
        if (nome in (attuale.current?.conflitti ?? {})) {
          return { code: "noteChanged" as const, detail: nome };
        }
        // Senza bozza non c'è niente da salvare: l'editor che si chiude dopo un testo arrivato da
        // fuori non lo riscrive con il suo.
        const da =
          bozzeAttuali.current[nome] === undefined
            ? null
            : salvataggio(nome, nota.salvato, bozza);
        if (!da) {
          return null;
        }
        const result = await commands.saveNota(path, nome, da.testo, da.atteso);
        if (result.status === "error") {
          onError(result.error);
          return result.error;
        }
        aggiorna((s) => ({
          ...s,
          ...conTesto(s, nome, da.testo, firmaDellUtente()),
        }));
        onBozze({ nome, path, testo: da.testo, type: "salvata" });
        onSaved(path, result.data);
        return null;
      }),
    [aggiorna, inCoda, onBozze, onError, onSaved, path]
  );

  const salvaTutte = useCallback(
    () => salvaOgni(bozzeAttuali.current, salva),
    [salva]
  );

  const scrivi = useCallback(
    (nome: string, testo: string) => {
      const nota = attuale.current?.note.find((n) => n.nome === nome);
      if (nota) {
        onBozze({ nome, path, salvato: nota.salvato, testo, type: "scritta" });
      }
    },
    [onBozze, path]
  );

  const rinomina = useCallback(
    (nome: string, nuovo: string) =>
      inCoda(async () => {
        const note = attuale.current?.note ?? [];
        const nota = note.find((n) => n.nome === nome);
        if (!nota || nuovo === nome) {
          return;
        }
        const rifiuto = rinominaRifiutata(note, nome, nuovo);
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
        onBozze({ nome, nuovo, path, type: "rinominata" });
        aggiorna((s) => ({ ...s, ...conNome(s, nome, nuovo) }));
      }),
    [aggiorna, inCoda, onBozze, onError, onSaved, path]
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
        onBozze({ nome, path, type: "eliminata" });
        aggiorna((s) => ({ ...s, ...senzaNota(s, nome, predefinito) }));
      }),
    [aggiorna, inCoda, onBozze, onError, onSaved, path, predefinito]
  );

  const tieniLeMie = useCallback(
    (nome: string) =>
      inCoda(async () => {
        const sua = attuale.current?.conflitti[nome];
        const bozza = bozzeAttuali.current[nome];
        if (!sua || bozza === undefined) {
          return;
        }
        const result = await commands.saveNota(path, nome, bozza, sua.testo);
        if (result.status === "error") {
          onError(result.error);
          return;
        }
        aggiorna((s) => ({
          ...s,
          ...conTesto(s, nome, bozza, firmaDellUtente()),
          conflitti: senza(s.conflitti, nome),
        }));
        onBozze({ nome, path, testo: bozza, type: "salvata" });
        onSaved(path, result.data);
      }),
    [aggiorna, inCoda, onBozze, onError, onSaved, path]
  );

  const usaLeSue = useCallback(
    (nome: string) => {
      const sua = attuale.current?.conflitti[nome];
      if (!sua) {
        return;
      }
      // La bozza va via con il testo nuovo: l'editor che si chiude non ha niente da salvare.
      onBozze({
        nome,
        path,
        salvato: sua.testo,
        testo: sua.testo,
        type: "scritta",
      });
      aggiorna((s) => ({
        ...conTesto(s, nome, sua.testo, {
          autore: sua.autore,
          modificata: sua.modificata,
        }),
        conflitti: senza(s.conflitti, nome),
        versione: s.versione + (s.aperta === nome ? 1 : 0),
      }));
    },
    [aggiorna, onBozze, path]
  );

  return useMemo(
    () =>
      stato && {
        ...stato,
        apri,
        bozze,
        crea,
        elimina,
        rinomina,
        salva,
        salvaTutte,
        scrivi,
        tieniLeMie,
        usaLeSue,
      },
    [
      apri,
      bozze,
      crea,
      elimina,
      rinomina,
      salva,
      salvaTutte,
      scrivi,
      stato,
      tieniLeMie,
      usaLeSue,
    ]
  );
}

/** `conflitti` senza la Nota `nome`. */
function senza(
  conflitti: Record<string, Nota>,
  nome: string
): Record<string, Nota> {
  const { [nome]: _, ...altri } = conflitti;
  return altri;
}
