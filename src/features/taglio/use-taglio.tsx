import { type ReactNode, useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type AppError,
  commands,
  type OpenedTape,
  type TapeInfo,
  type TranscriptPhrase,
} from "@/bindings";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import type { PlayerState } from "@/features/player/player";
import { elapsedText } from "@/features/recording/recording";
import {
  type Dove,
  type RichiestaTaglio,
  type Taglio,
  tagliato,
  taglioDelTurno,
} from "@/features/taglio/taglio";
import { type Turn, turnText } from "@/features/transcription/phrases";
import { durationWords } from "@/lib/duration";

/** Quanto testo del Turno che resta mostra la conferma. */
const CITAZIONE = 160;

/** L'inizio del Turno da cui comincerà il Tape, o la fine di quello con cui finirà. */
function citazione(testo: string, dove: Dove): string {
  if (testo.length <= CITAZIONE) {
    return testo;
  }
  return dove === "prima"
    ? `${testo.slice(0, CITAZIONE).trimEnd()}…`
    : `…${testo.slice(-CITAZIONE).trimStart()}`;
}

/**
 * L'anteprima del Taglio nella vista del testo: per ogni Turno i due Tagli possibili (`null` se non
 * c'è niente da togliere) e, mentre si punta una forbice, i Turni che sparirebbero (`fuori`) e il
 * bordo sul Turno puntato. Senza `taglio` non c'è niente.
 */
export function useAnteprimaTaglio(
  turns: Turn[],
  phrases: TranscriptPhrase[],
  taglio: { durataMs: number; onTaglia: (r: RichiestaTaglio) => void } | null
) {
  const durataMs = taglio?.durataMs ?? 0;
  const attiva = Boolean(taglio);
  const tagli = useMemo(
    () =>
      attiva
        ? turns.map((turn) => ({
            dopo: taglioDelTurno(phrases, turn, "dopo", durataMs),
            prima: taglioDelTurno(phrases, turn, "prima", durataMs),
          }))
        : null,
    [attiva, durataMs, phrases, turns]
  );
  const [punto, setPunto] = useState<{ dove: Dove; key: string } | null>(null);
  const indice = punto ? turns.findIndex((t) => t.key === punto.key) : -1;
  const attuale: Taglio | null =
    punto && tagli ? (tagli[indice]?.[punto.dove] ?? null) : null;
  const punta = useCallback(
    (key: string, dove: Dove | null) => setPunto(dove ? { dove, key } : null),
    []
  );
  const onTaglia = taglio?.onTaglia;
  const chiedi = useCallback(
    (turn: Turn, dove: Dove) => {
      const scelto = tagli?.[turns.indexOf(turn)]?.[dove];
      setPunto(null);
      if (scelto && onTaglia) {
        onTaglia({
          dove,
          taglio: scelto,
          turn,
          turni: turns.filter((t) => t.items.every((i) => tagliato(scelto, i)))
            .length,
        });
      }
    },
    [onTaglia, tagli, turns]
  );
  return {
    bordo: (turn: Turn): Dove | null =>
      attuale && punto?.key === turn.key ? punto.dove : null,
    chiedi: onTaglia ? chiedi : undefined,
    fuori: (turn: Turn) =>
      attuale !== null && turn.items.every((i) => tagliato(attuale, i)),
    /** La pausa prima del Turno `index` sparirebbe: il suo centro sta nel Taglio puntato. */
    fuoriPausa: (index: number) => {
      const turn = turns[index];
      const [first] = turn?.items ?? [];
      if (!(attuale && first && index > 0)) {
        return false;
      }
      const fine = Math.max(
        ...(turns[index - 1]?.items ?? []).map((i) => i.fineMs)
      );
      const centro = (fine + first.inizioMs) / 2;
      return attuale.inizioMs <= centro && centro <= attuale.fineMs;
    },
    punta,
    tagliDi: (index: number) => tagli?.[index],
  };
}

/**
 * Il Taglio del Tape `path` chiesto da un Turno: `chiedi` (assente se ci lavora l'Attività in
 * corso) apre la conferma in `dialog`, che dice cosa sparisce e da dove comincia o dove finisce il
 * Tape; su un volume senza Cestino una seconda conferma. `versione` cresce quando cambia la durata
 * del Tape, cioè a ogni Taglio, anche di un Assistente (il Tape ricaricato al ritorno in primo
 * piano): chi la usa come `key` del player gli fa rileggere il mix nuovo.
 */
export function useTaglio({
  attiva,
  info,
  onError,
  onTagliato,
  path,
  player: { audio, durationMs, move },
}: {
  attiva: boolean;
  info: TapeInfo | null;
  onError: (error: AppError) => void;
  onTagliato: (path: string, opened: OpenedTape) => void;
  path: string;
  player: PlayerState;
}) {
  const { i18n, t } = useTranslation();
  // La richiesta resta fino al Taglio: il dialogo si chiude al clic e si riapre senza Cestino.
  const [richiesta, setRichiesta] = useState<RichiestaTaglio | null>(null);
  const [aperto, setAperto] = useState(false);
  const [senzaCestino, setSenzaCestino] = useState(false);
  const [versione, setVersione] = useState(0);
  const [durataLetta, setDurataLetta] = useState(info?.durataMs);
  if (info?.durataMs !== durataLetta) {
    setDurataLetta(info?.durataMs);
    setVersione(versione + 1);
  }
  const durataMs = info?.durataMs ?? durationMs;

  const applica = useCallback(
    async (cestino: boolean) => {
      if (!richiesta) {
        return;
      }
      const result = await commands.tagliaTape(
        path,
        [richiesta.taglio],
        cestino
      );
      if (result.status === "error") {
        if (result.error.code === "noRecycleBin" && cestino) {
          setSenzaCestino(true);
          setAperto(true);
          return;
        }
        onError(result.error);
      } else {
        audio.current?.pause();
        move(0, "jump");
        onTagliato(path, result.data);
      }
    },
    [audio, move, onError, onTagliato, path, richiesta]
  );
  const conCestino = useCallback(() => applica(true), [applica]);
  const senza = useCallback(() => applica(false), [applica]);
  const chiudi = useCallback((open: boolean) => setAperto(open), []);
  const chiedi = useCallback((r: RichiestaTaglio) => {
    setRichiesta(r);
    setSenzaCestino(false);
    setAperto(true);
  }, []);

  let corpo: ReactNode = null;
  if (richiesta) {
    const { dove, taglio, turn, turni } = richiesta;
    const tolti = taglio.fineMs - taglio.inizioMs;
    const durata = durationWords(tolti, i18n.language);
    const testo = turnText(turn);
    corpo = (
      <AlertDialogDescription asChild>
        <div className="flex flex-col gap-3">
          <p>
            {turni > 0
              ? t(`taglio.conferma.${dove}`, { count: turni, durata })
              : t(`taglio.conferma.${dove}Silenzio`, { durata })}
          </p>
          <blockquote className="border-input border-l-2 pl-3 text-foreground">
            <span className="block text-muted-foreground text-xs">
              {t(`taglio.conferma.${dove}Testo`)}
            </span>
            {`«${citazione(testo, dove)}»`}
          </blockquote>
          <p className="tabular-nums">
            {t("taglio.conferma.durata", {
              dopo: elapsedText(Math.max(0, durataMs - tolti)),
              prima: elapsedText(durataMs),
            })}
          </p>
          <p>{t("taglio.conferma.cestino")}</p>
        </div>
      </AlertDialogDescription>
    );
  }

  const dialog = (
    <AlertDialog onOpenChange={chiudi} open={aperto}>
      <AlertDialogContent>
        {senzaCestino ? (
          <>
            <AlertDialogHeader>
              <AlertDialogTitle>
                {t("taglio.senzaCestino.titolo")}
              </AlertDialogTitle>
              <AlertDialogDescription>
                {t("taglio.senzaCestino.descrizione")}
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>{t("taglio.annulla")}</AlertDialogCancel>
              <AlertDialogAction onClick={senza}>
                {t("taglio.senzaCestino.conferma")}
              </AlertDialogAction>
            </AlertDialogFooter>
          </>
        ) : (
          <>
            <AlertDialogHeader>
              <AlertDialogTitle>
                {richiesta
                  ? t(`taglio.conferma.titolo.${richiesta.dove}`)
                  : null}
              </AlertDialogTitle>
              {corpo}
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>{t("taglio.annulla")}</AlertDialogCancel>
              <AlertDialogAction onClick={conCestino}>
                {t("taglio.conferma.azione")}
              </AlertDialogAction>
            </AlertDialogFooter>
          </>
        )}
      </AlertDialogContent>
    </AlertDialog>
  );

  return {
    chiedi: attiva ? chiedi : undefined,
    dialog,
    durataMs,
    versione,
  };
}
