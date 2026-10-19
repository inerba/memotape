import catalog from "../../../src-tauri/src/managers/models.json";

/** Un componente di terze parti con la sua licenza, nell'ordine di Informazioni. */
export interface Credit {
  /** Chi l'ha creato, per l'attribuzione. */
  author?: string;
  /** Il file del testo della licenza in `src-tauri/resources/licenses`. */
  file: string;
  license: string;
  name: string;
  /** Avvisi di terze parti, in un file a parte. */
  notices?: string;
  /** I modelli si scaricano nella versione GGUF di handy-computer, con questa quantizzazione. */
  quantization?: string;
  /** Quale parte di Memotape è: la chiave in `about.roles`. */
  role:
    | "decoder"
    | "cleaning"
    | "diarization"
    | "engine"
    | "font"
    | "model"
    | "vad";
  url: string;
}

/** La quantizzazione nel nome del file GGUF: `…-Q5_K_M.gguf`. */
const QUANTIZATION = /-([A-Z0-9_]+)\.gguf$/;

/** Nome, licenza e quantizzazione di un modello vengono dal catalogo. */
function model(
  id: string,
  author: string,
  file: string,
  url: string,
  role: Credit["role"] = "model"
): Credit {
  const entry = catalog.modelli.find((m) => m.id === id);
  if (!entry) {
    throw new Error(`modello ${id} non nel catalogo`);
  }
  return {
    author,
    file,
    license: entry.licenza,
    name: entry.nome,
    quantization: QUANTIZATION.exec(entry.url)?.[1],
    role,
    url,
  };
}

/** I modelli di una famiglia con lo stesso testo di licenza. */
function models(
  ids: string[],
  author: string,
  file: string,
  url: string
): Credit[] {
  return ids.map((id) => model(id, author, file, url));
}

// Prima i modelli CC BY 4.0, che chiedono l'attribuzione.
export const CREDITS: Credit[] = [
  model(
    "parakeet-tdt-0.6b-v3-q5km",
    "NVIDIA",
    "parakeet-tdt-0.6b-v3.txt",
    "https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3"
  ),
  model(
    "parakeet-primeline-q8",
    "primeLine",
    "parakeet-primeline.txt",
    "https://huggingface.co/primeline/parakeet-primeline"
  ),
  model(
    "canary-1b-v2-q5km",
    "NVIDIA",
    "canary-1b-v2.txt",
    "https://huggingface.co/nvidia/canary-1b-v2"
  ),
  model(
    "nemotron-3.5-streaming-0.6b-q5km",
    "NVIDIA",
    "nemotron-3.5-asr-streaming-0.6b.txt",
    "https://huggingface.co/nvidia/nemotron-3.5-asr-streaming-0.6b"
  ),
  model(
    "cohere-transcribe-03-2026-q5km",
    "Cohere Labs",
    "cohere-transcribe-03-2026.txt",
    "https://huggingface.co/CohereLabs/cohere-transcribe-03-2026"
  ),
  ...models(
    ["qwen3-asr-0.6b-q8", "qwen3-asr-1.7b-q5km"],
    "Qwen Team, Alibaba Cloud",
    "qwen3-asr.txt",
    "https://huggingface.co/Qwen"
  ),
  ...models(
    [
      "voxtral-mini-3b-2507-q5km",
      "voxtral-mini-4b-realtime-2602-q5km",
      "voxtral-small-24b-2507-q5km",
    ],
    "Mistral AI",
    "voxtral.txt",
    "https://huggingface.co/mistralai"
  ),
  model(
    "whisper-large-v3-turbo-q5km",
    "OpenAI",
    "whisper-large-v3-turbo.txt",
    "https://github.com/openai/whisper"
  ),
  ...models(
    [
      "whisper-tiny-q8",
      "whisper-base-q8",
      "whisper-small-q8",
      "whisper-medium-q8",
      "whisper-large-q5km",
      "whisper-large-v2-q5km",
      "whisper-large-v3-q5km",
    ],
    "OpenAI",
    "whisper.txt",
    "https://github.com/openai/whisper"
  ),
  model(
    "sortformer-4spk-v2.1-q8",
    "NVIDIA",
    "diar_streaming_sortformer_4spk-v2.1.txt",
    "https://huggingface.co/nvidia/diar_streaming_sortformer_4spk-v2.1",
    "diarization"
  ),
  {
    author: "Jean-Marc Valin, Xiph.Org",
    file: "rnnoise.txt",
    license: "BSD-3-Clause",
    name: "RNNoise 0.2",
    role: "cleaning",
    url: "https://gitlab.xiph.org/xiph/rnnoise/-/tree/v0.2",
  },
  {
    author: "Silero Team",
    file: "silero-vad.txt",
    license: "MIT",
    name: "Silero VAD v6.2",
    role: "vad",
    url: "https://huggingface.co/handy-computer/silero-vad-v6.2-gguf",
  },
  {
    author: "Philip Deljanov",
    file: "symphonia.txt",
    license: "MPL-2.0",
    name: "Symphonia",
    role: "decoder",
    url: "https://github.com/pdeljanov/Symphonia",
  },
  {
    author: "The transcribe.cpp authors",
    file: "transcribe-cpp.txt",
    license: "MIT",
    name: "transcribe.cpp",
    role: "engine",
    url: "https://github.com/handy-computer/transcribe.cpp",
  },
  {
    author: "Rasmus Andersson",
    file: "inter.txt",
    license: "OFL-1.1",
    name: "Inter",
    role: "font",
    url: "https://github.com/rsms/inter",
  },
  {
    author: "Kostas Bartsokas",
    file: "commissioner.txt",
    license: "OFL-1.1",
    name: "Commissioner",
    role: "font",
    url: "https://github.com/kosbarts/Commissioner",
  },
];
