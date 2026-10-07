//! La finestra parte con il tema salvato e il preload, prima del primo render React.

use crate::managers::settings::{Language, SettingsStore};
use serde_json::Value;
use tauri::{Manager, Theme, WebviewWindowBuilder, window::Color};

/// Carta e inchiostro di `global.css`, anche prima della prima pittura della WebView.
fn background(theme: Theme) -> Color {
    match theme {
        Theme::Dark => Color(25, 22, 19, 255),
        _ => Color(252, 250, 246, 255),
    }
}

/// Le copie iniziali vengono dalle stesse traduzioni dell'app, senza attendere il bundle React.
fn payload(language: Language) -> serde_json::Result<Value> {
    let source = match language {
        Language::It => include_str!("../../src/locales/it.json"),
        Language::En => include_str!("../../src/locales/en.json"),
        Language::Fr => include_str!("../../src/locales/fr.json"),
        Language::Es => include_str!("../../src/locales/es.json"),
        Language::De => include_str!("../../src/locales/de.json"),
        Language::Pl => include_str!("../../src/locales/pl.json"),
    };
    let texts: Value = serde_json::from_str(source)?;
    Ok(serde_json::json!({
        "language": language.code(),
        "startup": texts["startup"],
        "window": texts["window"],
        "error": texts["errors"]["app"],
    }))
}

pub fn create_main(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let settings = app.state::<SettingsStore>().get();
    let language = settings.interface_language.unwrap_or_else(Language::system);
    let mut data = payload(language)?;
    data["theme"] = match settings.tema.theme() {
        Some(Theme::Dark) => Value::from("dark"),
        Some(Theme::Light) => Value::from("light"),
        _ => Value::Null,
    };
    let script = format!("window.__MEMOTAPE_STARTUP__ = {data};");
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .expect("la configurazione contiene la finestra main");
    let window = WebviewWindowBuilder::from_config(app, config)?
        .visible(false)
        .initialization_script(script)
        .background_color(background(settings.tema.theme().unwrap_or(Theme::Light)))
        .build()?;
    // Il tema nativo viene applicato prima di mostrare la finestra. Il documento conserva
    // anche la scelta esplicita: non dipende dalla preferenza segnalata dalla WebView2.
    settings.tema.apply(app.handle());
    // Con Sistema la finestra appena creata conosce il tema effettivo di Windows. Si allineano
    // entrambi i fondi prima di mostrarla. Il documento usa il tema salvato esplicitamente,
    // perché le media query di WebView2 possono continuare a seguire Windows.
    window.set_background_color(Some(background(window.theme()?)))?;
    window.show()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn il_preload_ha_le_copie_dell_app_in_tutte_le_lingue() {
        for language in [
            Language::It,
            Language::En,
            Language::Fr,
            Language::Es,
            Language::De,
            Language::Pl,
        ] {
            let data = payload(language).unwrap();
            assert_eq!(data["language"], language.code());
            for (section, keys) in [
                ("startup", &["starting", "slow"][..]),
                ("window", &["minimize", "maximize", "restore", "close"][..]),
                ("error", &["description", "action"][..]),
            ] {
                for key in keys {
                    assert!(
                        data[section][key]
                            .as_str()
                            .is_some_and(|text| !text.is_empty()),
                        "{}: {section}.{key} deve essere disponibile prima di React",
                        language.code()
                    );
                }
            }
        }
    }
}
