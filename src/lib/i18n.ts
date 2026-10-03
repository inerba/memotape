import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import it from "@/locales/it.json";

// ponytail: solo italiano; le altre lingue e la scelta da impostazioni arrivano con il ticket 10.
// Modulo con effetti collaterali: va importato prima del primo render.
i18n.use(initReactI18next).init({
  fallbackLng: "it",
  // Risorse inline: init sincrono, `t` è pronto subito.
  initAsync: false,
  // React fa già l'escape.
  interpolation: { escapeValue: false },
  lng: "it",
  resources: { it: { translation: it } },
});
