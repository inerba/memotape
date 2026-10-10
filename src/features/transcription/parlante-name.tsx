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

/**
 * Le classi di ogni colore di voce (l'indice di `voiceColors`), scritte per intero perché Tailwind
 * le trovi: il pallino, l'anello del nodo del Nastro e la sottolineatura del nome nell'Intervista.
 */
export const VOICE_CLASSES = [
  { dot: "bg-voice-1", ring: "ring-voice-1", underline: "decoration-voice-1" },
  { dot: "bg-voice-2", ring: "ring-voice-2", underline: "decoration-voice-2" },
  { dot: "bg-voice-3", ring: "ring-voice-3", underline: "decoration-voice-3" },
  { dot: "bg-voice-4", ring: "ring-voice-4", underline: "decoration-voice-4" },
  { dot: "bg-voice-5", ring: "ring-voice-5", underline: "decoration-voice-5" },
  { dot: "bg-voice-6", ring: "ring-voice-6", underline: "decoration-voice-6" },
] as const;

/** Il pallino del colore di una voce; senza colore (il mix, il Parlante non determinato) un cerchio vuoto. */
export function VoiceDot({ color }: { color: number | undefined }) {
  return (
    <span
      aria-hidden
      className={`size-2.5 shrink-0 rounded-full ${
        color === undefined
          ? "border border-muted-foreground/50"
          : VOICE_CLASSES[color]?.dot
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
