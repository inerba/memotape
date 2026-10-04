import { type ChangeEvent, useCallback } from "react";
import { useTranslation } from "react-i18next";

export const SELECT =
  "h-9 min-w-0 rounded-md border border-input bg-transparent px-2 text-sm shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50";

/** Il valore della voce che fa da titolo: nessuna Raccolta inizia con un punto. */
const PROMPT = ".";

/**
 * "Sposta in…" (o "Aggiungi alla Libreria…" per un Bino da fuori): sceglie Senza raccolta o una
 * Raccolta, tranne quella in cui il Bino sta già (`current`, `""` Senza raccolta).
 */
export function MoveSelect({
  current,
  disabled,
  label,
  onMove,
  raccolte,
}: {
  current?: string;
  disabled?: boolean;
  label: string;
  onMove: (raccolta: string) => void;
  raccolte: string[];
}) {
  const { t } = useTranslation();
  const change = useCallback(
    (e: ChangeEvent<HTMLSelectElement>) => {
      if (e.target.value !== PROMPT) {
        onMove(e.target.value);
      }
    },
    [onMove]
  );
  return (
    <select
      aria-label={label}
      className={SELECT}
      disabled={disabled}
      onChange={change}
      value={PROMPT}
    >
      <option disabled value={PROMPT}>
        {label}
      </option>
      {current === "" ? null : <option value="">{t("library.none")}</option>}
      {raccolte
        .filter((r) => r !== current)
        .map((r) => (
          <option key={r} value={r}>
            {r}
          </option>
        ))}
    </select>
  );
}
