import { useEffect, useState } from "react";
import { commands, events, type ModelInfo } from "@/bindings";
import { withDownloadProgress } from "@/features/models/models";

/** I modelli del catalogo con il loro stato, aggiornati dagli eventi del backend. */
export function useModels(): ModelInfo[] {
  const [models, setModels] = useState<ModelInfo[]>([]);
  useEffect(() => {
    // Ogni cambio di stato rilegge la lista: un evento arrivato prima della prima lettura non
    // può restare indietro. L'avanzamento, frequente, si applica alla riga.
    const refresh = () => commands.listModels().then(setModels);
    const changed = events.modelStateChanged.listen(refresh);
    const progress = events.modelDownloadProgress.listen(({ payload }) => {
      setModels((current) => withDownloadProgress(current, payload));
    });
    Promise.all([changed, progress]).then(refresh);
    return () => {
      changed.then((stop) => stop());
      progress.then((stop) => stop());
    };
  }, []);
  return models;
}
