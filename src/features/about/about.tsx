import { type SyntheticEvent, useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { COMPONENTS, type Component } from "@/features/about/components";

const DIR = "../../../src-tauri/resources/licenses";

// I testi sono le stesse risorse che l'installer mette accanto all'app; Vite li carica solo quando
// si apre un testo (gli avvisi di ONNX Runtime sono oltre 300 KB).
const LICENSE_TEXTS = import.meta.glob<string>(
  "../../../src-tauri/resources/licenses/*.txt",
  { import: "default", query: "?raw" }
);

/** Impostazioni → Informazioni: la versione dell'app e le licenze dei componenti. */
export function About() {
  const { t } = useTranslation();
  const [version, setVersion] = useState("");

  useEffect(() => {
    commands.appVersion().then(setVersion);
  }, []);

  return (
    <div className="flex max-w-3xl flex-col gap-3">
      <p className="text-sm">{t("about.version", { version })}</p>
      <ul className="flex flex-col divide-y rounded-md border">
        {COMPONENTS.map((component) => (
          <ComponentRow component={component} key={component.name} />
        ))}
      </ul>
    </div>
  );
}

function ComponentRow({ component }: { component: Component }) {
  const { t } = useTranslation();
  const { author, converted, file, license, name, notices, role, url } =
    component;
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
      {converted ? (
        <p className="text-muted-foreground text-sm">{t("about.converted")}</p>
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
