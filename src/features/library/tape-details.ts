import type { TFunction } from "i18next";
import type { TapeInfo } from "@/bindings";
import { speechLanguageName } from "@/features/settings/settings";

/**
 * Le righe del popover Dettagli di un Tape: da dove viene, modello e Lingua del parlato (solo se
 * ha testo) e Ingressi. Il resto (Parlanti, correzioni, avvisi) resta visibile sotto il titolo.
 */
export function tapeDetails(
  info: TapeInfo,
  t: TFunction,
  locale: string
): { label: string; value: string }[] {
  const rows = [
    {
      label: t("tape.details.origine"),
      value: info.origine ?? t("tape.recording"),
    },
  ];
  if (info.modello) {
    rows.push(
      { label: t("tape.details.modello"), value: info.modello },
      {
        label: t("speechLanguage.label"),
        value:
          info.linguaParlato === "auto"
            ? t("speechLanguage.auto")
            : speechLanguageName(info.linguaParlato, locale),
      }
    );
  }
  rows.push({
    label: t("tape.details.ingressi"),
    value: info.ingressiSeparati ? t("tape.ingressi") : t("tape.details.mix"),
  });
  return rows;
}
