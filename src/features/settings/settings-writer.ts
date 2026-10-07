import type { AppError, Settings } from "@/bindings";

export type SettingsChange = (current: Settings) => Settings;

export type SettingField =
  | keyof Settings
  | `${"audioMicrofono" | "audioSistema" | "audioFileMisto"}.${"pulizia" | "sensibilita"}`;

export type SaveFeedback = {
  revision: number;
} & ({ status: "saving" | "saved" } | { status: "error"; error: AppError });

/** Serializza intenzioni, non snapshot obsoleti. Le due superfici vedono anche le scelte in coda. */
export function createSettingsWriter(
  initial: Settings,
  persist: (
    next: Settings
  ) => Promise<
    { status: "ok"; data: Settings } | { status: "error"; error: AppError }
  >,
  publish: (next: Settings) => void,
  publishFeedback: (
    field: SettingField,
    feedback?: SaveFeedback
  ) => void = () => undefined
) {
  let saved = initial;
  const pending: {
    change: SettingsChange;
    resolve: (error: AppError | null) => void;
    field?: SettingField;
    revision: number;
  }[] = [];
  let revision = 0;
  const latest = new Map<SettingField, number>();
  const failed = new Map<SettingField, SettingsChange>();
  let running = false;
  const inFlight = new Set<Promise<AppError | null>>();
  const show = () =>
    publish(pending.reduce((current, item) => item.change(current), saved));
  const drain = async () => {
    if (running) {
      return;
    }
    running = true;
    while (pending.length) {
      const [item] = pending;
      if (!item) {
        break;
      }
      let error: AppError | null = null;
      try {
        // biome-ignore lint/performance/noAwaitInLoops: le scritture devono essere sequenziali per conservare ogni scelta.
        const result = await persist(item.change(saved));
        if (result.status === "ok") {
          saved = result.data;
        } else {
          ({ error } = result);
        }
      } catch (cause) {
        error = { code: "internal", detail: String(cause) };
      }
      pending.shift();
      show();
      if (item.field && latest.get(item.field) === item.revision) {
        if (error) {
          failed.set(item.field, item.change);
          publishFeedback(item.field, {
            error,
            revision: item.revision,
            status: "error",
          });
        } else {
          failed.delete(item.field);
          publishFeedback(item.field, {
            revision: item.revision,
            status: "saved",
          });
        }
      }
      item.resolve(error);
    }
    running = false;
  };
  const save = (
    change: SettingsChange,
    field?: SettingField
  ): Promise<AppError | null> => {
    revision += 1;
    const currentRevision = revision;
    if (field) {
      latest.set(field, currentRevision);
      failed.delete(field);
      publishFeedback(field, { revision: currentRevision, status: "saving" });
    }
    const task = new Promise<AppError | null>((resolve) => {
      pending.push({ change, field, resolve, revision: currentRevision });
      show();
      drain();
    });
    inFlight.add(task);
    task.then(() => inFlight.delete(task));
    return task;
  };
  return {
    clearFeedback: (field: SettingField, currentRevision: number) => {
      if (latest.get(field) === currentRevision) {
        publishFeedback(field);
      }
    },
    /** Attende i salvataggi già richiesti prima di avviare una Registrazione. */
    flush: async () => {
      let failure: AppError | null = null;
      while (inFlight.size) {
        // biome-ignore lint/performance/noAwaitInLoops: include anche le scelte arrivate durante l'attesa prima di Registra.
        const results = await Promise.all([...inFlight]);
        failure ??= results.find((error) => error !== null) ?? null;
      }
      return failure;
    },
    retry: (field: SettingField): Promise<AppError | null> => {
      const change = failed.get(field);
      return change ? save(change, field) : Promise.resolve(null);
    },
    save,
  };
}
