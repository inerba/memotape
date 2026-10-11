import {
  type CloseRequestedEvent,
  getCurrentWindow,
} from "@tauri-apps/api/window";
import {
  useCallback,
  useEffect,
  useEffectEvent,
  useReducer,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { useBlocker } from "react-router";
import { type AppError, commands } from "@/bindings";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { noteSalvaDaSolo } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { errorText } from "@/features/status/status";
import { type BozzaDaSalvare, bozze, bozzeDaSalvare, uscita } from "./bozze";

/**
 * Un'uscita dal Tape ferma su «Salvare le modifiche alle Note?»: `annulla` se si resta, `errore`
 * se un salvataggio non è riuscito (il dialog lo dice e offre comunque Non salvare).
 */
interface Uscita {
  annulla?: () => void;
  errore?: AppError;
  vai: () => void;
}

/**
 * Le bozze delle Note, per Tape, e l'uscita protetta dal Tape `mostrato` (`null`: al centro non
 * c'è un Tape). Le bozze stanno qui, nella route, e restano se la vista del Tape si rimonta.
 * `proteggi(vai, verso)` fa `vai` subito senza bozze, dopo averle salvate con Salva da solo, o
 * dopo il dialog «Salvare le modifiche alle Note?» (Salva, Non salvare, Annulla), che la route
 * mostra (`dialog`). Un salvataggio che non riesce apre il dialog con il motivo. Lo stesso vale per
 * Impostazioni (il router si ferma) e per la chiusura della finestra, anche con Alt+F4, che guarda
 * le bozze di ogni Tape. La vista del Tape registra in `salvaNote` come salvare le sue bozze; quelle
 * degli altri Tape le salva la route; `scarti` cresce con Non salvare, e l'editor della Nota
 * riparte dal testo su disco.
 */
export function useUscitaProtetta(mostrato: string | null) {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const salvaDaSolo = noteSalvaDaSolo(settings);
  const [stato, dispatch] = useReducer(bozze, {});
  // Le bozze di adesso: un salvataggio le legge quando parte.
  const statoAttuale = useRef(stato);
  statoAttuale.current = stato;
  const salvaNote = useRef<(() => Promise<AppError | null>) | null>(null);
  const [chiesta, setChiesta] = useState<Uscita | null>(null);
  const [scarti, setScarti] = useState(0);

  // Le bozze del Tape mostrato le salva la sua vista, che tiene il testo su disco di ogni Nota;
  // quelle degli altri Tape (o del mostrato, se la vista non c'è ancora) le salva la route.
  const salvaTutte = useCallback(async () => {
    const vista = salvaNote.current;
    const errore = vista ? await vista() : null;
    const altrove = await salvaAltrove(
      bozzeDaSalvare(statoAttuale.current, vista ? mostrato : null),
      dispatch
    );
    return errore ?? altrove;
  }, [mostrato]);

  const proteggi = useCallback(
    (vai: () => void, verso?: string) => {
      const come = uscita(stato, { mostrato, salvaDaSolo, verso });
      if (come === "vai") {
        vai();
      } else if (come === "salva") {
        salvaTutte().then((errore) =>
          errore ? setChiesta({ errore, vai }) : vai()
        );
      } else {
        setChiesta({ vai });
      }
    },
    [mostrato, salvaDaSolo, salvaTutte, stato]
  );

  // Non riuscito, il dialog resta con il motivo.
  const salvaEVai = useCallback(async () => {
    if (!chiesta) {
      return;
    }
    const errore = await salvaTutte();
    if (errore) {
      setChiesta({ ...chiesta, errore });
      return;
    }
    setChiesta(null);
    chiesta.vai();
  }, [chiesta, salvaTutte]);
  const scartaEVai = useCallback(() => {
    if (!chiesta) {
      return;
    }
    setChiesta(null);
    if (mostrato) {
      dispatch({ path: mostrato, type: "scartate" });
    }
    setScarti((n) => n + 1);
    chiesta.vai();
  }, [chiesta, mostrato]);
  // Annulla, Esc o un clic fuori: si resta. Salva e Non salvare chiudono senza passare di qui.
  const chiudi = useCallback(
    (aperta: boolean) => {
      if (!aperta) {
        chiesta?.annulla?.();
        setChiesta(null);
      }
    },
    [chiesta]
  );

  // Impostazioni si apre sopra la finestra: il router si ferma finché non si risponde.
  const blocker = useBlocker(
    ({ currentLocation, nextLocation }) =>
      currentLocation.pathname === "/" &&
      nextLocation.pathname !== "/" &&
      uscita(stato, { mostrato, salvaDaSolo }) === "chiedi"
  );
  const fermata = useEffectEvent(() => {
    if (blocker.state === "blocked") {
      setChiesta({ annulla: blocker.reset, vai: blocker.proceed });
    }
  });
  useEffect(() => {
    if (blocker.state === "blocked") {
      fermata();
    }
  }, [blocker.state]);

  // Senza prevenire la chiusura, Tauri distrugge la finestra (`core:window:allow-destroy`). Contano
  // le bozze di ogni Tape, anche di quello consultato che un'Attività finita ha chiuso.
  const chiusura = useEffectEvent(async (event: CloseRequestedEvent) => {
    const come = uscita(stato, { chiusura: true, mostrato, salvaDaSolo });
    const vai = () => getCurrentWindow().destroy();
    if (come === "chiedi") {
      event.preventDefault();
      setChiesta({ vai });
    } else if (come === "salva") {
      const errore = await salvaTutte();
      if (errore) {
        event.preventDefault();
        setChiesta({ errore, vai });
      }
    }
  });
  useEffect(() => {
    const unlisten = getCurrentWindow().onCloseRequested(chiusura);
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  const dialog = (
    <AlertDialog onOpenChange={chiudi} open={chiesta !== null}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("notes.leave.title")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t("notes.leave.description")}
          </AlertDialogDescription>
          {chiesta?.errore ? (
            <p className="text-destructive text-sm" role="alert">
              {t("notes.leave.failed", {
                motivo: errorText(chiesta.errore, t),
              })}
            </p>
          ) : null}
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t("notes.leave.cancel")}</AlertDialogCancel>
          <Button onClick={scartaEVai} variant="outline">
            {t("notes.leave.discard")}
          </Button>
          <Button onClick={salvaEVai}>{t("notes.leave.save")}</Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );

  return { bozze: stato, dialog, dispatch, proteggi, salvaNote, scarti };
}

/**
 * Salva le bozze `daSalvare` di Tape senza vista, ognuna dal testo su disco da cui parte: `null`
 * se ora sono tutte su disco, se no il primo motivo.
 */
async function salvaAltrove(
  daSalvare: BozzaDaSalvare[],
  dispatch: (action: {
    nome: string;
    path: string;
    testo: string;
    type: "salvata";
  }) => void
): Promise<AppError | null> {
  const esiti = await Promise.all(
    daSalvare.map(async ({ atteso, nome, path, testo }) => {
      const result = await commands.saveNota(path, nome, testo, atteso);
      if (result.status === "error") {
        return result.error;
      }
      dispatch({ nome, path, testo, type: "salvata" });
      return null;
    })
  );
  return esiti.find((e) => e !== null) ?? null;
}
