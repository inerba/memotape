import {
  type ChangeEvent,
  type KeyboardEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { nameProblem } from "@/features/library/library";

/**
 * Il campo del nome di una Raccolta o del titolo di un Tape: Invio conferma un nome valido, Esc o
 * l'uscita dal campo annullano. Sotto, perché il nome non va.
 */
export function NameInput({
  className = "h-8",
  initial,
  label,
  onCancel,
  onSubmit,
  taken,
}: {
  /** Le classi del campo. */
  className?: string;
  initial: string;
  label: string;
  onCancel: () => void;
  onSubmit: (name: string) => void;
  /** I nomi già usati accanto, escluso quello attuale. */
  taken: string[];
}) {
  const { t } = useTranslation();
  const [name, setName] = useState(initial);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    input.current?.focus();
    input.current?.select();
  }, []);
  const problem = nameProblem(name, taken);
  const change = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => setName(e.target.value),
    []
  );
  const keyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter" && !problem) {
        if (name === initial) {
          onCancel();
        } else {
          onSubmit(name);
        }
      } else if (e.key === "Escape") {
        onCancel();
      }
    },
    [initial, name, onCancel, onSubmit, problem]
  );
  return (
    <div className="flex min-w-0 flex-col gap-1">
      <Input
        aria-invalid={problem !== null}
        aria-label={label}
        className={className}
        onBlur={onCancel}
        onChange={change}
        onKeyDown={keyDown}
        ref={input}
        value={name}
      />
      {problem && name !== "" ? (
        <span className="text-destructive text-xs" role="alert">
          {t(`library.names.${problem}`)}
        </span>
      ) : null}
    </div>
  );
}
