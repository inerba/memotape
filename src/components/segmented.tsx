import { RadioGroup } from "radix-ui";
import { type ReactNode, useCallback } from "react";
import { cn } from "@/lib/utils";

/** L'altezza di ogni segmento, dentro una traccia con 2 px di margine. */
const SIZES = { lg: "h-7", md: "h-6", sm: "h-[18px]" } as const;

/**
 * Una voce del controllo: quello che si mostra (testo breve o simbolo) e il nome intero, per il
 * tooltip e i lettori di schermo.
 */
export interface SegmentedOption<T extends string> {
  label: string;
  short: ReactNode;
  value: T;
}

/**
 * Poche scelte affiancate, una sola attiva: un gruppo di radio (frecce da tastiera, lettore di
 * schermo) disegnato come segmenti su una traccia tenue. `fill` divide tutta la larghezza.
 */
export function Segmented<T extends string>({
  className,
  describedBy,
  disabled,
  fill = false,
  id,
  label,
  name,
  onChange,
  options,
  size = "md",
  value,
}: {
  className?: string;
  describedBy?: string;
  disabled?: boolean;
  fill?: boolean;
  id?: string;
  label: string;
  name?: string;
  onChange: (value: T) => void;
  options: SegmentedOption<T>[];
  /** 22 px nella barra della Registrazione, 28 px nel menu, 32 px in Impostazioni. */
  size?: keyof typeof SIZES;
  value: T;
}) {
  const choose = useCallback(
    (next: string) => {
      const chosen = options.find((option) => option.value === next);
      if (chosen) {
        onChange(chosen.value);
      }
    },
    [onChange, options]
  );
  return (
    <RadioGroup.Root
      aria-describedby={describedBy}
      aria-label={label}
      className={cn(
        "gap-0.5 rounded-md bg-secondary p-0.5 has-[:disabled]:opacity-50",
        fill ? "grid auto-cols-fr grid-flow-col" : "inline-flex",
        className
      )}
      disabled={disabled}
      id={id}
      name={name}
      onValueChange={choose}
      orientation="horizontal"
      value={value}
    >
      {options.map((option) => (
        <RadioGroup.Item
          aria-label={option.label}
          className={cn(
            "inline-flex cursor-pointer items-center justify-center whitespace-nowrap rounded-sm border border-transparent px-2 font-medium text-muted-foreground text-xs transition-colors duration-150 ease-out hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 disabled:cursor-default data-[state=checked]:border-border data-[state=checked]:bg-card data-[state=checked]:text-foreground",
            SIZES[size]
          )}
          key={option.value}
          title={option.label}
          value={option.value}
        >
          {option.short}
        </RadioGroup.Item>
      ))}
    </RadioGroup.Root>
  );
}
