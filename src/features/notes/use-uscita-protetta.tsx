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
import { useSettings } from "@/features/settings/settings-context";
import { bozze, uscita } from "./bozze";

/** Un'uscita dal Tape ferma su «Salvare le modifiche alle Note?»: `annulla` se si resta. */
interface Uscita {
  annulla?: () => void;
  vai: () => void;
}

/**
 * Le bozze delle Note, per Tape, e l'uscita protetta dal Tape `mostrato` (`null`: al centro non
 * c'è un Tape). Le bozze stanno qui, nella route, e restano se la vista del Tape si rimonta.
 * `proteggi(vai, verso)` fa `vai` subito senza bozze, dopo averle salvate con Salva da solo, o
 * dopo il dialog «Salvare le modifiche alle Note?» (Salva, Non salvare, Annulla), che la route
 * mostra (`dialog`). Lo stesso vale per Impostazioni (il router si ferma) e per la chiusura della
 * finestra, anche con Alt+F4. La vista del Tape registra in `salvaNote` come salvare le sue bozze;
 * `scarti` cresce con Non salvare, e l'editor della Nota riparte dal testo su disco.
 */
export function useUscitaProtetta(mostrato: string | null) {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const salvaDaSolo = settings.noteSalvaDaSolo ?? true;
  const [stato, dispatch] = useReducer(bozze, {});
  const salvaNote = useRef<(() => Promise<boolean>) | null>(null);
  const [chiesta, setChiesta] = useState<Uscita | null>(null);
  const [scarti, setScarti] = useState(0);
  const salvaTutte = useCallback(
    () => salvaNote.current?.() ?? Promise.resolve(true),
    []
  );

  const proteggi = useCallback(
    (vai: () => void, verso?: string) => {
      const come = uscita(stato, { mostrato, salvaDaSolo, verso });
      if (come === "vai") {
        vai();
      } else if (come === "salva") {
        salvaTutte().then((ok) => ok && vai());
      } else {
        setChiesta({ vai });
      }
    },
    [mostrato, salvaDaSolo, salvaTutte, stato]
  );

  const salvaEVai = useCallback(async () => {
    if (!chiesta) {
      return;
    }
    setChiesta(null);
    if (await salvaTutte()) {
      chiesta.vai();
    } else {
      chiesta.annulla?.();
    }
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

  // Senza prevenire la chiusura, Tauri distrugge la finestra (`core:window:allow-destroy`).
  const chiusura = useEffectEvent(async (event: CloseRequestedEvent) => {
    const come = uscita(stato, { mostrato, salvaDaSolo });
    if (come === "chiedi") {
      event.preventDefault();
      setChiesta({ vai: () => getCurrentWindow().destroy() });
    } else if (come === "salva" && !(await salvaTutte())) {
      event.preventDefault();
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
