import { type ReactNode, Suspense } from "react";
import { ErrorBoundary } from "react-error-boundary";
import type { AppError, Settings } from "@/bindings";
import AppErrorPage from "@/features/errors/app-error";
import { SettingsProvider } from "@/features/settings/settings-context";

export default function AppProvider({
  children,
  settings,
  settingsError,
}: {
  children: ReactNode;
  settings: Settings;
  settingsError: AppError | null;
}) {
  return (
    <ErrorBoundary FallbackComponent={AppErrorPage}>
      <SettingsProvider initial={settings} loadError={settingsError}>
        <Suspense fallback={null}>{children}</Suspense>
      </SettingsProvider>
    </ErrorBoundary>
  );
}
