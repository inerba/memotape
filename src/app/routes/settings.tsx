import { zodResolver } from "@hookform/resolvers/zod";
import { ArrowLeft } from "lucide-react";
import { useCallback, useState } from "react";
import { useController, useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";
import type { AppError, Settings } from "@/bindings";
import { Button } from "@/components/ui/button";
import { ModelList } from "@/features/models/model-list";
import { settingsSchema } from "@/features/settings/settings";
import { useSettings } from "@/features/settings/settings-context";
import { errorText } from "@/features/status/status";

// ponytail: solo la sezione Trascrizione; le altre arrivano con i ticket 08 e 10.
export function SettingsPage() {
  const { t } = useTranslation();
  const { save, settings } = useSettings();
  const [error, setError] = useState<AppError | null>(null);
  // Ogni scelta si salva subito: non c'è un pulsante Salva.
  const form = useForm<Settings>({
    resolver: zodResolver(settingsSchema),
    values: settings,
  });
  const { field: model } = useController({
    control: form.control,
    name: "model",
  });
  const { onChange } = model;
  const { handleSubmit, reset } = form;
  const selectModel = useCallback(
    (id: string) => {
      onChange(id);
      handleSubmit(async (values) => {
        const failed = await save(values);
        setError(failed);
        if (failed) {
          // La scelta non è salvata: il form torna alle impostazioni correnti.
          reset(settings);
        }
      })();
    },
    [onChange, handleSubmit, reset, save, settings]
  );

  return (
    <div className="fixed inset-0 z-10 flex flex-col bg-background">
      <header className="flex items-center gap-3 border-b px-6 py-3">
        <Button asChild size="sm" variant="ghost">
          <Link to="/">
            <ArrowLeft />
            {t("settings.back")}
          </Link>
        </Button>
        <h1 className="font-semibold text-lg">{t("settings.title")}</h1>
      </header>
      <main className="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto p-6">
        <section
          aria-labelledby="settings-transcription"
          className="flex flex-col gap-3"
        >
          <div>
            <h2 className="font-medium" id="settings-transcription">
              {t("settings.transcription.title")}
            </h2>
            <p className="text-muted-foreground text-sm">
              {t("settings.transcription.description")}
            </p>
          </div>
          <ModelList onSelect={selectModel} selected={model.value} />
          {error ? (
            <p className="text-destructive text-sm" role="alert">
              {errorText(error, t)}
            </p>
          ) : null}
        </section>
      </main>
    </div>
  );
}
