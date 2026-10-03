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
  /** Quale parte di Sbobino è: la chiave in `about.roles`. */
  role:
    | "decoder"
    | "diarization"
    | "engine"
    | "model"
    | "onnx"
    | "vad"
    | "vadRs";
  url: string;
}

/** Nome e licenza di un modello vengono dal catalogo. */
function model(
  id: string,
  author: string,
  file: string,
  url: string,
  role: Credit["role"] = "model",
  quantization = "Q5_K_M"
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
    quantization,
    role,
    url,
  };
}

// Parakeet per primo: CC BY 4.0 chiede l'attribuzione.
export const CREDITS: Credit[] = [
  model(
    "parakeet-tdt-0.6b-v3-q5km",
    "NVIDIA",
    "parakeet-tdt-0.6b-v3.txt",
    "https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3"
  ),
  model(
    "nemotron-3.5-streaming-0.6b-q5km",
    "NVIDIA",
    "nemotron-3.5-asr-streaming-0.6b.txt",
    "https://huggingface.co/nvidia/nemotron-3.5-asr-streaming-0.6b"
  ),
  model(
    "whisper-large-v3-turbo-q5km",
    "OpenAI",
    "whisper-large-v3-turbo.txt",
    "https://github.com/openai/whisper"
  ),
  model(
    "sortformer-4spk-v2.1-q8",
    "NVIDIA",
    "diar_streaming_sortformer_4spk-v2.1.txt",
    "https://huggingface.co/nvidia/diar_streaming_sortformer_4spk-v2.1",
    "diarization",
    "Q8_0"
  ),
  {
    author: "Silero Team",
    file: "silero-vad.txt",
    license: "MIT",
    name: "Silero VAD v4",
    role: "vad",
    url: "https://github.com/snakers4/silero-vad",
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
    author: "Microsoft",
    file: "onnxruntime.txt",
    license: "MIT",
    name: "ONNX Runtime",
    notices: "onnxruntime-third-party-notices.txt",
    role: "onnx",
    url: "https://github.com/microsoft/onnxruntime",
  },
  {
    file: "vad-rs.txt",
    license: "MIT",
    name: "vad-rs",
    role: "vadRs",
    url: "https://github.com/cjpais/vad-rs",
  },
];
