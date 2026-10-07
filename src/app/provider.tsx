import { Tooltip } from "radix-ui";
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
        <Tooltip.Provider delayDuration={300}>
          <Suspense fallback={null}>{children}</Suspense>
        </Tooltip.Provider>
      </SettingsProvider>
    </ErrorBoundary>
  );
}
