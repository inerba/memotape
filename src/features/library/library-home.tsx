import { ArrowRight, ChevronRight, Clock, FileUp, Mic } from "lucide-react";
import { type MouseEvent, type ReactNode, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { TapeEntry } from "@/bindings";
import { Button } from "@/components/ui/button";
import { clockText, dayText, durationWords } from "@/features/library/library";
import { homeTapes, recentTitle } from "@/features/library/recent-tapes";
import { SIDEBAR_TOGGLE_PADDING } from "@/features/library/sidebar";
import { elapsedText } from "@/features/recording/recording";

/** La Home: ripresa esplicita del Tape, oppure primo avvio con le due azioni. */
export function LibraryHome({
  banner,
  busy,
  lastPath,
  loading,
  onImport,
  onOpen,
  onRecord,
  onShowAll,
  tapes,
}: {
  /** L'avviso della finestra, sotto la barra in alto. */
  banner?: ReactNode;
  busy: boolean;
  lastPath: string | null;
  loading: boolean;
  onImport: () => void;
  onOpen: (path: string) => void;
  onRecord: () => void;
  onShowAll: () => void;
  tapes: TapeEntry[];
}) {
  const { i18n, t } = useTranslation();
  const { featured, resumed, recent } = homeTapes(tapes, lastPath);
  const populated = tapes.length > 0;
  const openFeatured = useCallback(() => {
    if (featured) {
      onOpen(featured.path);
    }
  }, [featured, onOpen]);
  const openRecent = useCallback(
    (event: MouseEvent<HTMLButtonElement>) => {
      const { path } = event.currentTarget.dataset;
      if (path) {
        onOpen(path);
      }
    },
    [onOpen]
  );
  return (
    <>
      <header
        className={`flex h-12 shrink-0 items-center border-b pr-36 text-sm ${SIDEBAR_TOGGLE_PADDING}`}
        data-tauri-drag-region
      >
        <span className="pointer-events-none">{t("home.title")}</span>
      </header>
      {banner}
      <div className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
        <div className="mx-auto flex w-full max-w-5xl flex-col gap-10 px-8 py-10 min-[1200px]:px-14 min-[1200px]:py-12">
          {loading ? (
            <p className="text-muted-foreground text-sm" role="status">
              {t("home.loading")}
            </p>
          ) : (
            <>
              <section className="flex flex-col gap-6">
                <div className="flex flex-col gap-2">
                  <h1 className="max-w-[24ch] text-balance font-display font-medium text-4xl leading-tight tracking-[-0.015em]">
                    {populated ? t("home.resume") : t("welcome.title")}
                  </h1>
                  <p className="max-w-[65ch] text-base text-muted-foreground leading-relaxed">
                    {populated
                      ? t(
                          resumed
                            ? "home.resumeDescription"
                            : "home.recentDescription"
                        )
                      : t("welcome.description")}
                  </p>
                </div>
                {featured ? (
                  <section className="flex flex-wrap items-center justify-between gap-6 rounded-xl border bg-card p-7 text-card-foreground">
                    <div className="flex min-w-0 flex-1 basis-64 flex-col gap-2">
                      <h2
                        className="wrap-anywhere font-medium text-2xl leading-snug"
                        title={featured.titolo}
                      >
                        {recentTitle(featured, tapes, t)}
                      </h2>
                      <p className="text-muted-foreground text-sm tabular-nums">
                        {new Date(featured.creato).toLocaleDateString(
                          i18n.language,
                          {
                            day: "numeric",
                            month: "long",
                            year: "numeric",
                          }
                        )}
                        {" · "}
                        {clockText(featured.creato)}
                        {" · "}
                        {elapsedText(featured.durataMs ?? 0)}
                      </p>
                    </div>
                    <Button onClick={openFeatured} size="lg" variant="outline">
                      {t("home.openTape")}
                      <ArrowRight aria-hidden data-icon="inline-end" />
                    </Button>
                  </section>
                ) : null}
                {populated ? null : (
                  <div className="flex flex-wrap items-center gap-3">
                    <Button disabled={busy} onClick={onRecord} size="lg">
                      <Mic aria-hidden data-icon="inline-start" />
                      {t("sidebar.newRecording")}
                    </Button>
                    <Button
                      disabled={busy}
                      onClick={onImport}
                      size="lg"
                      variant="outline"
                    >
                      <FileUp aria-hidden data-icon="inline-start" />
                      {t("sidebar.importFile")}
                    </Button>
                  </div>
                )}
              </section>
              {populated ? (
                <section className="flex flex-col gap-3">
                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <h2 className="font-display font-medium text-2xl">
                      {t("sidebar.recents")}
                    </h2>
                    <Button onClick={onShowAll} variant="ghost">
                      {t("home.openLibrary")}
                      <ArrowRight aria-hidden data-icon="inline-end" />
                    </Button>
                  </div>
                  <ul className="divide-y">
                    {recent.map((tape) => (
                      <li key={tape.path}>
                        <button
                          className="group flex w-full items-center gap-4 rounded-lg px-3 py-5 text-left transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
                          data-path={tape.path}
                          onClick={openRecent}
                          title={tape.titolo}
                          type="button"
                        >
                          <span className="flex min-w-0 flex-1 flex-col gap-1">
                            <span className="wrap-anywhere font-medium">
                              {recentTitle(tape, tapes, t)}
                            </span>
                            <span className="text-muted-foreground text-sm tabular-nums">
                              {dayText(
                                tape.creato,
                                new Date(),
                                t,
                                i18n.language
                              )}
                              {" · "}
                              {clockText(tape.creato)}
                            </span>
                            <span className="sr-only">{tape.titolo}</span>
                          </span>
                          <span className="inline-flex shrink-0 items-center gap-1.5 text-muted-foreground text-sm tabular-nums">
                            {tape.durataMs === null ? (
                              t("home.unreadable")
                            ) : (
                              <>
                                <Clock aria-hidden className="size-3.5" />
                                {durationWords(tape.durataMs, i18n.language)}
                              </>
                            )}
                          </span>
                          <ChevronRight
                            aria-hidden
                            className="size-4 shrink-0 text-muted-foreground"
                          />
                        </button>
                      </li>
                    ))}
                  </ul>
                </section>
              ) : null}
              <p className="max-w-[70ch] text-muted-foreground text-sm leading-relaxed">
                {populated ? t("home.hint") : t("sidebar.localOnlyNote")}
              </p>
            </>
          )}
        </div>
      </div>
    </>
  );
}
