import { expect, test } from "bun:test";
import { Tooltip } from "radix-ui";
import { renderToStaticMarkup } from "react-dom/server";
import "@/lib/i18n";
import { DEFAULT_SETTINGS } from "@/features/settings/settings";
import { SettingsProvider } from "@/features/settings/settings-context";
import { TapeMenu } from "./tape-actions";

const noAction = () => undefined;

function render(diarized: boolean, disabled: boolean) {
  return renderToStaticMarkup(
    <SettingsProvider initial={DEFAULT_SETTINGS} loadError={null}>
      <Tooltip.Provider>
        <TapeMenu
          diarized={diarized}
          diarizingDisabled={disabled}
          disabled={disabled}
          library={{ raccolte: [], tapes: [] }}
          onDiarize={noAction}
          onError={noAction}
          onExported={noAction}
          onMove={noAction}
          onReveal={noAction}
          onTranscribe={disabled ? undefined : noAction}
          onTrash={noAction}
          path="D:\Call.tape"
        />
      </Tooltip.Provider>
    </SettingsProvider>
  );
}

test("Diarizza è accanto a Trascrivi di nuovo ed è un pulsante accessibile", () => {
  const html = render(false, false);
  expect(html.indexOf("Diarizza")).toBeGreaterThan(
    html.indexOf("Trascrivi di nuovo")
  );
  const button = [...html.matchAll(/<button\b[^>]*>[\s\S]*?<\/button>/g)].find(
    ([markup]) => markup.includes("Diarizza")
  )?.[0];
  expect(button).toContain('type="button"');
  expect(button).not.toContain('disabled=""');
  expect(button).toContain('aria-hidden="true"');
});

test("Diarizza di nuovo resta visibile e disabilitato durante un'Attività o senza testo", () => {
  const html = render(true, true);
  const button = [...html.matchAll(/<button\b[^>]*>[\s\S]*?<\/button>/g)].find(
    ([markup]) => markup.includes("Diarizza di nuovo")
  )?.[0];
  expect(button).toContain('disabled=""');
});
