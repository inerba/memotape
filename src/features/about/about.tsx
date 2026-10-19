import { type SyntheticEvent, useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { BrandMark } from "@/components/brand-mark";
import { CREDITS, type Credit } from "@/features/about/credits";

const DIR = "../../../src-tauri/resources/licenses";
// Dentro `RELEASES_PREFIX` di `updates.rs`: `open_update` lo accetta.
const REPO_URL = "https://github.com/inerba/memotape/";
const openRepo = () => commands.openUpdate(REPO_URL);

// I testi sono le stesse risorse che l'installer mette accanto all'app; Vite li carica solo quando
// si apre un testo.
const LICENSE_TEXTS = import.meta.glob<string>(
  "../../../src-tauri/resources/licenses/*.txt",
  { import: "default", query: "?raw" }
);

/** Impostazioni → Informazioni: il logo con la versione dell'app e le licenze dei componenti. */
export function About() {
  const { t } = useTranslation();
  const [version, setVersion] = useState("");

  useEffect(() => {
    commands.appVersion().then(setVersion);
  }, []);

  return (
    <div className="flex max-w-3xl flex-col gap-3">
      <div className="mb-3 flex items-center gap-4">
        <BrandMark className="size-16" />
        <div>
          <p className="font-display font-medium text-[1.75rem] leading-tight tracking-[-0.01em]">
            memotape
          </p>
          {version ? (
            <p className="text-muted-foreground text-sm">
              {t("about.version", { version })}
            </p>
          ) : null}
          <button
            aria-label={t("about.repo")}
            className="mt-1 rounded-sm text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
            onClick={openRepo}
            title={t("about.repo")}
            type="button"
          >
            {/* Il marchio di GitHub (Octicons `mark-github`, MIT). */}
            <svg
              aria-hidden="true"
              className="size-5"
              fill="currentColor"
              viewBox="0 0 16 16"
            >
              <path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z" />
            </svg>
          </button>
        </div>
      </div>
      <ul className="flex flex-col divide-y rounded-md border">
        {CREDITS.map((credit) => (
          <CreditRow credit={credit} key={credit.name} />
        ))}
      </ul>
    </div>
  );
}

function CreditRow({ credit }: { credit: Credit }) {
  const { t } = useTranslation();
  const { author, file, license, name, notices, quantization, role, url } =
    credit;
  return (
    <li className="flex flex-col gap-1 p-4">
      <div className="flex flex-wrap items-baseline gap-x-2">
        <span className="font-medium">{name}</span>
        <span className="text-muted-foreground text-sm">
          {[author, t("models.license", { license })]
            .filter(Boolean)
            .join(" · ")}
        </span>
      </div>
      <p className="text-muted-foreground text-sm">
        {t(`about.roles.${role}`)}
      </p>
      {quantization ? (
        <p className="text-muted-foreground text-sm">
          {t("about.converted", { quantization })}
        </p>
      ) : null}
      <p className="select-text break-all text-muted-foreground text-sm">
        {t("about.source", { url })}
      </p>
      <LicenseText file={file} label={t("about.licenseText")} />
      {notices ? (
        <LicenseText file={notices} label={t("about.thirdParty")} />
      ) : null}
    </li>
  );
}

/** Un testo di licenza, caricato alla prima apertura. */
function LicenseText({ file, label }: { file: string; label: string }) {
  const [text, setText] = useState<string | null>(null);
  const load = useCallback(
    (e: SyntheticEvent<HTMLDetailsElement>) => {
      if (e.currentTarget.open && text === null) {
        LICENSE_TEXTS[`${DIR}/${file}`]?.().then(setText);
      }
    },
    [file, text]
  );
  return (
    <details className="text-sm" onToggle={load}>
      <summary className="w-fit cursor-pointer rounded-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
        {label}
      </summary>
      <pre className="mt-2 max-h-80 select-text overflow-auto whitespace-pre-wrap rounded-md border bg-muted p-3 font-mono text-xs">
        {text}
      </pre>
    </details>
  );
}
