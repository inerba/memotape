import "./global.css";

import AppProvider from "@/app/provider";
import AppRouter from "@/app/router";
import type { AppError, Settings } from "@/bindings";

export default function App({
  settings,
  settingsError,
}: {
  settings: Settings;
  settingsError: AppError | null;
}) {
  return (
    <AppProvider settings={settings} settingsError={settingsError}>
      <AppRouter />
    </AppProvider>
  );
}
