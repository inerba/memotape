import { expect, test } from "bun:test";
import type { AppError, Settings } from "@/bindings";
import { DEFAULT_SETTINGS } from "./settings";
import { createSettingsWriter } from "./settings-writer";

test("azioni contemporanee di barra e Impostazioni conservano entrambi i campi e il Guadagno", async () => {
  let disk = DEFAULT_SETTINGS;
  let visible = disk;
  const writer = createSettingsWriter(
    disk,
    async (next) => {
      await new Promise((resolve) => setTimeout(resolve, 1));
      disk = next;
      return { data: disk, status: "ok" };
    },
    (next) => {
      visible = next;
    }
  );
  const cleaning = writer.save((current) => ({
    ...current,
    audioMicrofono: { ...current.audioMicrofono, pulizia: true },
  }));
  const sensitivity = writer.save((current) => ({
    ...current,
    audioMicrofono: { ...current.audioMicrofono, sensibilita: "spento" },
  }));
  const gain = writer.save((current) => ({ ...current, guadagnoSistema: 6 }));
  expect(visible.audioMicrofono).toEqual({
    pulizia: true,
    sensibilita: "spento",
  });
  expect(await Promise.all([cleaning, sensitivity, gain])).toEqual([
    null,
    null,
    null,
  ]);
  expect(disk.audioMicrofono).toEqual({ pulizia: true, sensibilita: "spento" });
  expect(disk.guadagnoSistema).toBe(6);
  expect(visible).toEqual(disk);
  // Un nuovo avvio riceve i valori persistiti, senza dipendere dalla superficie.
  let restarted: Settings = DEFAULT_SETTINGS;
  const second = createSettingsWriter(
    disk,
    async (next) => ({ data: next, status: "ok" }),
    (next) => {
      restarted = next;
    }
  );
  await second.save((current) => ({
    ...current,
    audioSistema: { ...current.audioSistema, pulizia: true },
  }));
  expect(restarted.audioMicrofono).toEqual({
    pulizia: true,
    sensibilita: "spento",
  });
  expect(restarted.audioSistema?.pulizia).toBe(true);
});

test("un salvataggio rifiutato si ritira senza perdere la scelta successiva e blocca l'avvio", async () => {
  let visible = DEFAULT_SETTINGS;
  let first = true;
  const error = { code: "unwritableFolder" as const, detail: "D:\\prove" };
  const writer = createSettingsWriter(
    DEFAULT_SETTINGS,
    async (next) => {
      await new Promise((resolve) => setTimeout(resolve, 1));
      if (first) {
        first = false;
        return { error, status: "error" };
      }
      return { data: next, status: "ok" };
    },
    (next) => {
      visible = next;
    }
  );
  const cleaning = writer.save((current) => ({
    ...current,
    audioMicrofono: { ...current.audioMicrofono, pulizia: true },
  }));
  const sensitivity = writer.save((current) => ({
    ...current,
    audioMicrofono: { ...current.audioMicrofono, sensibilita: "sensibile" },
  }));
  expect(await writer.flush()).toEqual(error);
  expect(await cleaning).toEqual(error);
  expect(await sensitivity).toBeNull();
  expect(visible.audioMicrofono).toEqual({
    pulizia: false,
    sensibilita: "sensibile",
  });
});

test("i cambi rapidi dello stesso controllo mantengono l'ultima scelta e l'avvio aspetta la persistenza", async () => {
  let disk = DEFAULT_SETTINGS;
  let visible = disk;
  const writer = createSettingsWriter(
    disk,
    async (next) => {
      await new Promise((resolve) => setTimeout(resolve, 1));
      disk = next;
      return { data: disk, status: "ok" };
    },
    (next) => {
      visible = next;
    }
  );
  writer.save((current) => ({
    ...current,
    audioSistema: { ...current.audioSistema, pulizia: true },
  }));
  writer.save((current) => ({
    ...current,
    audioSistema: { ...current.audioSistema, pulizia: false },
  }));
  writer.save((current) => ({
    ...current,
    audioSistema: { ...current.audioSistema, sensibilita: "selettivo" },
  }));
  expect(await writer.flush()).toBeNull();
  expect(disk.audioSistema).toEqual({
    pulizia: false,
    sensibilita: "selettivo",
  });
  expect(visible).toEqual(disk);
});

test("un errore di trasporto ripristina il valore confermato e permette di riprovare", async () => {
  let visible = DEFAULT_SETTINGS;
  let first = true;
  const writer = createSettingsWriter(
    DEFAULT_SETTINGS,
    async (next) => {
      await Promise.resolve();
      if (first) {
        first = false;
        throw new Error("trasporto");
      }
      return { data: next, status: "ok" };
    },
    (next) => {
      visible = next;
    }
  );
  const choose = (current: Settings): Settings => ({
    ...current,
    audioMicrofono: { ...current.audioMicrofono, pulizia: true },
  });
  expect((await writer.save(choose))?.code).toBe("internal");
  expect(visible.audioMicrofono?.pulizia).toBe(false);
  expect(await writer.save(choose)).toBeNull();
  expect(visible.audioMicrofono?.pulizia).toBe(true);
});

test("Registra attende anche una scelta aggiunta mentre sta aspettando il primo salvataggio", async () => {
  let disk = DEFAULT_SETTINGS;
  const writer = createSettingsWriter(
    disk,
    async (next) => {
      await new Promise((resolve) => setTimeout(resolve, 5));
      disk = next;
      return { data: next, status: "ok" };
    },
    () => undefined
  );
  writer.save((current) => ({
    ...current,
    audioMicrofono: { ...current.audioMicrofono, pulizia: true },
  }));
  const starting = writer.flush();
  await Promise.resolve();
  writer.save((current) => ({
    ...current,
    audioSistema: { ...current.audioSistema, sensibilita: "selettivo" },
  }));
  expect(await starting).toBeNull();
  expect(disk.audioSistema?.sensibilita).toBe("selettivo");
});

test("il riscontro di una scelta vecchia non sostituisce quello della scelta più recente", async () => {
  const results: ((
    result:
      | { status: "ok"; data: Settings }
      | { status: "error"; error: AppError }
  ) => void)[] = [];
  const states: { status: string; revision: number }[] = [];
  const writer = createSettingsWriter(
    DEFAULT_SETTINGS,
    () => new Promise((resolve) => results.push(resolve)),
    () => undefined,
    (_field, state) => {
      if (state) {
        states.push(state);
      }
    }
  );
  const first = writer.save(
    (current) => ({ ...current, guadagnoMicrofono: 3 }),
    "guadagnoMicrofono"
  );
  const second = writer.save(
    (current) => ({ ...current, guadagnoMicrofono: 6 }),
    "guadagnoMicrofono"
  );
  results[0]?.({
    error: { code: "internal", detail: "test" },
    status: "error",
  });
  await first;
  expect(states.map((state) => state.status)).toEqual(["saving", "saving"]);
  results[1]?.({
    data: { ...DEFAULT_SETTINGS, guadagnoMicrofono: 6 },
    status: "ok",
  });
  await second;
  expect(states.map((state) => state.status)).toEqual([
    "saving",
    "saving",
    "saved",
  ]);
  const [lastState] = states.slice(-1);
  expect(lastState?.revision).toBe(states[1]?.revision);
});

test("Riprova salva solo la scelta fallita e conserva i successi degli altri campi", async () => {
  let first = true;
  let disk = DEFAULT_SETTINGS;
  const states = new Map<string, string>();
  const writer = createSettingsWriter(
    disk,
    async (next) => {
      await Promise.resolve();
      if (first) {
        first = false;
        return {
          error: { code: "unwritableFolder", detail: "test" },
          status: "error",
        };
      }
      disk = next;
      return { data: next, status: "ok" };
    },
    () => undefined,
    (field, state) => {
      if (state) {
        states.set(field, state.status);
      }
    }
  );
  await writer.save(
    (current) => ({
      ...current,
      audioMicrofono: { ...current.audioMicrofono, pulizia: true },
    }),
    "audioMicrofono.pulizia"
  );
  await writer.save(
    (current) => ({ ...current, guadagnoSistema: 9 }),
    "guadagnoSistema"
  );
  expect(states.get("audioMicrofono.pulizia")).toBe("error");
  expect(states.get("guadagnoSistema")).toBe("saved");
  expect(await writer.retry("audioMicrofono.pulizia")).toBeNull();
  expect(disk.audioMicrofono?.pulizia).toBe(true);
  expect(disk.guadagnoSistema).toBe(9);
  expect(states.get("audioMicrofono.pulizia")).toBe("saved");
});

test("una nuova scelta invalida la riprova precedente e la scadenza di una conferma vecchia", async () => {
  let first = true;
  let visible = DEFAULT_SETTINGS;
  const states: ({ revision: number; status: string } | undefined)[] = [];
  const writer = createSettingsWriter(
    DEFAULT_SETTINGS,
    async (next) => {
      await Promise.resolve();
      if (first) {
        first = false;
        return { error: { code: "internal", detail: "test" }, status: "error" };
      }
      return { data: next, status: "ok" };
    },
    (next) => {
      visible = next;
    },
    (_field, state) => states.push(state)
  );
  await writer.save(
    (current) => ({ ...current, guadagnoMicrofono: 3 }),
    "guadagnoMicrofono"
  );
  const [oldState] = states.slice(-1);
  const oldRevision = oldState?.revision ?? 0;
  const latestChoice = writer.save(
    (next) => ({ ...next, guadagnoMicrofono: 9 }),
    "guadagnoMicrofono"
  );
  const count = states.length;
  writer.clearFeedback("guadagnoMicrofono", oldRevision);
  expect(states).toHaveLength(count);
  await latestChoice;
  await writer.retry("guadagnoMicrofono");
  expect(visible.guadagnoMicrofono).toBe(9);
  expect(states).toHaveLength(count + 1);
});
