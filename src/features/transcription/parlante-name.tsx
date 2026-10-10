import {
  type ChangeEvent,
  type KeyboardEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { nomeTaken, type Parlante } from "@/features/transcription/phrases";

/** Le classi del pallino di una voce, per il colore da `voiceColors`. */
export const VOICE_DOTS = [
  "bg-voice-1",
  "bg-voice-2",
  "bg-voice-3",
  "bg-voice-4",
  "bg-voice-5",
  "bg-voice-6",
];

/** Il pallino del colore di una voce; senza colore (il mix, il Parlante non determinato) un cerchio vuoto. */
export function VoiceDot({ color }: { color: number | undefined }) {
  return (
    <span
      aria-hidden
      className={`size-2.5 shrink-0 rounded-full ${
        color === undefined
          ? "border border-muted-foreground/50"
          : VOICE_DOTS[color]
      }`}
    />
  );
}

/**
 * Il nome nuovo di un Parlante, scritto al suo posto: Invio conferma, Esc o l'uscita dal campo
 * annullano. Non si conferma vuoto né uguale a quello di un altro Parlante del suo Ingresso.
 */
export function ParlanteNameInput({
  className = "",
  list,
  onCancel,
  onRename,
  voce,
}: {
  className?: string;
  list: Parlante[];
  onCancel: () => void;
  onRename: (voce: Parlante, nome: string) => void;
  voce: Parlante;
}) {
  const { t } = useTranslation();
  const [nome, setNome] = useState(voce.nome);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    input.current?.focus({ preventScroll: true });
    input.current?.select();
  }, []);
  const change = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => setNome(e.target.value),
    []
  );
  const taken = nomeTaken(list, voce, nome.trim());
  const valid = nome.trim() !== "" && !taken;
  const keyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter" && valid) {
        if (nome.trim() === voce.nome) {
          onCancel();
        } else {
          onRename(voce, nome.trim());
        }
      } else if (e.key === "Escape") {
        onCancel();
      }
    },
    [nome, onCancel, onRename, valid, voce]
  );
  return (
    <input
      aria-invalid={!valid}
      aria-label={t("transcription.renameName", { label: voce.label })}
      className={`h-7 w-44 rounded-md border border-input bg-background px-2 font-medium text-sm outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/30 aria-[invalid=true]:border-destructive ${className}`}
      onBlur={onCancel}
      onChange={change}
      onKeyDown={keyDown}
      ref={input}
      title={t(taken ? "transcription.renameTaken" : "transcription.rename")}
      value={nome}
    />
  );
}
