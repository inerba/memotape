# Risultati backend osservati

Nessun codice Rust modificato dal 07. Target isolato e override come nel README.

- `cargo test --locked --manifest-path src-tauri/Cargo.toml`, fuori sandbox,
  exit 0: 325 test individuati; 296 passed, 0 failed, 29 ignored,
  0 measured, 0 filtered out, 7.41 s. Main e doc-test: 0 test, exit 0.
  `tests::i_bindings_committati_sono_aggiornati`: ok.
- `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`,
  fuori sandbox, exit 0: finished dev profile in 6.58 s.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, fuori
  sandbox, exit 0, nessun output. Prima esecuzione in sandbox: errore di
  canonicalizzazione/accesso negato, risolto dalla verifica fuori sandbox.

Questo riepilogo trascrive risultati dei tool, non è il log raw della suite.
Non sono stati eseguiti smoke nativi ignored per il ticket 07.
