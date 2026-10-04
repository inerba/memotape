import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import type { BinoInfo, LibraryList } from "@/bindings";
import { infoParts } from "@/features/library/library";
import { NameInput } from "@/features/library/name-input";
import { fileName, folderOf } from "@/features/source/file-name";

const EXTENSION = /\.bino$/i;

/** La voce del Bino `path` nella Libreria, se ci sta, senza distinguere maiuscole e minuscole. */
export function entryOf(library: LibraryList, path: string) {
  const lower = path.toLowerCase();
  return library.bini.find((b) => b.path.toLowerCase() === lower);
}

/** Il titolo del Bino `path`, che un clic rinomina, e la riga delle sue informazioni. */
export function BinoHeader({
  disabled,
  info,
  library,
  onRename,
  path,
}: {
  /** Ci lavora l'Attività in corso: niente rinomina. */
  disabled: boolean;
  info: BinoInfo | null;
  library: LibraryList;
  onRename: (path: string, titolo: string) => void;
  path: string;
}) {
  const { t } = useTranslation();
  const [renaming, setRenaming] = useState(false);
  const entry = entryOf(library, path);
  const titolo = entry?.titolo ?? fileName(path).replace(EXTENSION, "");
  const folder = folderOf(path.toLowerCase());
  const siblings = library.bini
    .filter((b) => b !== entry && folderOf(b.path.toLowerCase()) === folder)
    .map((b) => b.titolo);

  const start = useCallback(() => setRenaming(true), []);
  const cancel = useCallback(() => setRenaming(false), []);
  const submit = useCallback(
    (name: string) => {
      setRenaming(false);
      onRename(path, name);
    },
    [onRename, path]
  );

  return (
    <section className="flex min-w-0 flex-col gap-1">
      {renaming ? (
        <NameInput
          initial={titolo}
          label={t("library.renameName")}
          onCancel={cancel}
          onSubmit={submit}
          taken={siblings}
        />
      ) : (
        <button
          className="block max-w-full cursor-text truncate rounded-sm text-left font-medium text-lg hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-default disabled:no-underline"
          disabled={disabled}
          onClick={start}
          title={t("library.rename")}
          type="button"
        >
          {titolo}
        </button>
      )}
      {info ? (
        <InfoLine
          info={info}
          inLibrary={entry !== undefined}
          raccolta={entry?.raccolta}
        />
      ) : null}
    </section>
  );
}

/** Le informazioni in una riga, intere nel tooltip. */
function InfoLine({
  inLibrary,
  info,
  raccolta,
}: {
  inLibrary: boolean;
  info: BinoInfo;
  raccolta?: string | null;
}) {
  const { i18n, t } = useTranslation();
  const text = infoParts(
    info,
    inLibrary ? (raccolta ?? null) : undefined,
    t,
    i18n.language
  ).join(" · ");
  return (
    <p className="truncate text-muted-foreground text-sm" title={text}>
      {text}
    </p>
  );
}
