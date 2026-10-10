import {
  type ComponentProps,
  type ReactNode,
  type ToggleEvent,
  useCallback,
  useState,
} from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/** L'ancora invisibile nel punto del clic destro, comune a tutti i menu. */
const PUNTO = "menu-punto";

/** L'id del pannello del menu `id`, per `popoverTarget` di una voce che lo chiude. */
export function menuId(id: string): string {
  return `menu-${id}`;
}

/** Chiude il menu `id`, se è aperto. */
export function closeMenu(id: string) {
  document.getElementById(menuId(id))?.hidePopover();
}

/**
 * Apre il menu `id` nel punto (`x`, `y`) della finestra, come un menu contestuale: il pannello si
 * ancora a un punto invisibile lì e, chiuso, torna sotto il suo pulsante.
 */
export function openMenuAt(id: string, x: number, y: number) {
  const panel = document.getElementById(menuId(id));
  if (!panel) {
    return;
  }
  let punto = document.getElementById(PUNTO);
  if (!punto) {
    punto = document.createElement("span");
    punto.id = PUNTO;
    punto.style.cssText = `position:fixed;width:0;height:0;pointer-events:none;anchor-name:--${PUNTO}`;
    document.body.append(punto);
  }
  punto.style.left = `${x}px`;
  punto.style.top = `${y}px`;
  panel.style.setProperty("position-anchor", `--${PUNTO}`);
  panel.style.setProperty("position-area", "bottom span-right");
  panel.showPopover();
}

/**
 * Un pulsante ▾ che apre un pannello sotto di sé; un clic fuori o Esc lo chiude. `id` è anche il
 * nome dell'ancora: deve essere unico nella pagina, anche nei menu ripetuti per Turno. `onOpen`
 * scatta a ogni apertura; `panelClassName` aggiunge classi al pannello (per esempio la larghezza).
 * `lazy` monta il contenuto solo da aperto, per i menu ripetuti su ogni riga di un elenco.
 */
export function PopoverMenu({
  children,
  className = "",
  disabled,
  icon,
  id,
  label,
  lazy,
  onOpen,
  panelClassName,
  size = "icon",
  tabIndex,
  variant = "outline",
}: {
  children: ReactNode;
  /** Le classi del pulsante. */
  className?: string;
  disabled?: boolean;
  icon: ReactNode;
  id: string;
  label: string;
  lazy?: boolean;
  onOpen?: () => void;
  panelClassName?: string;
  /** `icon` per un pulsante con la sola icona; un'altra misura per un pulsante con il testo. */
  size?: ComponentProps<typeof Button>["size"];
  tabIndex?: number;
  variant?: ComponentProps<typeof Button>["variant"];
}) {
  const anchor = `--menu-${id}`;
  const [open, setOpen] = useState(false);
  const toggle = useCallback(
    (event: ToggleEvent<HTMLDivElement>) => {
      const opened = event.newState === "open";
      setOpen(opened);
      if (opened) {
        onOpen?.();
      } else {
        // Aperto nel punto del clic destro: la prossima volta torna sotto il pulsante.
        event.currentTarget.style.setProperty("position-anchor", anchor);
        event.currentTarget.style.removeProperty("position-area");
      }
    },
    [anchor, onOpen]
  );
  return (
    <>
      <Button
        aria-label={label}
        className={className}
        disabled={disabled}
        popoverTarget={menuId(id)}
        size={size}
        style={{ anchorName: anchor }}
        tabIndex={tabIndex}
        title={label}
        variant={variant}
      >
        {icon}
      </Button>
      <div
        className={cn(
          "inset-auto m-0 mt-1.5 hidden min-w-60 flex-col gap-0.5 rounded-xl border bg-popover p-1.5 text-popover-foreground text-sm shadow-float [position-area:bottom_span-left] [position-try-fallbacks:flip-block,flip-inline] open:flex",
          panelClassName
        )}
        id={menuId(id)}
        onToggle={toggle}
        popover="auto"
        style={{ positionAnchor: anchor }}
      >
        {open || !lazy ? children : null}
      </div>
    </>
  );
}
