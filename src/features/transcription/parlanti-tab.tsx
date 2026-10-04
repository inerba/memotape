import { Play } from "lucide-react";
import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { keepFocus } from "@/features/player/player";
import { elapsedText } from "@/features/recording/recording";
import {
  ParlanteNameInput,
  VoiceDot,
} from "@/features/transcription/parlante-name";
import {
  type Conversation,
  type Parlante,
  parlanteStats,
  turnsOf,
  voiceColors,
} from "@/features/transcription/phrases";

/**
 * La scheda Parlanti: ogni Parlante con il suo colore, quanto parla, in quanti turni e da quando;
 * un clic sul nome lo rinomina sul posto, ▶ ascolta il suo primo intervento.
 */
export function ParlantiTab({
  conversation,
  list,
  onPlay,
  onRename,
  onRenaming,
  renaming,
}: {
  conversation: Conversation;
  list: Parlante[];
  onPlay?: (ms: number) => void;
  onRename: (voce: Parlante, nome: string) => void;
  onRenaming: (voce: Parlante | null) => void;
  renaming: Parlante | null;
}) {
  const { t } = useTranslation();
  if (list.length === 0) {
    return (
      <div className="flex flex-col gap-1.5 py-10">
        <p className="font-medium">{t("parlantiTab.empty")}</p>
        <p className="max-w-[34rem] text-muted-foreground text-sm leading-relaxed">
          {t("parlantiTab.emptyDescription")}
        </p>
      </div>
    );
  }
  const turns = turnsOf(conversation, t);
  const colors = voiceColors(turns);
  return (
    <ul className="flex flex-col divide-y">
      {list.map((voce) => (
        <ParlanteRow
          color={colors.get(voce.label)}
          editing={
            renaming?.ingresso === voce.ingresso &&
            renaming.parlante === voce.parlante
          }
          key={`${voce.ingresso}:${voce.parlante}`}
          list={list}
          onPlay={onPlay}
          onRename={onRename}
          onRenaming={onRenaming}
          stats={parlanteStats(conversation, turns, voce)}
          voce={voce}
        />
      ))}
    </ul>
  );
}

function ParlanteRow({
  color,
  editing,
  list,
  onPlay,
  onRename,
  onRenaming,
  stats,
  voce,
}: {
  color: number | undefined;
  editing: boolean;
  list: Parlante[];
  onPlay?: (ms: number) => void;
  onRename: (voce: Parlante, nome: string) => void;
  onRenaming: (voce: Parlante | null) => void;
  stats: ReturnType<typeof parlanteStats>;
  voce: Parlante;
}) {
  const { t } = useTranslation();
  const start = useCallback(() => onRenaming(voce), [onRenaming, voce]);
  const cancel = useCallback(() => onRenaming(null), [onRenaming]);
  const play = useCallback(
    () => onPlay?.(stats.firstMs),
    [onPlay, stats.firstMs]
  );
  // Con gli Ingressi separati l'etichetta dice anche l'Ingresso: sta sotto il nome.
  const ingresso =
    voce.label === voce.nome ? null : voce.label.replace(` · ${voce.nome}`, "");
  return (
    <li className="flex items-center gap-4 py-3.5">
      <VoiceDot color={color} />
      <div className="flex min-w-0 flex-1 flex-col">
        {editing ? (
          <ParlanteNameInput
            list={list}
            onCancel={cancel}
            onRename={onRename}
            voce={voce}
          />
        ) : (
          <button
            className="self-start truncate rounded-sm font-medium decoration-muted-foreground/50 underline-offset-4 hover:underline focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40"
            onClick={start}
            title={t("transcription.rename")}
            type="button"
          >
            {voce.nome}
          </button>
        )}
        {ingresso ? (
          <span className="text-muted-foreground text-xs">{ingresso}</span>
        ) : null}
      </div>
      <span className="w-28 text-right text-muted-foreground text-sm tabular-nums">
        {t("parlantiTab.talk", { time: elapsedText(stats.talkMs) })}
      </span>
      <span className="w-20 text-right text-muted-foreground text-sm tabular-nums">
        {t("parlantiTab.turns", { count: stats.turns })}
      </span>
      {onPlay ? (
        <button
          className="flex w-20 items-center justify-end gap-1.5 rounded-md text-sm tabular-nums transition-colors hover:text-play focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 [&_svg]:size-3 [&_svg]:fill-current"
          onClick={play}
          onPointerDown={keepFocus}
          title={t("parlantiTab.first")}
          type="button"
        >
          <Play />
          {elapsedText(stats.firstMs)}
        </button>
      ) : (
        <span className="w-20 text-right text-sm tabular-nums">
          {elapsedText(stats.firstMs)}
        </span>
      )}
    </li>
  );
}
