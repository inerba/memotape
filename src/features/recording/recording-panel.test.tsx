import { expect, test } from "bun:test";
import { Tooltip } from "radix-ui";
import { renderToStaticMarkup } from "react-dom/server";
import "@/lib/i18n";
import { DEFAULT_SETTINGS } from "@/features/settings/settings";
import { SettingsProvider } from "@/features/settings/settings-context";
import { RecordingPanel } from "./recording-panel";

const ignore = () => undefined;

test("la barra offre solo i profili degli Ingressi reali, con default e descrizioni accessibili", () => {
  for (const recordingSource of ["mic", "system", "both"] as const) {
    const html = renderToStaticMarkup(
      <SettingsProvider
        initial={{ ...DEFAULT_SETTINGS, recordingSource }}
        loadError={null}
      >
        <Tooltip.Provider>
          <RecordingPanel
            onError={ignore}
            onPausedChange={ignore}
            paused={false}
            sessionId="current"
          />
        </Tooltip.Provider>
      </SettingsProvider>
    );
    expect(html.includes('name="audioMicrofono.pulizia"')).toBe(
      recordingSource !== "system"
    );
    expect(html.includes('name="audioSistema.sensibilita"')).toBe(
      recordingSource !== "mic"
    );
    expect(html).not.toContain("audioFileMisto");
    expect(html).toContain('value="bilanciato" selected=""');
    expect(html).toContain("aria-describedby=");
    expect(html).toContain("<legend");
    expect(html).not.toContain('type="checkbox" checked=""');
  }
});

test("il bypass è separato dalla scelta salvata e ignora guasti obsoleti", () => {
  const render = (sessionId: string) =>
    renderToStaticMarkup(
      <SettingsProvider
        initial={{
          ...DEFAULT_SETTINGS,
          audioMicrofono: { pulizia: true, sensibilita: "spento" },
          recordingSource: "both",
        }}
        loadError={null}
      >
        <Tooltip.Provider>
          <RecordingPanel
            cleaningFailures={[
              {
                error: { code: "audioCleaningMissing" },
                ingresso: "microfono",
                sessionId,
              },
            ]}
            onError={ignore}
            onPausedChange={ignore}
            paused
            sessionId="current"
          />
        </Tooltip.Provider>
      </SettingsProvider>
    );
  expect(render("old")).not.toContain('role="status"');
  const html = render("current");
  expect(html.match(/role="status"/g)).toHaveLength(1);
  expect(html).toContain('checked=""');
  expect(html).toContain('value="spento" selected=""');
  expect(html).toContain("La sensibilità mantiene il valore scelto");
});
