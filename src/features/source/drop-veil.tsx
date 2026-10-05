import { Ban, FileUp } from "lucide-react";
import { useTranslation } from "react-i18next";
import { type DropVerdict, SOURCE_EXTENSIONS } from "@/features/source/drop";

const REASONS = {
  busy: "source.dropBusy",
  format: "source.dropFormat",
  many: "source.dropMany",
} as const;

/** Sopra tutta la finestra mentre si trascinano file da Esplora file: dice se il rilascio li apre. */
export function DropVeil({ verdict }: { verdict: DropVerdict }) {
  const { t } = useTranslation();
  return (
    <div
      aria-live="polite"
      className="pointer-events-none fixed inset-0 z-50 grid place-items-center bg-background/85"
      role="status"
    >
      <div
        className={`flex max-w-md flex-col items-center gap-3 rounded-2xl border border-dashed bg-card px-12 py-10 text-center shadow-float ${
          verdict.accepted ? "border-primary" : "border-destructive/60"
        }`}
      >
        {verdict.accepted ? (
          <FileUp aria-hidden className="size-8" />
        ) : (
          <Ban aria-hidden className="size-8 text-destructive" />
        )}
        <p className="font-medium text-lg">
          {verdict.accepted ? t("source.dropOpen") : t(REASONS[verdict.reason])}
        </p>
        {!verdict.accepted && verdict.reason === "format" ? (
          <p className="text-muted-foreground text-sm">
            {t("source.dropFormats", {
              formats: SOURCE_EXTENSIONS.map((e) => `.${e}`).join(" "),
            })}
          </p>
        ) : null}
      </div>
    </div>
  );
}
