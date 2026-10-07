import { Info } from "lucide-react";
import { Tooltip } from "radix-ui";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

/** Aiuto contestuale, disponibile anche con Tab e chiudibile con Escape. */
export function FieldHelp({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  const { t } = useTranslation();
  return (
    <Tooltip.Root>
      <Tooltip.Trigger asChild>
        <button
          aria-label={t("settings.help", { field: label })}
          className="inline-flex size-6 shrink-0 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
          type="button"
        >
          <Info aria-hidden className="size-4" />
        </button>
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content
          className="z-50 max-w-[min(22rem,calc(100vw-2rem))] rounded-2xl bg-popover px-4 py-3 text-popover-foreground text-sm leading-relaxed shadow-float"
          collisionPadding={16}
          sideOffset={6}
        >
          {children}
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}
