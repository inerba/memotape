import type { TFunction } from "i18next";
import { X } from "lucide-react";
import {
  createContext,
  type MouseEvent,
  type ReactNode,
  use,
  useCallback,
  useEffect,
  useEffectEvent,
  useLayoutEffect,
  useMemo,
  useRef,
} from "react";
import { useTranslation } from "react-i18next";
import {
  type Azione,
  type Campo,
  type Gruppo,
  SCORCIATOIE,
  scorciatoia,
  tastiDi,
} from "@/features/shortcuts/shortcuts";

interface Contesto {
  apriPannello: () => void;
  /** Collega `azione` a `handler` finché non si chiama la funzione restituita. */
  registra: (azione: Azione, handler: () => void) => () => void;
}

const Scorciatoie = createContext<Contesto | null>(null);

/** Dove sta il focus, per le scorciatoie che valgono solo fuori dai campi. */
function campoDi(target: EventTarget | null): Campo {
  if (!(target instanceof Element)) {
    return null;
  }
  if (
    target.closest(
      "textarea, select, [contenteditable], input:not([type=range])"
    )
  ) {
    return "testo";
  }
  if (target.closest("input[type=range]")) {
    return "cursore";
  }
  return target.closest("button, a, summary") ? "pulsante" : null;
}

/** Un dialog aperto sopra la finestra: le scorciatoie, tranne il pannello, tacciono. */
function dialogAperto(): boolean {
  return (
    document.querySelector(
      "[role=dialog], [role=alertdialog], dialog[open]"
    ) !== null
  );
}

/**
 * L'unico listener delle scorciatoie della finestra principale, e il pannello che le elenca. Le
 * azioni le collegano i componenti con `useScorciatoia`, finché il loro pulsante è attivo.
 * `coperta`: Impostazioni sta sopra la finestra.
 */
export function ScorciatoieProvider({
  attivita,
  children,
  coperta,
  registrazione,
}: {
  attivita: boolean;
  children: ReactNode;
  coperta: boolean;
  registrazione: boolean;
}) {
  const handlers = useRef(new Map<Azione, () => void>());
  const pannello = useRef<HTMLDialogElement>(null);

  const apriPannello = useCallback(() => pannello.current?.showModal(), []);
  const registra = useCallback((azione: Azione, handler: () => void) => {
    handlers.current.set(azione, handler);
    return () => {
      if (handlers.current.get(azione) === handler) {
        handlers.current.delete(azione);
      }
    };
  }, []);
  const contesto = useMemo(
    () => ({ apriPannello, registra }),
    [apriPannello, registra]
  );

  const premuto = useEffectEvent((e: KeyboardEvent) => {
    const trovata = scorciatoia(e, campoDi(e.target), {
      attivita,
      coperta: coperta || dialogAperto(),
      registrazione,
    });
    // Un tasto già usato da un controllo (le frecce di un gruppo di scelte) resta suo.
    if (!trovata || (e.defaultPrevented && !trovata.riservata)) {
      return;
    }
    const handler =
      trovata.azione === "pannello"
        ? () => {
            const dialog = pannello.current;
            if (dialog?.open) {
              dialog.close();
            } else {
              dialog?.showModal();
            }
          }
        : handlers.current.get(trovata.azione);
    if (trovata.riservata || (trovata.esegui && handler)) {
      e.preventDefault();
    }
    if (trovata.esegui) {
      handler?.();
    }
  });
  useEffect(() => {
    const listener = (e: KeyboardEvent) => premuto(e);
    window.addEventListener("keydown", listener);
    return () => window.removeEventListener("keydown", listener);
  }, []);

  return (
    <Scorciatoie value={contesto}>
      {children}
      <Pannello ref={pannello} />
    </Scorciatoie>
  );
}

/** Collega `azione` a `handler` mentre il componente è montato; con `null` la scorciatoia tace. */
export function useScorciatoia(azione: Azione, handler: (() => void) | null) {
  const contesto = use(Scorciatoie);
  const ultimo = useRef(handler);
  useLayoutEffect(() => {
    ultimo.current = handler;
  });
  const attiva = handler !== null;
  useEffect(() => {
    if (!(contesto && attiva)) {
      return;
    }
    return contesto.registra(azione, () => ultimo.current?.());
  }, [attiva, azione, contesto]);
}

/** Apre il pannello delle scorciatoie: il collegamento in Impostazioni. */
export function useApriPannello(): () => void {
  return use(Scorciatoie)?.apriPannello ?? noop;
}

function noop() {
  // Fuori dalla finestra principale non c'è pannello.
}

const FRECCE: Record<string, string> = { ArrowLeft: "←", ArrowRight: "→" };

/** Il nome di un tasto nella lingua dell'interfaccia. */
const TRADOTTI = new Set(["Ctrl", "Shift", "Space", "Enter", "Delete"]);

function nomeTasto(tasto: string, t: TFunction): string {
  if (TRADOTTI.has(tasto)) {
    return t(`shortcuts.keys.${tasto}`);
  }
  return FRECCE[tasto] ?? tasto;
}

/** I tasti di `azione` nella lingua dell'interfaccia: «Ctrl+Maiusc+C», «Canc». */
export function nomeTasti(t: TFunction, azione: Azione): string {
  return tastiDi(azione)
    .map((tasto) => nomeTasto(tasto, t))
    .join("+");
}

/**
 * Il tooltip di un pulsante con le sue scorciatoie: «Copia testo (Ctrl+Maiusc+C)»; più azioni si
 * separano con uno spazio («Velocità ([ ])»).
 */
export function conTasti(t: TFunction, testo: string, ...azioni: Azione[]) {
  const tasti = azioni.map((azione) => nomeTasti(t, azione)).join(" ");
  return `${testo} (${tasti})`;
}

/** Il valore di `aria-keyshortcuts` di `azione`. */
export function ariaTasti(azione: Azione): string {
  return tastiDi(azione)
    .map((tasto) => (tasto === "Ctrl" ? "Control" : tasto))
    .join("+");
}

const GRUPPI: Gruppo[] = [
  "generale",
  "ascolto",
  "testo",
  "registrazione",
  "libreria",
];

/** Il pannello «Scorciatoie da tastiera»: Esc, Ctrl+/ o un clic fuori lo chiudono. */
function Pannello({ ref }: { ref: React.Ref<HTMLDialogElement> }) {
  const { t } = useTranslation();
  const fuori = useCallback((e: MouseEvent<HTMLDialogElement>) => {
    if (e.target === e.currentTarget) {
      e.currentTarget.close();
    }
  }, []);
  const chiudi = useCallback(
    (e: MouseEvent<HTMLButtonElement>) =>
      e.currentTarget.closest("dialog")?.close(),
    []
  );
  return (
    // biome-ignore lint/a11y/useKeyWithClickEvents: Esc chiude il dialog nativo
    <dialog
      aria-labelledby="scorciatoie-titolo"
      className="m-auto w-[35rem] max-w-[calc(100%-2rem)] rounded-2xl border bg-popover p-0 text-popover-foreground shadow-float backdrop:bg-background/55"
      onClick={fuori}
      ref={ref}
    >
      <div className="flex flex-col gap-4 px-6 pt-5 pb-6">
        <div className="flex items-center justify-between gap-4">
          <h2
            className="font-display font-medium text-[1.375rem]"
            id="scorciatoie-titolo"
          >
            {t("shortcuts.title")}
          </h2>
          <button
            aria-label={t("window.close")}
            className="-mr-2 flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
            onClick={chiudi}
            title={t("window.close")}
            type="button"
          >
            <X className="size-4" />
          </button>
        </div>
        <div className="grid grid-cols-2 gap-x-7 gap-y-5">
          {GRUPPI.map((gruppo) => (
            <section className="flex flex-col gap-1.5" key={gruppo}>
              <h3 className="mb-0.5 font-medium text-[0.6875rem] text-muted-foreground uppercase tracking-[0.07em]">
                {t(`shortcuts.groups.${gruppo}`)}
              </h3>
              <dl className="flex flex-col gap-1.5">
                {righe(gruppo).map(({ riga, tasti }) => (
                  <div
                    className="flex items-center justify-between gap-3 text-[0.84375rem]"
                    key={riga}
                  >
                    <dt>{t(`shortcuts.lines.${riga}`)}</dt>
                    <dd className="flex shrink-0 gap-1">
                      {tasti.map((tasto) => (
                        <kbd
                          className="rounded-[5px] border bg-card px-1.5 py-px font-sans text-[0.6875rem] text-muted-foreground"
                          key={tasto}
                        >
                          {nomeTasto(tasto, t)}
                        </kbd>
                      ))}
                    </dd>
                  </div>
                ))}
              </dl>
            </section>
          ))}
        </div>
      </div>
    </dialog>
  );
}

/** Le righe del pannello di `gruppo`: indietro e avanti, e le due velocità, in una sola. */
function righe(gruppo: Gruppo): { riga: string; tasti: string[] }[] {
  const lista = new Map<string, string[]>();
  for (const s of SCORCIATOIE) {
    // ponytail: Ctrl+B non fa ancora nulla; la barra laterale richiudibile (ticket 07) la mostra.
    if (s.gruppo !== gruppo || s.azione === "barraLaterale") {
      continue;
    }
    lista.set(s.riga, [...(lista.get(s.riga) ?? []), ...s.tasti]);
  }
  return [...lista].map(([riga, tasti]) => ({ riga, tasti }));
}
