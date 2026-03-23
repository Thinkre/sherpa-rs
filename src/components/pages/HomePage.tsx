import React, { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { Mic, Copy, Check, FileText, Clock } from "lucide-react";
import { type } from "@tauri-apps/plugin-os";
import { commands, type HistoryEntry } from "@/bindings";
import { useModelStore } from "../../stores/modelStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { formatKeyCombination, type OSType } from "../../lib/utils/keyboard";
import { Textarea } from "../ui/Textarea";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

interface StatsCardProps {
  icon: React.ReactNode;
  title: string;
  value: string;
  subtitle: string;
}

const StatsCard: React.FC<StatsCardProps> = ({
  icon,
  title,
  value,
  subtitle,
}) => {
  return (
    <div
      className="group relative rounded-xl p-4 flex items-center gap-3 overflow-hidden transition-all duration-500 hover:scale-[1.02]"
      style={{
        background: "rgba(255, 255, 255, 0.1)",
        backdropFilter: "blur(30px) saturate(200%)",
        WebkitBackdropFilter: "blur(30px) saturate(200%)",
        border: "1px solid rgba(139, 92, 246, 0.3)",
        boxShadow: "0 8px 32px 0 rgba(139, 92, 246, 0.15)",
      }}
    >
      {/* Animated gradient overlay */}
      <div
        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500"
        style={{
          background:
            "linear-gradient(135deg, rgba(139, 92, 246, 0.2) 0%, rgba(99, 102, 241, 0.15) 50%, rgba(139, 92, 246, 0.1) 100%)",
          backgroundSize: "200% 200%",
          animation: "gradient-shift 3s ease infinite",
        }}
      />

      {/* Glass reflection effect */}
      <div
        className="absolute top-0 left-0 w-full h-1/2 opacity-20 group-hover:opacity-30 transition-opacity duration-500"
        style={{
          background:
            "linear-gradient(180deg, rgba(255, 255, 255, 0.3) 0%, transparent 100%)",
        }}
      />

      <div
        className="relative z-10 text-logo-primary dark:text-logo-primary"
        style={{ color: "#8b5cf6" }}
      >
        {icon}
      </div>
      <div className="relative z-10 flex-1">
        <p className="text-sm text-gray-600 dark:text-gray-400">{title}</p>
        <p className="text-xl font-semibold text-gray-900 dark:text-gray-100">
          {value}
        </p>
        <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
          {subtitle}
        </p>
      </div>
    </div>
  );
};

interface HomePageProps {
  onNavigateToHistory?: () => void;
}

export const HomePage: React.FC<HomePageProps> = ({ onNavigateToHistory }) => {
  const { t } = useTranslation();
  const [historyEntries, setHistoryEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [userName] = useState("用户"); // TODO: Get from settings or user profile
  const { currentModel, getModelInfo } = useModelStore();
  const currentModelInfo = getModelInfo(currentModel);
  const [osType, setOsType] = useState<OSType>("unknown");
  const [textInput, setTextInput] = useState("");
  const textareaRef = React.useRef<HTMLTextAreaElement>(null);
  const lastTranscriptionRef = React.useRef<string>("");
  const [copiedId, setCopiedId] = useState<number | null>(null);

  // Get shortcut binding for transcribe from store (reactive)
  const bindings = useSettingsStore((state) => state.settings?.bindings || {});
  const transcribeBinding = bindings.transcribe?.current_binding || "";

  // Detect OS type
  useEffect(() => {
    const detectOsType = async () => {
      try {
        const detectedType = await type();
        let normalizedType: OSType;

        switch (detectedType) {
          case "macos":
            normalizedType = "macos";
            break;
          case "windows":
            normalizedType = "windows";
            break;
          case "linux":
            normalizedType = "linux";
            break;
          default:
            normalizedType = "unknown";
        }

        setOsType(normalizedType);
      } catch (error) {
        console.error("Error detecting OS type:", error);
        setOsType("unknown");
      }
    };

    detectOsType();
  }, []);

  // Format shortcut for display
  const formattedShortcut = useMemo(() => {
    if (!transcribeBinding) {
      return "";
    }
    return formatKeyCombination(transcribeBinding, osType);
  }, [transcribeBinding, osType]);

  useEffect(() => {
    loadHistory();
    const setupListener = async () => {
      const unlistenHistory = await listen("history-updated", () => {
        loadHistory();
      });

      // Listen for transcription results to display in text box at cursor position
      // NOTE: We should NOT insert text here if the textarea has focus, because the paste
      // operation will already insert the text via Ctrl+V/Cmd+V. We only insert if the
      // textarea does NOT have focus (user is typing elsewhere).
      const unlistenTranscription = await listen<string>(
        "transcription-result",
        (event) => {
          const result = event.payload;

          // Prevent duplicate processing of the same transcription
          if (!result || result === lastTranscriptionRef.current) {
            return;
          }
          lastTranscriptionRef.current = result;

          if (textareaRef.current) {
            const textarea = textareaRef.current;
            // Check if textarea has focus - if it does, the paste operation will handle it
            // and we should NOT insert again to avoid duplicates
            const hasFocus = document.activeElement === textarea;
            
            if (hasFocus) {
              // Textarea has focus - paste operation will handle insertion, skip to avoid duplicate
              console.log("Textarea has focus, skipping insertion to avoid duplicate with paste operation");
              return;
            }

            const start = textarea.selectionStart;
            const end = textarea.selectionEnd;

            // Get current value directly from textarea to avoid stale closure
            const currentValue = textarea.value;

            // Insert text at cursor position
            const newValue =
              currentValue.substring(0, start) +
              result +
              currentValue.substring(end);

            setTextInput(newValue);

            // Update cursor position after insertion
            setTimeout(() => {
              if (textareaRef.current) {
                const newCursorPos = start + result.length;
                textareaRef.current.setSelectionRange(
                  newCursorPos,
                  newCursorPos,
                );
                textareaRef.current.focus();
              }
            }, 0);
          } else {
            // Fallback: if textarea ref is not available, append to end
            setTextInput((prev) => (prev ? prev + result : result));
          }
        },
      );

      return () => {
        unlistenHistory();
        unlistenTranscription();
      };
    };
    let unlistenFn: (() => void) | null = null;
    setupListener().then((fn) => {
      unlistenFn = fn;
    });
    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  const loadHistory = async () => {
    try {
      const result = await commands.getHistoryEntries();
      if (result.status === "ok") {
        setHistoryEntries(result.data);
      }
    } catch (error) {
      console.error("Failed to load history:", error);
    } finally {
      setLoading(false);
    }
  };

  // Calculate statistics
  const stats = useMemo(() => {
    const totalTranscriptions = historyEntries.length;
    const totalWords = historyEntries.reduce((sum, entry) => {
      const text = entry.post_processed_text || entry.transcription_text;
      return sum + (text.match(/\S+/g)?.length || 0);
    }, 0);

    // Estimate time saved (assuming 40 WPM typing speed)
    const timeSavedMinutes = Math.round(totalWords / 40);

    // Calculate total recording time (rough estimate: ~10 seconds per entry)
    const totalMinutes = Math.round((totalTranscriptions * 10) / 60);

    return {
      totalTranscriptions,
      totalWords,
      timeSavedMinutes,
      totalMinutes,
    };
  }, [historyEntries]);

  // Copy text to clipboard
  const handleCopyText = async (entry: HistoryEntry) => {
    // Use same logic as display: post_processed_text > itn_text > transcription_text
    const textToCopy =
      entry.post_processed_text ||
      entry.itn_text ||
      entry.transcription_text;

    if (!textToCopy) {
      console.error("No text to copy for entry:", entry.id);
      return;
    }

    console.log("Copying text:", textToCopy.substring(0, 50) + "...");

    try {
      await writeText(textToCopy);
      console.log("Text copied successfully");
      setCopiedId(entry.id);
      setTimeout(() => setCopiedId(null), 2000);
    } catch (error) {
      console.error("Failed to copy text:", error);
    }
  };

  // Group history by date
  const groupedHistory = useMemo(() => {
    const groups: Record<string, HistoryEntry[]> = {};
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    historyEntries.forEach((entry) => {
      const entryDate = new Date(entry.timestamp * 1000);
      entryDate.setHours(0, 0, 0, 0);

      let key: string;
      if (entryDate.getTime() === today.getTime()) {
        key = "今天";
      } else {
        const yesterday = new Date(today);
        yesterday.setDate(yesterday.getDate() - 1);
        if (entryDate.getTime() === yesterday.getTime()) {
          key = "昨天";
        } else {
          const dateStr = new Date(entry.timestamp * 1000).toLocaleDateString(
            "zh-CN",
            {
              year: "numeric",
              month: "long",
              day: "numeric",
              weekday: "long",
            },
          );
          key = dateStr;
        }
      }

      if (!groups[key]) {
        groups[key] = [];
      }
      groups[key].push(entry);
    });

    return Object.entries(groups).sort((a, b) => {
      if (a[0] === "今天") return -1;
      if (b[0] === "今天") return 1;
      if (a[0] === "昨天") return -1;
      if (b[0] === "昨天") return 1;
      return b[0].localeCompare(a[0]);
    });
  }, [historyEntries]);

  return (
    <div
      className="max-w-4xl w-full mx-auto p-6 space-y-6"
      style={{
        background:
          "linear-gradient(135deg, rgba(139, 92, 246, 0.03) 0%, rgba(99, 102, 241, 0.02) 100%)",
      }}
    >
      {/* Welcome Section */}
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-2">
          {userName}, 欢迎回来
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          选中文本框，按住
          {formattedShortcut ? (
            <span className="font-semibold text-logo-primary dark:text-logo-primary mx-1 px-2 py-0.5 rounded bg-logo-primary/10 border border-logo-primary/30">
              {formattedShortcut}
            </span>
          ) : (
            <span className="font-semibold mx-1">快捷键</span>
          )}
          开始说话，近期你已完成了 {stats.totalTranscriptions} 次转录
        </p>
      </div>

      {/* Statistics Cards */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        <StatsCard
          icon={<Mic className="w-6 h-6" />}
          title="累计语音输入"
          value={`${stats.totalMinutes}分钟`}
          subtitle={`共${stats.totalTranscriptions}次转写`}
        />
        <StatsCard
          icon={<FileText className="w-6 h-6" />}
          title="生成文本"
          value={`${stats.totalWords}词`}
          subtitle={`平均${Math.round(stats.totalWords / Math.max(stats.totalMinutes, 1))}/分`}
        />
        <StatsCard
          icon={<Clock className="w-6 h-6" />}
          title="节省时间"
          value={`${stats.timeSavedMinutes}分钟`}
          subtitle="按40 WPM 估算"
        />
      </div>

      {/* Text Input Section */}
      <div className="space-y-2">
        <Textarea
          ref={textareaRef}
          value={textInput}
          onChange={(e) => setTextInput(e.target.value)}
          placeholder="在这里输入文本..."
          className="w-full min-h-[120px] resize-y"
          style={{
            background: "rgba(255, 255, 255, 0.15)",
            backdropFilter: "blur(25px) saturate(200%)",
            WebkitBackdropFilter: "blur(25px) saturate(200%)",
            border: "1px solid rgba(139, 92, 246, 0.25)",
            boxShadow: "0 4px 16px 0 rgba(139, 92, 246, 0.1)",
          }}
        />
      </div>

      {/* History Section */}
      <div className="space-y-4">
        {loading ? (
          <div className="text-center py-8 text-gray-500">加载中...</div>
        ) : groupedHistory.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            还没有历史记录，开始使用语音输入吧！
          </div>
        ) : (
          groupedHistory.map(([date, entries]) => (
            <div key={date}>
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
                {date}
              </h2>
              <div className="space-y-2">
                {entries.map((entry) => {
                  const time = new Date(
                    entry.timestamp * 1000,
                  ).toLocaleTimeString("zh-CN", {
                    hour: "2-digit",
                    minute: "2-digit",
                  });
                  // Get model info for this specific entry (the model that was actually used)
                  // For API models, model_id contains the actual model name (e.g., "qwen3-asr-flash")
                  // For local models, model_id is the model ID that can be found in models list
                  const entryModelInfo = entry.model_id
                    ? getModelInfo(entry.model_id)
                    : null;
                  
                  // Check if this is an API model (model_id exists but not found in local models)
                  const isApiModel = entry.model_id && !entryModelInfo;

                  // Show final result: LLM processed text > ITN text > original transcription
                  const finalText =
                    entry.post_processed_text ||
                    entry.itn_text ||
                    entry.transcription_text;

                  return (
                    <div
                      key={entry.id}
                      className="group relative rounded-lg p-3 overflow-hidden transition-all duration-500 hover:scale-[1.01] select-text"
                      style={{
                        background: "rgba(255, 255, 255, 0.15)",
                        backdropFilter: "blur(25px) saturate(200%)",
                        WebkitBackdropFilter: "blur(25px) saturate(200%)",
                        border: "1px solid rgba(139, 92, 246, 0.25)",
                        boxShadow: "0 4px 16px 0 rgba(139, 92, 246, 0.1)",
                        userSelect: "text",
                        WebkitUserSelect: "text",
                      }}
                      onMouseDown={(e) => {
                        // Store the initial mouse position
                        const startX = e.clientX;
                        const startY = e.clientY;
                        const startTime = Date.now();
                        
                        // Check if user is selecting text (mouse moved)
                        const handleMouseUp = (upEvent: MouseEvent) => {
                          const endTime = Date.now();
                          const timeDiff = endTime - startTime;
                          const selection = window.getSelection();
                          const hasSelection = selection && selection.toString().length > 0;
                          
                          // Only navigate if:
                          // 1. No text was selected, AND
                          // 2. Mouse didn't move much (click, not drag), AND
                          // 3. Click was quick (not a long press)
                          if (!hasSelection && timeDiff < 300) {
                            const endX = upEvent.clientX;
                            const endY = upEvent.clientY;
                            const moved = Math.abs(endX - startX) > 5 || Math.abs(endY - startY) > 5;
                            
                            if (!moved) {
                              onNavigateToHistory?.();
                            }
                          }
                          
                          document.removeEventListener("mouseup", handleMouseUp);
                        };
                        
                        document.addEventListener("mouseup", handleMouseUp);
                      }}
                    >
                      {/* Glass reflection effect */}
                      <div
                        className="absolute top-0 left-0 w-full h-1/2 opacity-10 group-hover:opacity-20 transition-opacity duration-500"
                        style={{
                          background:
                            "linear-gradient(180deg, rgba(255, 255, 255, 0.4) 0%, transparent 100%)",
                        }}
                      />

                      {/* Hover glow effect */}
                      <div
                        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500"
                        style={{
                          background:
                            "radial-gradient(circle at center, rgba(139, 92, 246, 0.15) 0%, transparent 70%)",
                        }}
                      />
                      <div className="relative z-10 flex items-start justify-between gap-3">
                        <div className="flex-1 select-text" style={{ userSelect: "text", WebkitUserSelect: "text" }}>
                          <div className="flex items-center gap-2 mb-1 flex-wrap">
                            <span className="text-sm text-gray-500 dark:text-gray-400">
                              {time}
                            </span>
                            {entryModelInfo && (
                              <>
                                <span className="text-xs text-gray-400 dark:text-gray-500">
                                  ·
                                </span>
                                <span className="text-xs text-gray-500 dark:text-gray-400">
                                  {entryModelInfo.name}
                                </span>
                                <span className="text-xs text-gray-400 dark:text-gray-500">
                                  {" "}(本地)
                                </span>
                              </>
                            )}
                            {isApiModel && (
                              <>
                                <span className="text-xs text-gray-400 dark:text-gray-500">
                                  ·
                                </span>
                                <span className="text-xs text-gray-500 dark:text-gray-400">
                                  {entry.model_id}
                                </span>
                                <span className="text-xs text-blue-500 dark:text-blue-400">
                                  {" "}(API)
                                </span>
                              </>
                            )}
                            {entry.llm_model && (
                              <>
                                <span className="text-xs text-gray-400 dark:text-gray-500">
                                  ·
                                </span>
                                <span className="text-xs text-purple-500 dark:text-purple-400">
                                  LLM: {entry.llm_model}
                                </span>
                              </>
                            )}
                            {entry.llm_role_name && (
                              <>
                                <span className="text-xs text-gray-400 dark:text-gray-500">
                                  ·
                                </span>
                                <span className="text-xs text-purple-500 dark:text-purple-400">
                                  {entry.llm_role_name}
                                </span>
                              </>
                            )}
                          </div>
                          {/* Display voice command with selected text separately */}
                          {entry.is_voice_command && entry.selected_text && entry.command_text ? (
                            <div className="space-y-2">
                              <div>
                                <span className="text-xs text-purple-500 dark:text-purple-400 font-medium">
                                  指令:
                                </span>
                                <span className="text-gray-900 dark:text-gray-100 text-sm ml-1 select-text" style={{ userSelect: "text", WebkitUserSelect: "text" }}>
                                  {entry.command_text}
                                </span>
                              </div>
                              <div>
                                <span className="text-xs text-blue-500 dark:text-blue-400 font-medium">
                                  选中文本:
                                </span>
                                <span className="text-gray-700 dark:text-gray-300 text-sm ml-1 select-text" style={{ userSelect: "text", WebkitUserSelect: "text" }}>
                                  {entry.selected_text.length > 50
                                    ? `${entry.selected_text.substring(0, 50)}...`
                                    : entry.selected_text}
                                </span>
                              </div>
                              <div>
                                <span className="text-xs text-green-500 dark:text-green-400 font-medium">
                                  结果:
                                </span>
                                <span className="text-gray-900 dark:text-gray-100 text-sm ml-1 select-text" style={{ userSelect: "text", WebkitUserSelect: "text" }}>
                                  {finalText.length > 100
                                    ? `${finalText.substring(0, 100)}...`
                                    : finalText}
                                </span>
                              </div>
                            </div>
                          ) : (
                            <p className="text-gray-900 dark:text-gray-100 text-sm select-text" style={{ userSelect: "text", WebkitUserSelect: "text" }}>
                              {finalText.length > 100
                                ? `${finalText.substring(0, 100)}...`
                                : finalText}
                            </p>
                          )}
                        </div>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleCopyText(entry);
                          }}
                          className="relative z-20 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                          title={t("settings.history.copyToClipboard")}
                        >
                          {copiedId === entry.id ? (
                            <Check className="w-4 h-4 text-green-500" />
                          ) : (
                            <Copy className="w-4 h-4 text-gray-500 dark:text-gray-400" />
                          )}
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
