import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

/** La larghezza dei tre pulsanti: le barre del titolo lasciano libero questo spazio a destra. */
export const WINDOW_CONTROLS_PADDING = "pr-[138px]";

const BUTTON =
  "flex h-full w-[46px] items-center justify-center text-foreground/80 transition-colors hover:bg-foreground/8 focus-visible:bg-foreground/8 focus-visible:outline-none [&_svg]:size-4 [&_svg]:stroke-[1.25]";

/**
 * Riduci a icona, Ingrandisci/Ripristina e Chiudi, come quelli di Windows: la finestra non ha la
 * cornice di sistema. Restano sopra tutto, anche sopra Impostazioni.
 */
export function WindowControls() {
  const { t } = useTranslation();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const win = getCurrentWindow();
    const update = () => win.isMaximized().then(setMaximized);
    update();
    const resized = win.onResized(update);
    return () => {
      resized.then((stop) => stop());
    };
  }, []);

  const minimize = useCallback(() => getCurrentWindow().minimize(), []);
  const toggle = useCallback(() => getCurrentWindow().toggleMaximize(), []);
  const close = useCallback(() => getCurrentWindow().close(), []);
  const resize = maximized ? t("window.restore") : t("window.maximize");

  return (
    <div className="fixed top-0 right-0 z-50 flex h-9">
      <button
        aria-label={t("window.minimize")}
        className={BUTTON}
        onClick={minimize}
        tabIndex={-1}
        title={t("window.minimize")}
        type="button"
      >
        <Minus />
      </button>
      <button
        aria-label={resize}
        className={BUTTON}
        onClick={toggle}
        tabIndex={-1}
        title={resize}
        type="button"
      >
        {maximized ? <Copy className="-scale-x-100" /> : <Square />}
      </button>
      <button
        aria-label={t("window.close")}
        className={`${BUTTON} hover:bg-close hover:text-white focus-visible:bg-close focus-visible:text-white`}
        onClick={close}
        tabIndex={-1}
        title={t("window.close")}
        type="button"
      >
        <X />
      </button>
    </div>
  );
}
