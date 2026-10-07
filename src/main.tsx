import "@/lib/i18n";

import i18n from "i18next";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/app";
import { type AppError, commands, type Settings } from "@/bindings";
import { DEFAULT_SETTINGS, languageOf } from "@/features/settings/settings";

/**
 * Le impostazioni si leggono prima del primo render: nessun valore predefinito compare per un
 * attimo. Se non si leggono l'app parte comunque, con i predefiniti e l'errore nell'avviso.
 */
async function loadSettings(): Promise<[Settings, AppError | null]> {
  try {
    const result = await commands.getSettings();
    return result.status === "ok"
      ? [result.data, null]
      : [DEFAULT_SETTINGS, result.error];
  } catch (e) {
    return [
      DEFAULT_SETTINGS,
      { code: "unreadableSettings", detail: String(e) },
    ];
  }
}

async function start() {
  const [settings, error] = await loadSettings();
  // Senza una scelta salvata vale la lingua di Windows; se il backend non risponde, quella della
  // WebView, che la segue.
  const language =
    settings.interfaceLanguage ??
    (await commands
      .systemLanguage()
      .catch(() => languageOf(navigator.language)));
  await i18n.changeLanguage(language);
  document.documentElement.lang = language;
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App settings={settings} settingsError={error} />
    </React.StrictMode>
  );
}

start().catch(() => {
  window.dispatchEvent(new Event("memotape-startup-error"));
});
