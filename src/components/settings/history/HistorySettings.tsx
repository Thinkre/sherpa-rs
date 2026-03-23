import React, { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { AudioPlayer } from "../../ui/AudioPlayer";
import { Button } from "../../ui/Button";
import { Copy, Star, Check, Trash2, FolderOpen } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { commands, type HistoryEntry, type ModelInfo } from "@/bindings";
import { formatDateTime } from "@/utils/dateFormat";
import { getTranslatedModelName } from "@/lib/utils/modelTranslation";

interface OpenRecordingsButtonProps {
  onClick: () => void;
  label: string;
}

const OpenRecordingsButton: React.FC<OpenRecordingsButtonProps> = ({
  onClick,
  label,
}) => (
  <Button
    onClick={onClick}
    variant="secondary"
    size="sm"
    className="flex items-center gap-2"
    title={label}
  >
    <FolderOpen className="w-4 h-4" />
    <span>{label}</span>
  </Button>
);

export const HistorySettings: React.FC = () => {
  const { t } = useTranslation();
  const [historyEntries, setHistoryEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [models, setModels] = useState<ModelInfo[]>([]);

  const loadModels = useCallback(async () => {
    try {
      const result = await commands.getAvailableModels();
      if (result.status === "ok") {
        setModels(result.data);
      }
    } catch (error) {
      console.error("Failed to load models:", error);
    }
  }, []);

  const loadHistoryEntries = useCallback(async () => {
    try {
      const result = await commands.getHistoryEntries();
      if (result.status === "ok") {
        setHistoryEntries(result.data);
      }
    } catch (error) {
      console.error("Failed to load history entries:", error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadModels();
    loadHistoryEntries();

    // Listen for history update events
    const setupListener = async () => {
      const unlisten = await listen("history-updated", () => {
        console.log("History updated, reloading entries...");
        loadHistoryEntries();
      });

      // Return cleanup function
      return unlisten;
    };

    let unlistenPromise = setupListener();

    return () => {
      unlistenPromise.then((unlisten) => {
        if (unlisten) {
          unlisten();
        }
      });
    };
  }, [loadHistoryEntries, loadModels]);

  const toggleSaved = async (id: number) => {
    try {
      await commands.toggleHistoryEntrySaved(id);
      // No need to reload here - the event listener will handle it
    } catch (error) {
      console.error("Failed to toggle saved status:", error);
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch (error) {
      console.error("Failed to copy to clipboard:", error);
    }
  };

  const getAudioUrl = async (fileName: string) => {
    try {
      const result = await commands.getAudioFilePath(fileName);
      if (result.status === "ok") {
        return convertFileSrc(`${result.data}`, "asset");
      }
      return null;
    } catch (error) {
      console.error("Failed to get audio file path:", error);
      return null;
    }
  };

  const deleteAudioEntry = async (id: number) => {
    try {
      await commands.deleteHistoryEntry(id);
    } catch (error) {
      console.error("Failed to delete audio entry:", error);
      throw error;
    }
  };

  const openRecordingsFolder = async () => {
    try {
      await commands.openRecordingsFolder();
    } catch (error) {
      console.error("Failed to open recordings folder:", error);
    }
  };

  if (loading) {
    return (
      <div className="max-w-3xl w-full mx-auto space-y-6">
        <div className="space-y-2">
          <div className="px-4 flex items-center justify-between">
            <div>
              <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
                {t("settings.history.title")}
              </h2>
            </div>
            <OpenRecordingsButton
              onClick={openRecordingsFolder}
              label={t("settings.history.openFolder")}
            />
          </div>
          <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
            <div className="px-4 py-3 text-center text-text/60">
              {t("settings.history.loading")}
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (historyEntries.length === 0) {
    return (
      <div className="max-w-3xl w-full mx-auto space-y-6">
        <div className="space-y-2">
          <div className="px-4 flex items-center justify-between">
            <div>
              <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
                {t("settings.history.title")}
              </h2>
            </div>
            <OpenRecordingsButton
              onClick={openRecordingsFolder}
              label={t("settings.history.openFolder")}
            />
          </div>
          <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
            <div className="px-4 py-3 text-center text-text/60">
              {t("settings.history.empty")}
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <div className="space-y-2">
        <div className="px-4 flex items-center justify-between">
          <div>
            <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
              {t("settings.history.title")}
            </h2>
          </div>
          <OpenRecordingsButton
            onClick={openRecordingsFolder}
            label={t("settings.history.openFolder")}
          />
        </div>
        <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
          <div className="divide-y divide-mid-gray/20">
            {historyEntries.map((entry) => (
              <HistoryEntryComponent
                key={entry.id}
                entry={entry}
                models={models}
                onToggleSaved={() => toggleSaved(entry.id)}
                onCopyText={() => copyToClipboard(entry.transcription_text)}
                getAudioUrl={getAudioUrl}
                deleteAudio={deleteAudioEntry}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

interface HistoryEntryProps {
  entry: HistoryEntry;
  models: ModelInfo[];
  onToggleSaved: () => void;
  onCopyText: () => void;
  getAudioUrl: (fileName: string) => Promise<string | null>;
  deleteAudio: (id: number) => Promise<void>;
}

const HistoryEntryComponent: React.FC<HistoryEntryProps> = ({
  entry,
  models,
  onToggleSaved,
  onCopyText,
  getAudioUrl,
  deleteAudio,
}) => {
  const { t, i18n } = useTranslation();
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [showCopied, setShowCopied] = useState(false);

  useEffect(() => {
    const loadAudio = async () => {
      const url = await getAudioUrl(entry.file_name);
      setAudioUrl(url);
    };
    loadAudio();
  }, [entry.file_name, getAudioUrl]);

  const handleCopyText = () => {
    onCopyText();
    setShowCopied(true);
    setTimeout(() => setShowCopied(false), 2000);
  };

  const handleDeleteEntry = async () => {
    try {
      await deleteAudio(entry.id);
    } catch (error) {
      console.error("Failed to delete entry:", error);
      alert("Failed to delete entry. Please try again.");
    }
  };

  const formattedDate = formatDateTime(String(entry.timestamp), i18n.language);

  // Find the model used for this transcription
  // For API models, model_id contains the actual model name (e.g., "qwen3-asr-flash")
  // For local models, model_id is the model ID that can be found in models list
  const model = models.find((m) => m.id === entry.model_id);
  
  // Check if this is an API model (model_id exists but not found in local models)
  const isApiModel = entry.model_id && !model;

  // Format display text according to requirements with color coding
  const formatDisplayText = () => {
    const parts: JSX.Element[] = [];

    // Show voice command info if available (command + selected text + result) - use color scheme from image
    if (entry.is_voice_command && entry.command_text && entry.selected_text) {
      parts.push(
        <div key="command" className="text-sm">
          <span className="text-purple-500 dark:text-purple-400 font-medium">指令: </span>
          <span className="text-gray-900 dark:text-gray-100">{entry.command_text}</span>
        </div>
      );
      parts.push(
        <div key="selected" className="text-sm">
          <span className="text-blue-500 dark:text-blue-400 font-medium">选中文本: </span>
          <span className="text-gray-700 dark:text-gray-300">{entry.selected_text}</span>
        </div>
      );
      if (entry.post_processed_text) {
        parts.push(
          <div key="result" className="text-sm">
            <span className="text-green-500 dark:text-green-400 font-medium">结果: </span>
            <span className="text-gray-900 dark:text-gray-100">{entry.post_processed_text}</span>
          </div>
        );
      }
      return parts;
    }

    // For regular transcriptions, show transcription result
    parts.push(
      <div key="transcription" className="text-sm text-gray-900 dark:text-gray-100">
        {entry.transcription_text}
      </div>
    );

    // Show ITN result if available and different from transcription
    if (entry.itn_text && entry.itn_text !== entry.transcription_text) {
      parts.push(
        <div key="itn" className="text-sm">
          <span className="text-text/60 dark:text-text/40">逆文本: </span>
          <span className="text-gray-900 dark:text-gray-100">{entry.itn_text}</span>
        </div>
      );
    }

    // Show LLM result only if LLM role was matched (but not voice command)
    if (entry.post_processed_text && entry.llm_role_name) {
      parts.push(
        <div key="llm" className="text-sm">
          <span className="text-purple-500 dark:text-purple-400 font-medium">LLM {entry.llm_role_name}: </span>
          <span className="text-gray-900 dark:text-gray-100">{entry.post_processed_text}</span>
        </div>
      );
    }

    return parts;
  };

  return (
    <div className="px-4 py-2 pb-5 flex flex-col gap-3">
      <div className="flex justify-between items-center">
        <div className="flex flex-col">
          <div className="flex items-center gap-2 flex-wrap">
            <p className="text-sm font-medium">{formattedDate}</p>
            {model && (
              <>
                <span className="text-xs text-text/40">·</span>
                <span className="text-xs text-text/60">
                  {getTranslatedModelName(model, t)}
                </span>
                <span className="text-xs text-text/40">
                  {" "}(本地)
                </span>
              </>
            )}
            {isApiModel && (
              <>
                <span className="text-xs text-text/40">·</span>
                <span className="text-xs text-text/60">
                  {entry.model_id}
                </span>
                <span className="text-xs text-blue-500 dark:text-blue-400">
                  {" "}(API)
                </span>
              </>
            )}
            {entry.llm_model && (
              <>
                <span className="text-xs text-text/40">·</span>
                <span className="text-xs text-purple-500">
                  LLM: {entry.llm_model}
                </span>
              </>
            )}
            {entry.llm_role_name && (
              <>
                <span className="text-xs text-text/40">·</span>
                <span className="text-xs text-purple-500">
                  {entry.llm_role_name}
                </span>
              </>
            )}
          </div>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={handleCopyText}
            className="text-text/50 hover:text-logo-primary  hover:border-logo-primary transition-colors cursor-pointer"
            title={t("settings.history.copyToClipboard")}
          >
            {showCopied ? (
              <Check width={16} height={16} />
            ) : (
              <Copy width={16} height={16} />
            )}
          </button>
          <button
            onClick={onToggleSaved}
            className={`p-2 rounded  transition-colors cursor-pointer ${
              entry.saved
                ? "text-logo-primary hover:text-logo-primary/80"
                : "text-text/50 hover:text-logo-primary"
            }`}
            title={
              entry.saved
                ? t("settings.history.unsave")
                : t("settings.history.save")
            }
          >
            <Star
              width={16}
              height={16}
              fill={entry.saved ? "currentColor" : "none"}
            />
          </button>
          <button
            onClick={handleDeleteEntry}
            className="text-text/50 hover:text-logo-primary transition-colors cursor-pointer"
            title={t("settings.history.delete")}
          >
            <Trash2 width={16} height={16} />
          </button>
        </div>
      </div>
      <div className="text-sm pb-2 select-text cursor-text space-y-1">
        {formatDisplayText()}
      </div>
      {audioUrl && <AudioPlayer src={audioUrl} className="w-full" />}
    </div>
  );
};
