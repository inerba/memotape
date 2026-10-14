import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

/** La larghezza dei tre pulsanti: le barre del titolo lasciano libero questo spazio a destra. */
export const WINDOW_CONTROLS_PADDING = "pr-[138px]";

/** ChromeMinimize, ChromeMaximize, ChromeRestore e ChromeClose di Segoe Fluent Icons. */
const GLYPH = {
  close: "",
  maximize: "",
  minimize: "",
  restore: "",
};

const BUTTON =
  "flex h-full w-[46px] items-center justify-center font-caption text-[10px] leading-none transition-colors duration-100 focus-visible:outline-none";
/** Il velo di Windows 11 (4–6%); premuto, più leggero e con il glifo attenuato. */
const SUBTLE = `${BUTTON} hover:bg-foreground/5 focus-visible:bg-foreground/5 active:bg-foreground/3 active:text-foreground/60`;
const CLOSE = `${BUTTON} hover:bg-close hover:text-white focus-visible:bg-close focus-visible:text-white active:bg-close/90 active:text-white/70`;

/**
 * Riduci a icona, Ingrandisci/Ripristina e Chiudi, come quelli di Windows 11: gli stessi glifi, e
 * spenti quando la finestra non è attiva. La finestra non ha la cornice di sistema. Restano sopra
 * tutto, anche sopra Impostazioni.
 */
export function WindowControls() {
  const { t } = useTranslation();
  const [maximized, setMaximized] = useState(false);
  const [focused, setFocused] = useState(true);

  useEffect(() => {
    const win = getCurrentWindow();
    const update = () => win.isMaximized().then(setMaximized);
    update();
    win.isFocused().then(setFocused);
    const resized = win.onResized(update);
    const focus = win.onFocusChanged(({ payload }) => setFocused(payload));
    return () => {
      resized.then((stop) => stop());
      focus.then((stop) => stop());
    };
  }, []);

  const minimize = useCallback(() => getCurrentWindow().minimize(), []);
  const toggle = useCallback(() => getCurrentWindow().toggleMaximize(), []);
  const close = useCallback(() => getCurrentWindow().close(), []);
  const resize = maximized ? t("window.restore") : t("window.maximize");

  return (
    <div
      className={`fixed top-0 right-0 z-50 flex h-8 ${focused ? "text-foreground" : "text-foreground/40"}`}
    >
      <button
        aria-label={t("window.minimize")}
        className={SUBTLE}
        onClick={minimize}
        tabIndex={-1}
        title={t("window.minimize")}
        type="button"
      >
        {GLYPH.minimize}
      </button>
      <button
        aria-label={resize}
        className={SUBTLE}
        onClick={toggle}
        tabIndex={-1}
        title={resize}
        type="button"
      >
        {maximized ? GLYPH.restore : GLYPH.maximize}
      </button>
      <button
        aria-label={t("window.close")}
        className={CLOSE}
        onClick={close}
        tabIndex={-1}
        title={t("window.close")}
        type="button"
      >
        {GLYPH.close}
      </button>
    </div>
  );
}
