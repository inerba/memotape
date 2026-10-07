import { getCurrentWindow } from "@tauri-apps/api/window";

declare global {
  interface Window {
    __MEMOTAPE_STARTUP__?: {
      theme?: "light" | "dark" | null;
      window: { maximize: string; restore: string };
    };
  }
}

/** Solo i controlli della finestra: il preload e il suo timer non aspettano questo modulo. */
function setupWindowControls() {
  const preload = document.getElementById("startup");
  if (!preload) {
    return;
  }
  const win = getCurrentWindow();
  const minimize = document.getElementById("startup-minimize");
  const maximize = document.getElementById("startup-maximize");
  const close = document.getElementById("startup-close");

  minimize?.addEventListener("click", () => {
    win.minimize();
  });
  maximize?.addEventListener("click", () => {
    win.toggleMaximize();
  });
  close?.addEventListener("click", () => {
    win.close();
  });

  const update = async () => {
    const maximized = await win.isMaximized();
    const label = window.__MEMOTAPE_STARTUP__?.window;
    if (label && maximize) {
      maximize.setAttribute(
        "aria-label",
        maximized ? label.restore : label.maximize
      );
    }
  };
  update();
  const resized = win.onResized(update);
  window.addEventListener(
    "memotape-startup-ready",
    () => {
      resized.then((stop) => stop());
    },
    { once: true }
  );
}

setupWindowControls();

// Il bootstrap React è un chunk separato: i controlli funzionano anche mentre lo si carica.
import("./main").catch(() => {
  window.dispatchEvent(new Event("memotape-startup-error"));
});
