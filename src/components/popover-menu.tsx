import type { ComponentProps, ReactNode } from "react";
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
  className = "",
  disabled,
  icon,
  id,
  label,
  variant = "outline",
}: {
  children: ReactNode;
  /** Le classi del pulsante. */
  className?: string;
  disabled?: boolean;
  icon: ReactNode;
  id: keyof typeof ANCHORS;
  label: string;
  variant?: ComponentProps<typeof Button>["variant"];
}) {
  const { anchor, popover } = ANCHORS[id];
  return (
    <>
      <Button
        aria-label={label}
        className={`${anchor} ${className}`}
        disabled={disabled}
        popoverTarget={`menu-${id}`}
        size="icon"
        title={label}
        variant={variant}
      >
        {icon}
      </Button>
      <div
        className={`${popover} inset-auto m-0 mt-1.5 hidden min-w-60 flex-col gap-0.5 rounded-xl border bg-popover p-1.5 text-popover-foreground text-sm shadow-float [position-area:bottom_span-left] [position-try-fallbacks:flip-block,flip-inline] open:flex`}
        id={`menu-${id}`}
        popover="auto"
      >
        {children}
      </div>
    </>
  );
}
