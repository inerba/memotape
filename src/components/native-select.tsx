import { ChevronDown } from "lucide-react";
import type { ComponentProps } from "react";
import { cn } from "@/lib/utils";

/**
 * Un `<select>` nativo (tastiera, lettore di schermo e tema di Windows nell'elenco) con il disegno
 * dell'app: bordo sottile, fondo carta e la freccia di lucide al posto di quella di Windows.
 */
export function NativeSelect({
  className = "",
  wrapperClassName = "",
  ...props
}: ComponentProps<"select"> & { wrapperClassName?: string }) {
  return (
    <span className={`relative inline-flex min-w-0 ${wrapperClassName}`}>
      <select
        className={cn(
          "h-8 w-full min-w-0 cursor-pointer appearance-none truncate rounded-md border border-input bg-card pr-8 pl-2.5 text-foreground text-sm transition-colors hover:bg-accent focus-visible:border-ring focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/30 disabled:cursor-default disabled:opacity-50",
          className
        )}
        {...props}
      />
      <ChevronDown
        aria-hidden
        className="pointer-events-none absolute top-1/2 right-2 size-4 -translate-y-1/2 text-muted-foreground"
      />
    </span>
  );
}
