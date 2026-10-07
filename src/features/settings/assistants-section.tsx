import { Check, Copy } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { type AppError, commands } from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  type AssistantConfig,
  assistantConfigs,
} from "@/features/settings/assistants";
import { SettingSwitch } from "@/features/settings/setting-switch";

const CONFIGS: AssistantConfig[] = ["claudeCode", "codex", "claudeDesktop"];
/** Quanto resta "Copiato" sul pulsante. */
const COPIED_MS = 2000;

/** Impostazioni → Assistenti: il consenso a leggere la Libreria e come collegare un Assistente. */
export function AssistantsSection({
  onError,
}: {
  onError: (error: AppError) => void;
}) {
  const { t } = useTranslation();
  const [exe, setExe] = useState<string | null>(null);

  useEffect(() => {
    commands.appExe().then((result) => {
      if (result.status === "ok") {
        setExe(result.data);
      } else {
        onError(result.error);
      }
    });
  }, [onError]);

  const configs = exe === null ? null : assistantConfigs(exe);
  return (
    <>
      <div className="flex max-w-xl flex-col gap-1">
        <SettingSwitch
          description={t("settings.assistants.allowDescription")}
          label={t("settings.assistants.allow")}
          name="assistenti"
          onError={onError}
        />
      </div>
      <div className="flex max-w-xl flex-col gap-1">
        <h3 className="font-medium text-sm">
          {t("settings.assistants.connect")}
        </h3>
        <p className="text-muted-foreground text-sm">
          {t("settings.assistants.connectDescription")}
        </p>
      </div>
      {configs
        ? CONFIGS.map((key) => (
            <ConfigBlock
              key={key}
              label={t(`settings.assistants.${key}`)}
              value={configs[key]}
            />
          ))
        : null}
    </>
  );
}

function ConfigBlock({ label, value }: { label: string; value: string }) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = setTimeout(() => setCopied(false), COPIED_MS);
    return () => clearTimeout(timer);
  }, [copied]);
  const copy = useCallback(async () => {
    await navigator.clipboard.writeText(value);
    setCopied(true);
  }, [value]);
  return (
    <div className="flex max-w-xl flex-col gap-2">
      <div className="flex items-center justify-between gap-2">
        <span className="text-sm">{label}</span>
        <Button className="h-8 gap-2" onClick={copy} variant="outline">
          {copied ? <Check className="text-play" /> : <Copy />}
          {copied
            ? t("settings.assistants.copied")
            : t("settings.assistants.copy")}
        </Button>
      </div>
      <pre className="overflow-x-auto rounded-md border bg-muted px-3 py-2 font-mono text-xs">
        {value}
      </pre>
    </div>
  );
}
