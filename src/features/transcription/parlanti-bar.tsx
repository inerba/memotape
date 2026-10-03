import {
  type ChangeEvent,
  type KeyboardEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { nomeTaken, type Parlante } from "@/features/transcription/phrases";

/**
 * I Parlanti del testo: un clic su uno (o sulla sua etichetta nell'area) lo rinomina sul posto, con
 * Invio per confermare ed Esc per annullare.
 */
export function ParlantiBar({
  editing,
  list,
  onEdit,
  onRename,
}: {
  editing: Parlante | null;
  list: Parlante[];
  onEdit: (voce: Parlante | null) => void;
  onRename: (voce: Parlante, nome: string) => void;
}) {
  const { t } = useTranslation();
  if (list.length === 0) {
    return null;
  }
  return (
    <div className="flex min-w-0 flex-wrap items-center gap-2">
      <span className="text-muted-foreground text-sm">
        {t("transcription.parlantiList")}
      </span>
      {list.map((voce) => (
        <ParlanteChip
          editing={
            voce.ingresso === editing?.ingresso &&
            voce.parlante === editing.parlante
          }
          key={`${voce.ingresso}:${voce.parlante}`}
          list={list}
          onEdit={onEdit}
          onRename={onRename}
          voce={voce}
        />
      ))}
    </div>
  );
}

/**
 * Un Parlante: l'etichetta, o in modifica il suo nome, che non si conferma vuoto né uguale a quello
 * di un altro Parlante del suo Ingresso.
 */
function ParlanteChip({
  editing,
  list,
  onEdit,
  onRename,
  voce,
}: {
  editing: boolean;
  list: Parlante[];
  onEdit: (voce: Parlante | null) => void;
  onRename: (voce: Parlante, nome: string) => void;
  voce: Parlante;
}) {
  const { t } = useTranslation();
  const [nome, setNome] = useState(voce.nome);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (editing) {
      setNome(voce.nome);
      input.current?.focus();
      input.current?.select();
    }
  }, [editing, voce.nome]);
  const start = useCallback(() => onEdit(voce), [onEdit, voce]);
  // Uscire dal campo annulla, come Esc.
  const cancel = useCallback(() => onEdit(null), [onEdit]);
  const change = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => setNome(e.target.value),
    []
  );
  const taken = nomeTaken(list, voce, nome.trim());
  const valid = nome.trim() !== "" && !taken;
  const keyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter" && valid) {
        onRename(voce, nome.trim());
      } else if (e.key === "Escape") {
        cancel();
      }
    },
    [cancel, nome, onRename, valid, voce]
  );
  if (!editing) {
    return (
      <Button
        onClick={start}
        size="sm"
        title={t("transcription.rename")}
        variant="outline"
      >
        {voce.label}
      </Button>
    );
  }
  return (
    <Input
      aria-invalid={!valid}
      aria-label={t("transcription.renameName", { label: voce.label })}
      className="h-8 w-40"
      onBlur={cancel}
      onChange={change}
      onKeyDown={keyDown}
      ref={input}
      title={t(taken ? "transcription.renameTaken" : "transcription.rename")}
      value={nome}
    />
  );
}
