import { type ChangeEvent, useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands, events } from "@/bindings";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";

const PATH_SEPARATOR = /[\\/]/;

function fileName(path: string) {
  return path.split(PATH_SEPARATOR).pop() ?? path;
}

export function HomePage() {
  const { t } = useTranslation();
  const [source, setSource] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<AppError | null>(null);

  useEffect(() => {
    // Una Frase per riga, nell'ordine in cui arrivano.
    const unlisten = events.transcriptPhrase.listen(({ payload }) => {
      setText((current) =>
        current ? `${current}\n${payload.text}` : payload.text
      );
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  const browse = useCallback(async () => {
    const picked = await commands.pickSource(t("source.filter"));
    if (picked) {
      setSource(picked);
      setError(null);
    }
  }, [t]);

  const transcribe = useCallback(async () => {
    if (!source) {
      return;
    }
    setText("");
    setError(null);
    setRunning(true);
    const result = await commands.transcribe(source);
    setRunning(false);
    if (result.status === "error") {
      setError(result.error);
    }
  }, [source]);

  const edit = useCallback(
    (e: ChangeEvent<HTMLTextAreaElement>) => setText(e.target.value),
    []
  );

  return (
    <main className="flex h-screen flex-col gap-4 p-6">
      <div className="flex items-center gap-3">
        <Button disabled={running} onClick={browse} variant="outline">
          {t("source.browse")}
        </Button>
        <span className="min-w-0 flex-1 truncate text-sm" title={source ?? ""}>
          {source ? fileName(source) : t("source.none")}
        </span>
        <Button disabled={!source || running} onClick={transcribe}>
          {running ? t("transcription.running") : t("transcription.start")}
        </Button>
      </div>
      <Textarea
        aria-label={t("transcription.text")}
        className="flex-1 resize-none"
        onChange={edit}
        readOnly={running}
        value={text}
      />
      {error ? (
        <p className="text-destructive text-sm" role="alert">
          {t(`errors.codes.${error.code}`, { detail: error.detail })}
        </p>
      ) : null}
    </main>
  );
}
