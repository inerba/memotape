import { LoaderCircle, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import type { Status } from "@/features/status/status";

export function PreparationPanel({
  cancelling,
  onCancel,
  stage,
}: {
  cancelling: boolean;
  onCancel: () => void;
  stage: Extract<Status, { phase: "preparingRecording" }>["stage"];
}) {
  const { t } = useTranslation();
  return (
    <section
      aria-label={t("recording.title")}
      className="mx-auto flex w-full max-w-[52rem] items-center gap-3 rounded-2xl border bg-card px-5 py-3.5 shadow-float"
    >
      <LoaderCircle
        aria-hidden
        className="size-5 shrink-0 text-muted-foreground motion-safe:animate-spin"
      />
      <p className="min-w-0 flex-1 text-sm">
        {t(
          cancelling
            ? "transcription.cancelling"
            : `recording.preparation.${stage}`
        )}
      </p>
      <Button disabled={cancelling} onClick={onCancel} variant="outline">
        <X aria-hidden />
        {t("transcription.cancel")}
      </Button>
    </section>
  );
}
