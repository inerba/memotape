import "./global.css";

import AppProvider from "@/app/provider";
import AppRouter from "@/app/router";
import type { Settings } from "@/bindings";

export default function App({ settings }: { settings: Settings }) {
  return (
    <AppProvider settings={settings}>
      <AppRouter />
    </AppProvider>
  );
}
