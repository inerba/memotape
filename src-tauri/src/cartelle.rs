//! Le cartelle dei dati dell'app: impostazioni, indice della Libreria e Cartella della Libreria
//! predefinita. Nelle build di debug `MEMOTAPE_DATA_DIR` le sposta in una cartella dati di prova,
//! così un agente può registrare, trascrivere e cestinare senza toccare i dati veri. I modelli
//! restano in `app_data_dir/models`: non ne fanno parte.

use std::path::{Path, PathBuf};

/// La variabile d'ambiente della cartella dati di prova, letta solo in debug.
pub const VARIABILE: &str = "MEMOTAPE_DATA_DIR";

/// Dove l'app legge e scrive i suoi dati. In `tauri::State` e nel server MCP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cartelle {
    /// `settings.json` (e accanto il suo `.json.tmp`).
    pub settings: PathBuf,
    /// La cartella degli indici della Libreria.
    pub indice: PathBuf,
    /// La Cartella della Libreria se le impostazioni non ne indicano un'altra; `None` se manca la
    /// cartella Documenti.
    pub libreria: Option<PathBuf>,
    /// La cartella dati di prova, se attiva.
    pub prova: Option<PathBuf>,
}

impl Cartelle {
    /// Le cartelle dell'app da quelle di Windows (`app_data_dir`, `app_local_data_dir`, Documenti)
    /// o, se c'è, dalla cartella dati di prova.
    pub fn risolvi(
        prova: Option<PathBuf>,
        app_data: &Path,
        app_local_data: &Path,
        documenti: Option<&Path>,
    ) -> Self {
        match prova {
            Some(dir) => Self {
                settings: dir.join("settings.json"),
                indice: dir.join("indice"),
                libreria: Some(dir.join("Libreria")),
                prova: Some(dir),
            },
            None => Self {
                settings: app_data.join("settings.json"),
                indice: app_local_data.join("libreria"),
                libreria: documenti.map(|d| d.join(crate::library::DEFAULT_FOLDER)),
                prova: None,
            },
        }
    }

    /// Crea le cartelle della cartella dati di prova; senza, non fa nulla.
    pub fn crea_prova(&self) -> std::io::Result<()> {
        if self.prova.is_some() {
            for dir in [&self.indice].into_iter().chain(&self.libreria) {
                std::fs::create_dir_all(dir)?;
            }
        }
        Ok(())
    }
}

/// La cartella dati di prova di `MEMOTAPE_DATA_DIR`, resa assoluta. Sempre `None` in release.
pub fn prova() -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let dir = std::env::var_os(VARIABILE).filter(|v| !v.is_empty())?;
    std::path::absolute(dir).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn senza_cartella_di_prova_valgono_le_cartelle_dell_app() {
        let cartelle = Cartelle::risolvi(
            None,
            Path::new(r"C:\Roaming\it.memotape.desktop"),
            Path::new(r"C:\Local\it.memotape.desktop"),
            Some(Path::new(r"C:\Documenti")),
        );
        assert_eq!(
            cartelle,
            Cartelle {
                settings: r"C:\Roaming\it.memotape.desktop\settings.json".into(),
                indice: r"C:\Local\it.memotape.desktop\libreria".into(),
                libreria: Some(r"C:\Documenti\Memotape".into()),
                prova: None,
            }
        );
    }

    #[test]
    fn la_cartella_di_prova_tiene_impostazioni_indice_e_libreria() {
        let cartelle = Cartelle::risolvi(
            Some(r"D:\prova".into()),
            Path::new(r"C:\Roaming\it.memotape.desktop"),
            Path::new(r"C:\Local\it.memotape.desktop"),
            None,
        );
        assert_eq!(
            cartelle,
            Cartelle {
                settings: r"D:\prova\settings.json".into(),
                // Non `libreria`: su Windows sarebbe la stessa cartella della Libreria, e l'indice
                // ne diventerebbe una Raccolta.
                indice: r"D:\prova\indice".into(),
                libreria: Some(r"D:\prova\Libreria".into()),
                prova: Some(r"D:\prova".into()),
            }
        );
    }
}
