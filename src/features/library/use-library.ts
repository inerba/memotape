import { useEffect, useState } from "react";
import { type AppError, commands, events, type LibraryList } from "@/bindings";

const EMPTY: LibraryList = { raccolte: [], tapes: [] };

/** L'elenco della Libreria, riletto a ogni `library-changed`. */
export function useLibrary(onError: (error: AppError) => void): LibraryList {
  const [list, setList] = useState(EMPTY);
  useEffect(() => {
    const load = async () => {
      const result = await commands.libraryList();
      if (result.status === "ok") {
        setList(result.data);
      } else {
        onError(result.error);
      }
    };
    const changed = events.libraryChanged.listen(load);
    load();
    return () => {
      changed.then((stop) => stop());
    };
  }, [onError]);
  return list;
}
