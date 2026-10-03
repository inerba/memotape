# Sbobino

App desktop per Windows x64 che trasforma in testo, interamente in locale, il parlato di file audio, video e registrazioni fatte dal computer. Tauri 2 + React.

## Avvio rapido

Servono Rust stable MSVC, Visual Studio 2022 con il workload C++, Bun 1.4 e Node 22.22 o successivo. Gli altri prerequisiti della pipeline audio (CMake, Vulkan SDK, ONNX Runtime) sono in [`AGENTS.md`](AGENTS.md#prerequisiti-di-build).

```sh
bun install
bun tauri dev
```

## Controlli

```sh
bun run typecheck
bun run test
bun run check
bun run format:backend
bun run lint:backend
cd src-tauri && cargo test
```

## Dove trovare il resto

- [`PRODUCT.md`](PRODUCT.md): requisiti di prodotto.
- [`AGENTS.md`](AGENTS.md): comandi, prerequisiti, architettura e insidie.
- [`CONTEXT.md`](CONTEXT.md): glossario del dominio.
- [`docs/adr/`](docs/adr/): decisioni architetturali.

Lo scaffold nasce dal template [create-tauri-react](https://github.com/MrLightful/create-tauri-react) (MIT).
