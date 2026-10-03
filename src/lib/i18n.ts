import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import de from "@/locales/de.json";
import en from "@/locales/en.json";
import es from "@/locales/es.json";
import fr from "@/locales/fr.json";
import it from "@/locales/it.json";
import pl from "@/locales/pl.json";

// Modulo con effetti collaterali: va importato prima del primo render. Parte in italiano, la
// lingua di riferimento; `main.tsx` passa alla Lingua dell'interfaccia prima del render.
i18n.use(initReactI18next).init({
  fallbackLng: "en",
  // Risorse inline: init sincrono, `t` è pronto subito.
  initAsync: false,
  // React fa già l'escape.
  interpolation: { escapeValue: false },
  lng: "it",
  resources: {
    de: { translation: de },
    en: { translation: en },
    es: { translation: es },
    fr: { translation: fr },
    it: { translation: it },
    pl: { translation: pl },
  },
});
