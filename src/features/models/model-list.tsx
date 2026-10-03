import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands, type ModelInfo, type ModelKind } from "@/bindings";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { mebibytes, stateText } from "@/features/models/models";
import { useModels } from "@/features/models/use-models";
import { errorText } from "@/features/status/status";

/**
 * I modelli del catalogo di un tipo: download, Annulla ed Elimina, e per i modelli di trascrizione
 * la scelta (`selected`). Il download prosegue anche fuori da qui.
 */
export function ModelList({
  kind,
  onSelect,
  selected,
}: {
  kind: ModelKind;
  onSelect?: (id: string) => void;
  selected?: string;
}) {
  const { t } = useTranslation();
  const models = useModels().filter((m) => m.kind === kind);
  const [toDelete, setToDelete] = useState<ModelInfo | null>(null);

  // Esito ed errori arrivano con `model-state-changed`.
  const download = useCallback((id: string) => commands.downloadModel(id), []);

  const confirmDelete = useCallback(() => {
    if (toDelete) {
      commands.deleteModel(toDelete.id);
    }
  }, [toDelete]);

  const closeDelete = useCallback((open: boolean) => {
    if (!open) {
      setToDelete(null);
    }
  }, []);

  return (
    <>
      <ul
        aria-label={
          kind === "trascrizione"
            ? t("models.choose")
            : t("settings.transcription.diarization")
        }
        className="flex flex-col divide-y rounded-md border"
      >
        {models.map((model) => (
          <ModelRow
            key={model.id}
            model={model}
            onCancel={commands.cancelModelDownload}
            onDelete={setToDelete}
            onDownload={download}
            onSelect={onSelect}
            selected={model.id === selected}
          />
        ))}
      </ul>
      <AlertDialog onOpenChange={closeDelete} open={toDelete !== null}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t("models.delete.title", { name: toDelete?.name })}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t("models.delete.description")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("models.delete.keep")}</AlertDialogCancel>
            <AlertDialogAction onClick={confirmDelete}>
              {t("models.delete.confirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

function ModelRow({
  model,
  onCancel,
  onDelete,
  onDownload,
  onSelect,
  selected,
}: {
  model: ModelInfo;
  onCancel: (id: string) => void;
  onDelete: (model: ModelInfo) => void;
  onDownload: (id: string) => void;
  onSelect?: (id: string) => void;
  selected: boolean;
}) {
  const { t } = useTranslation();
  const { state } = model.state;
  const cancel = useCallback(() => onCancel(model.id), [onCancel, model.id]);
  const remove = useCallback(() => onDelete(model), [onDelete, model]);
  const download = useCallback(
    () => onDownload(model.id),
    [onDownload, model.id]
  );
  const select = useCallback(() => onSelect?.(model.id), [onSelect, model.id]);
  const radio = `model-${model.id}`;

  return (
    <li className="flex flex-col gap-2 p-4">
      <div className="flex items-center gap-3">
        {onSelect ? (
          <input
            checked={selected}
            className="size-4 shrink-0 accent-primary"
            id={radio}
            name="model"
            onChange={select}
            type="radio"
            value={model.id}
          />
        ) : null}
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <label className="font-medium" htmlFor={radio}>
              {model.name}
            </label>
            {model.recommended ? (
              <span className="rounded-sm bg-primary px-1.5 py-0.5 font-medium text-primary-foreground text-xs">
                {t("models.recommended")}
              </span>
            ) : null}
          </div>
          <p className="text-muted-foreground text-sm">
            {[
              t("models.size", { mb: mebibytes(model.size) }),
              model.mode ? t(`models.mode.${model.mode}`) : null,
              t("models.license", { license: model.license }),
            ]
              .filter(Boolean)
              .join(" · ")}
          </p>
        </div>
        {state === "notDownloaded" ? (
          <Button onClick={download} size="sm">
            {t("models.download")}
          </Button>
        ) : null}
        {state === "interrupted" ? (
          <Button onClick={download} size="sm">
            {t("models.resume")}
          </Button>
        ) : null}
        {state === "downloading" ? (
          <Button onClick={cancel} size="sm" variant="outline">
            {t("models.cancel")}
          </Button>
        ) : null}
        {state === "interrupted" || state === "downloaded" ? (
          <Button
            disabled={model.inUse}
            onClick={remove}
            size="sm"
            variant="outline"
          >
            {t("models.delete.action")}
          </Button>
        ) : null}
      </div>
      <div className="flex items-center gap-3 text-sm">
        <span
          className={
            state === "downloaded" ? "text-foreground" : "text-muted-foreground"
          }
        >
          {model.inUse
            ? `${stateText(model.state, t)} · ${t("models.inUse")}`
            : stateText(model.state, t)}
        </span>
        {model.state.state === "downloading" ? (
          <progress
            aria-label={t("models.progress", { name: model.name })}
            className="h-1.5 flex-1 accent-primary"
            max={100}
            value={model.state.percent}
          />
        ) : null}
      </div>
      {model.error ? (
        <p className="text-destructive text-sm" role="alert">
          {errorText(model.error, t)}
        </p>
      ) : null}
    </li>
  );
}
