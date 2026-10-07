import type { ComponentProps, ReactNode, ToggleEvent } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/**
 * Un pulsante ▾ che apre un pannello sotto di sé; un clic fuori o Esc lo chiude. `id` è anche il
 * nome dell'ancora: deve essere unico nella pagina, anche nei menu ripetuti per Turno. `onOpen`
 * scatta a ogni apertura; `panelClassName` aggiunge classi al pannello (per esempio la larghezza).
 */
export function PopoverMenu({
  children,
  className = "",
  disabled,
  icon,
  id,
  label,
  onOpen,
  panelClassName,
  variant = "outline",
}: {
  children: ReactNode;
  /** Le classi del pulsante. */
  className?: string;
  disabled?: boolean;
  icon: ReactNode;
  id: string;
  label: string;
  onOpen?: () => void;
  panelClassName?: string;
  variant?: ComponentProps<typeof Button>["variant"];
}) {
  const anchor = `--menu-${id}`;
  return (
    <>
      <Button
        aria-label={label}
        className={className}
        disabled={disabled}
        popoverTarget={`menu-${id}`}
        size="icon"
        style={{ anchorName: anchor }}
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
        id={`menu-${id}`}
        onToggle={
          onOpen
            ? (event: ToggleEvent<HTMLDivElement>) => {
                if (event.newState === "open") {
                  onOpen();
                }
              }
            : undefined
        }
        popover="auto"
        style={{ positionAnchor: anchor }}
      >
        {children}
      </div>
    </>
  );
}
