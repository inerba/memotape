import "@/lib/i18n";

import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/app";
import { commands } from "@/bindings";

// Le impostazioni si leggono prima del primo render: nessun valore predefinito compare per un attimo.
commands.getSettings().then((settings) => {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App settings={settings} />
    </React.StrictMode>
  );
});
