import { type ReactNode, Suspense } from "react";
import { ErrorBoundary } from "react-error-boundary";
import AppErrorPage from "@/features/errors/app-error";

export default function AppProvider({ children }: { children: ReactNode }) {
  return (
    <ErrorBoundary FallbackComponent={AppErrorPage}>
      <Suspense fallback={null}>{children}</Suspense>
    </ErrorBoundary>
  );
}
