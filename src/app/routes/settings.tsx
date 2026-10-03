import { ArrowLeft } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";
import { Button } from "@/components/ui/button";
import { ModelList } from "@/features/models/model-list";

// ponytail: solo la sezione Trascrizione; le altre arrivano con i ticket 06, 08 e 10.
export function SettingsPage() {
  const { t } = useTranslation();
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
          <ModelList />
        </section>
      </main>
    </div>
  );
}
