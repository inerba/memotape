import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

export function ErrorView({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  const { t } = useTranslation();
  return (
    <main
      className={cn(
        "flex h-full flex-col items-center justify-center bg-background p-8 text-center",
        className
      )}
    >
      <div className="text-center">
        <p className="font-semibold text-base text-destructive">
          {t("errors.label")}
        </p>
        {children}
      </div>
    </main>
  );
}

export function ErrorHeader({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <h1
      className={cn(
        "mt-4 font-bold text-3xl text-foreground tracking-tight sm:text-5xl",
        className
      )}
    >
      {children}
    </h1>
  );
}

export function ErrorDescription({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <p
      className={cn(
        "mt-6 text-base text-muted-foreground leading-7",
        className
      )}
    >
      {children}
    </p>
  );
}

export function ErrorActions({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "mt-10 flex items-center justify-center gap-x-6",
        className
      )}
    >
      {children}
    </div>
  );
}
