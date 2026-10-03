import { type ReactNode, Suspense } from "react";
import { ErrorBoundary } from "react-error-boundary";
import type { Settings } from "@/bindings";
import AppErrorPage from "@/features/errors/app-error";
import { SettingsProvider } from "@/features/settings/settings-context";

export default function AppProvider({
  children,
  settings,
}: {
  children: ReactNode;
  settings: Settings;
}) {
  return (
    <ErrorBoundary FallbackComponent={AppErrorPage}>
      <SettingsProvider initial={settings}>
        <Suspense fallback={null}>{children}</Suspense>
      </SettingsProvider>
    </ErrorBoundary>
  );
}
