import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";

/**
 * I menu a comparsa: con il Popover API nativo, ancorati al loro pulsante con l'anchor positioning
 * di CSS. Le classi sono scritte per intero perché Tailwind le trovi.
 */
const ANCHORS = {
  more: {
    anchor: "[anchor-name:--menu-more]",
    popover: "[position-anchor:--menu-more]",
  },
  record: {
    anchor: "[anchor-name:--menu-record]",
    popover: "[position-anchor:--menu-record]",
  },
  transcribe: {
    anchor: "[anchor-name:--menu-transcribe]",
    popover: "[position-anchor:--menu-transcribe]",
  },
} as const;

/**
 * Un pulsante ▾ che apre un pannello sotto di sé; un clic fuori o Esc lo chiude. `id` è anche il
 * nome dell'ancora: un menu di quel tipo per pagina.
 */
export function PopoverMenu({
  children,
  disabled,
  icon,
  id,
  label,
}: {
  children: ReactNode;
  disabled?: boolean;
  icon: ReactNode;
  id: keyof typeof ANCHORS;
  label: string;
}) {
  const { anchor, popover } = ANCHORS[id];
  return (
    <>
      <Button
        aria-label={label}
        className={anchor}
        disabled={disabled}
        popoverTarget={`menu-${id}`}
        size="icon"
        title={label}
        variant="outline"
      >
        {icon}
      </Button>
      <div
        className={`${popover} inset-auto m-0 mt-1 hidden min-w-56 flex-col gap-3 rounded-md border bg-popover p-3 text-popover-foreground text-sm shadow-md [position-area:bottom_span-left] [position-try-fallbacks:flip-block,flip-inline] open:flex`}
        id={`menu-${id}`}
        popover="auto"
      >
        {children}
      </div>
    </>
  );
}
