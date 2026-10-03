import { use } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";

const versionPromise = commands.appVersion();

export function HomePage() {
  const { t } = useTranslation();
  const version = use(versionPromise);

  return (
    <main className="flex h-screen flex-col items-center justify-center gap-2">
      <h1 className="font-semibold text-3xl">Sbobino</h1>
      <p className="text-muted-foreground">{t("home.version", { version })}</p>
    </main>
  );
}
