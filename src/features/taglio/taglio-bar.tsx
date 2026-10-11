import { Scissors, X } from "lucide-react";
import {
  type PointerEvent,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
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
import { Button } from "@/components/ui/button";
import type { PlayerState } from "@/features/player/player";
import { elapsedText } from "@/features/recording/recording";
import { ariaTasti, conTasti } from "@/features/shortcuts/shortcuts";
import { useScorciatoia } from "@/features/shortcuts/shortcuts-provider";
import {
  conTaglio,
  correzioniDivise,
  durataTagliata,
  type Taglio,
  taglioDelleFrasi,
  taglioDelSilenzio,
} from "@/features/taglio/taglio";
import { pauseDuration } from "@/lib/duration";

/** Il testo in modalità Taglio: la vista lo segna con `data-taglio`. */
function testoDelTaglio(): HTMLElement | null {
  return document.querySelector<HTMLElement>("[data-taglio]");
}

/** La selezione dentro il testo in modalità Taglio, se c'è e non è vuota. */
function selezione(): Selection | null {
  const selection = window.getSelection();
  const root = testoDelTaglio();
  return selection &&
    root &&
    !selection.isCollapsed &&
    root.contains(selection.anchorNode)
    ? selection
    : null;
}

/**
 * I Tagli della selezione: le Frasi che tocca, allargate alle Frasi intere e ai silenzi intorno, o
 * senza Frasi i silenzi che tocca.
 */
function tagliDellaSelezione(
  phrases: TranscriptPhrase[],
  durataMs: number
): Taglio[] {
  const selection = selezione();
  const root = testoDelTaglio();
  if (!(selection && root)) {
    return [];
  }
  // Conta il testo dell'elemento: `intersectsNode` vale anche per una selezione tutta dentro, e
  // una selezione che finisce all'inizio dell'elemento dopo (un triplo clic) non lo prende.
  const range = selection.getRangeAt(0);
  const toccati = (selector: string) =>
    [...root.querySelectorAll<HTMLElement>(selector)].filter((el) =>
      range.intersectsNode(
        el.querySelector(":scope > span") ?? el.firstChild ?? el
      )
    );
  const frasi = new Set(
    toccati("[data-phrase]").map((el) => el.dataset.phrase ?? "")
  );
  const taglio = taglioDelleFrasi(phrases, frasi, durataMs);
  if (taglio) {
    return [taglio];
  }
  return toccati("[data-silenzio]").map((el) =>
    taglioDelSilenzio(Number(el.dataset.a), Number(el.dataset.b), durataMs)
  );
}

/** La modalità Taglio aperta su un Tape. */
export interface ModalitaTaglio {
  /** Applica i Tagli: l'errore, o `null` se il Tape è tagliato. */
  applica: (cestino: boolean) => Promise<AppError | null>;
  /** Esce senza cambiare niente. */
  chiudi: () => void;
  durataMs: number;
  setTagli: (tagli: Taglio[]) => void;
  tagli: Taglio[];
}

/**
 * La modalità Taglio del Tape `path`: `modalita` è `null` fuori, e si chiude da sola se l'Attività
 * in corso comincia a lavorarci (`attiva` falso). `apri` (dopo `onApri`) c'è solo con il testo e
 * fuori dalla modalità. `versione` cresce quando cambia la durata del Tape, cioè a ogni Taglio,
 * applicato qui o da un Assistente (il Tape ricaricato al ritorno in primo piano): chi la usa come
 * `key` del player gli fa rileggere il mix nuovo. I Tagli in sospeso di quel momento non valgono
 * più per il Tape nuovo e la modalità si chiude.
 */
export function useModalitaTaglio({
  attiva,
  info,
  onApri,
  onTagliato,
  path,
  player: { audio, durationMs, move },
  testo,
}: {
  attiva: boolean;
  info: TapeInfo | null;
  onApri: () => void;
  onTagliato: (path: string, opened: OpenedTape) => void;
  path: string;
  player: PlayerState;
  testo: boolean;
}): {
  apri?: () => void;
  modalita: ModalitaTaglio | null;
  versione: number;
} {
  const [tagli, setTagli] = useState<Taglio[] | null>(null);
  if (tagli && !attiva) {
    setTagli(null);
  }
  const [versione, setVersione] = useState(0);
  const [durataLetta, setDurataLetta] = useState(info?.durataMs);
  if (info?.durataMs !== durataLetta) {
    setDurataLetta(info?.durataMs);
    setVersione(versione + 1);
    setTagli(null);
  }
  const durataMs = info?.durataMs ?? durationMs;
  const apri = useCallback(() => {
    onApri();
    setTagli([]);
  }, [onApri]);
  const chiudi = useCallback(() => setTagli(null), []);
  const applica = useCallback(
    async (cestino: boolean) => {
      const result = await commands.tagliaTape(path, tagli ?? [], cestino);
      if (result.status === "error") {
        return result.error;
      }
      audio.current?.pause();
      move(0, "jump");
      onTagliato(path, result.data);
      setTagli(null);
      return null;
    },
    [audio, move, onTagliato, path, tagli]
  );
  const modalita = useMemo(
    () => (tagli ? { applica, chiudi, durataMs, setTagli, tagli } : null),
    [applica, chiudi, durataMs, tagli]
  );
  return {
    apri: attiva && testo && !tagli ? apri : undefined,
    modalita,
    versione,
  };
}

/**
 * La barra della modalità Taglio, sopra il player: i Tagli in sospeso (con Ripristina), Taglia la
 * selezione (Canc), Annulla (Esc) e Applica i tagli con la sua conferma. Se il volume non ha un
 * Cestino chiede una seconda conferma prima di tagliare senza.
 */
export function TaglioBar({
  modalita: {
    applica: onApplica,
    chiudi: onAnnulla,
    durataMs,
    setTagli: onTagli,
    tagli,
  },
  onError,
  phrases,
}: {
  modalita: ModalitaTaglio;
  onError: (error: AppError) => void;
  phrases: TranscriptPhrase[];
}) {
  const { i18n, t } = useTranslation();
  const [selezionato, setSelezionato] = useState(false);
  const [conferma, setConferma] = useState<"tagli" | "cestino" | null>(null);
  const [applying, setApplying] = useState(false);

  useEffect(() => {
    const update = () => setSelezionato(selezione() !== null);
    document.addEventListener("selectionchange", update);
    return () => document.removeEventListener("selectionchange", update);
  }, []);

  const taglia = useCallback(() => {
    const nuovi = tagliDellaSelezione(phrases, durataMs);
    if (nuovi.length > 0) {
      onTagli(nuovi.reduce(conTaglio, tagli));
      window.getSelection()?.removeAllRanges();
    }
  }, [durataMs, onTagli, phrases, tagli]);
  useScorciatoia("tagliaSelezione", selezionato ? taglia : null);
  useScorciatoia("esciDalTaglio", conferma ? null : onAnnulla);

  const applica = useCallback(
    async (cestino: boolean) => {
      setApplying(true);
      const error = await onApplica(cestino);
      setApplying(false);
      if (error?.code === "noRecycleBin" && cestino) {
        setConferma("cestino");
      } else if (error) {
        onError(error);
      }
    },
    [onApplica, onError]
  );
  const conCestino = useCallback(() => applica(true), [applica]);
  const senzaCestino = useCallback(() => applica(false), [applica]);
  const chiudi = useCallback((open: boolean) => {
    if (!open) {
      setConferma(null);
    }
  }, []);
  const chiedi = useCallback(() => setConferma("tagli"), []);
  const ripristina = useCallback(
    (index: number) => onTagli(tagli.filter((_, i) => i !== index)),
    [onTagli, tagli]
  );

  const tolti = durataTagliata(tagli);
  return (
    <div className="mx-auto mb-2 flex w-full max-w-[52rem] flex-col gap-2 rounded-2xl border bg-card px-3.5 py-2.5 shadow-float">
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <Scissors aria-hidden className="size-4 text-foreground/80" />
        <span className="font-medium">{t("taglio.titolo")}</span>
        <span className="min-w-0 flex-1 truncate text-muted-foreground">
          {tagli.length > 0
            ? t("taglio.inSospeso", {
                count: tagli.length,
                durata: pauseDuration(tolti, i18n.language),
              })
            : t("taglio.suggerimento")}
        </span>
        <Button
          className="h-8"
          onClick={onAnnulla}
          title={conTasti(t, t("taglio.annulla"), "esciDalTaglio")}
          variant="ghost"
        >
          {t("taglio.annulla")}
        </Button>
        <Button
          aria-keyshortcuts={ariaTasti("tagliaSelezione")}
          className="h-8 bg-card"
          disabled={!selezionato}
          onClick={taglia}
          onPointerDown={trattieniSelezione}
          title={conTasti(t, t("taglio.tagliaSelezione"), "tagliaSelezione")}
          variant="outline"
        >
          {t("taglio.tagliaSelezione")}
        </Button>
        <Button
          className="h-8"
          disabled={tagli.length === 0 || applying || tolti >= durataMs}
          onClick={chiedi}
        >
          {t("taglio.applica")}
        </Button>
      </div>
      {tagli.length > 0 ? (
        <ul aria-label={t("taglio.elenco")} className="flex flex-wrap gap-1.5">
          {tagli.map((taglio, index) => (
            <TaglioChip
              index={index}
              key={taglio.inizioMs}
              onRipristina={ripristina}
              taglio={taglio}
            />
          ))}
        </ul>
      ) : null}
      <AlertDialog onOpenChange={chiudi} open={conferma !== null}>
        <AlertDialogContent>
          {conferma === "cestino" ? (
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
                <AlertDialogAction onClick={senzaCestino}>
                  {t("taglio.senzaCestino.conferma")}
                </AlertDialogAction>
              </AlertDialogFooter>
            </>
          ) : (
            <>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  {t("taglio.conferma.titolo", { count: tagli.length })}
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {t("taglio.conferma.descrizione", {
                    dopo: elapsedText(Math.max(0, durataMs - tolti)),
                    prima: elapsedText(durataMs),
                  })}
                  {correzioniDivise(phrases, tagli)
                    ? ` ${t("taglio.conferma.correzioni")}`
                    : null}
                </AlertDialogDescription>
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
    </div>
  );
}

/** Il clic su Taglia la selezione non toglie la selezione dal testo. */
function trattieniSelezione(e: PointerEvent) {
  e.preventDefault();
}

function TaglioChip({
  index,
  onRipristina,
  taglio,
}: {
  index: number;
  onRipristina: (index: number) => void;
  taglio: Taglio;
}) {
  const { t } = useTranslation();
  const ripristina = useCallback(
    () => onRipristina(index),
    [index, onRipristina]
  );
  const tratto = `${elapsedText(taglio.inizioMs)}–${elapsedText(taglio.fineMs)}`;
  return (
    <li className="flex items-center gap-1 rounded-full border bg-background py-0.5 pr-0.5 pl-2.5 text-xs tabular-nums">
      {tratto}
      <button
        aria-label={t("taglio.ripristina", { tratto })}
        className="flex size-5 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 [&_svg]:size-3"
        onClick={ripristina}
        title={t("taglio.ripristina", { tratto })}
        type="button"
      >
        <X />
      </button>
    </li>
  );
}
