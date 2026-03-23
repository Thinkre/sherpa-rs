import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { commands, type ModelInfo, type TranscriptionApiConfig, type TranscriptionApiProvider } from "@/bindings";
import { getTranslatedModelName } from "@/lib/utils/modelTranslation";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Search,
  Settings as SettingsIcon,
  Mic,
  CheckCircle,
  Cloud,
  Settings,
  Copy,
  Play,
  Trash2,
  Plus,
  Eye,
  EyeOff,
  FolderOpen,
  RefreshCw,
} from "lucide-react";
import { Input } from "../ui/Input";
import { Button } from "../ui/Button";
import { Textarea } from "../ui/Textarea";
import { Dropdown } from "../ui/Dropdown";
import { Alert } from "../ui/Alert";
import { useSettings } from "@/hooks/useSettings";
import UpdateChecker from "../update-checker";

type ModelCategory = "all" | "local" | "api";

interface ModelCardProps {
  model: ModelInfo;
  isActive: boolean;
  onSelect: (modelId: string) => void;
  onDownload: (modelId: string) => void;
  onDelete: (modelId: string) => void;
  onCancel: (modelId: string) => void;
  isDownloading: boolean;
  downloadProgress?: number;
}

const ModelCard: React.FC<ModelCardProps> = ({
  model,
  isActive,
  onSelect,
  onDownload,
  onDelete,
  onCancel,
  isDownloading,
  downloadProgress,
}) => {
  const { t } = useTranslation();
  const modelName = getTranslatedModelName(model, t);
  const [showDeleteConfirm, setShowDeleteConfirm] = React.useState(false);

  // Determine if this is a local model (downloaded)
  const isLocal = model.is_downloaded;

  return (
    <div
      className={`border rounded-lg p-4 transition-all cursor-pointer ${
        isActive
          ? "border-logo-primary bg-logo-primary/5"
          : "border-mid-gray/20 hover:border-mid-gray/40"
      }`}
      onClick={() => {
        console.log("ModelCard clicked:", { modelId: model.id, isLocal, isDownloading });
        if (isLocal && !isDownloading) {
          console.log("Calling onSelect for model:", model.id);
          onSelect(model.id);
        } else {
          console.log("Model not selectable:", { isLocal, isDownloading });
        }
      }}
    >
      <div className="flex items-start gap-3">
        {/* Model Icon */}
        <div className="w-12 h-12 rounded-lg bg-logo-primary/10 flex items-center justify-center shrink-0">
          <span className="text-xl font-bold text-logo-primary">
            {modelName.charAt(0)}
          </span>
        </div>

        {/* Model Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1 flex-wrap">
            <h3 className="font-semibold text-sm truncate">{modelName}</h3>
            {isActive && (
              <CheckCircle className="w-4 h-4 text-logo-primary shrink-0" />
            )}
            {model.source != null && (
              <span
                className={`text-[10px] px-1.5 py-0.5 rounded ${
                  model.source === "download"
                    ? "bg-mid-gray/15 text-mid-gray"
                    : "bg-logo-primary/10 text-logo-primary"
                }`}
              >
                {model.source === "download"
                  ? t("models.sourceDownload")
                  : t("models.sourceLocalImport")}
              </span>
            )}
          </div>

          <div className="flex items-center justify-between gap-3 mb-3">
            <p className="text-xs text-mid-gray">{model.description}</p>
            
            {/* Actions - moved up to be inline with description */}
            <div className="flex items-center gap-2 flex-shrink-0">
              {isLocal ? (
                <>
                  {isActive && (
                    <span className="text-xs text-logo-primary font-medium">
                      {t("models.currentlyUsing")}
                    </span>
                  )}
                  {!showDeleteConfirm ? (
                    <button
                      className="px-3 py-1.5 rounded-lg border border-red-500 text-red-500 text-xs font-medium hover:bg-red-50 transition-colors"
                      onClick={(e) => {
                        e.stopPropagation();
                        setShowDeleteConfirm(true);
                      }}
                    >
                      {t("common.delete")}
                    </button>
                  ) : (
                    <div className="flex gap-2">
                      <button
                        className="px-3 py-1.5 rounded-lg border border-mid-gray/20 text-xs font-medium hover:bg-mid-gray/10 transition-colors"
                        onClick={(e) => {
                          e.stopPropagation();
                          setShowDeleteConfirm(false);
                        }}
                      >
                        {t("common.cancel")}
                      </button>
                      <button
                        className="px-3 py-1.5 rounded-lg bg-red-500 text-white text-xs font-medium hover:bg-red-600 transition-colors"
                        onClick={(e) => {
                          e.stopPropagation();
                          onDelete(model.id);
                          setShowDeleteConfirm(false);
                        }}
                      >
                        {t("common.delete")}
                      </button>
                    </div>
                  )}
                </>
              ) : isDownloading ? (
                <button
                  className="px-3 py-1.5 rounded-lg border border-mid-gray/20 text-xs font-medium hover:bg-mid-gray/10 transition-colors"
                  onClick={(e) => {
                    e.stopPropagation();
                    onCancel(model.id);
                  }}
                >
                  {t("modelSelector.cancelDownload")}
                </button>
              ) : (
                <button
                  className="px-3 py-1.5 rounded-lg bg-logo-primary text-white text-xs font-medium hover:bg-logo-primary/90 transition-colors disabled:opacity-50"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDownload(model.id);
                  }}
                  disabled={isDownloading}
                >
                  {t("modelSelector.download")}
                </button>
              )}
            </div>
          </div>

          {/* Metadata */}
          <div className="flex items-center gap-3 text-xs text-mid-gray">
            <div className="flex items-center gap-1">
              <span>{t("models.accuracy")}</span>
              <div className="flex gap-0.5">
                {[...Array(5)].map((_, i) => (
                  <div
                    key={i}
                    className={`w-1 h-3 rounded-full ${
                      i < model.accuracy_score
                        ? "bg-logo-primary"
                        : "bg-mid-gray/20"
                    }`}
                  />
                ))}
              </div>
            </div>

            <div className="flex items-center gap-1">
              <span>{t("models.speed")}</span>
              <div className="flex gap-0.5">
                {[...Array(5)].map((_, i) => (
                  <div
                    key={i}
                    className={`w-1 h-3 rounded-full ${
                      i < model.speed_score
                        ? "bg-logo-primary"
                        : "bg-mid-gray/20"
                    }`}
                  />
                ))}
              </div>
            </div>

            <div className="flex items-center gap-1">
              <span>{t("models.multiLanguage")}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Download Progress */}
      {isDownloading && downloadProgress !== undefined && (
        <div className="mt-3 w-full bg-mid-gray/10 rounded-full h-1.5 overflow-hidden">
          <div
            className="bg-logo-primary h-full transition-all duration-300"
            style={{ width: `${downloadProgress}%` }}
          />
        </div>
      )}
    </div>
  );
};

interface ApiModelCardProps {
  config: TranscriptionApiConfig;
  isActive: boolean;
  onSelect: (configId: string) => void;
  onEdit: (configId: string) => void;
  onCopy: (config: TranscriptionApiConfig) => void;
  onTest: (configId: string) => void;
  onDelete: (configId: string) => void;
  providerName: string;
  isTesting: boolean;
  testStatus: string;
}

const ApiModelCard: React.FC<ApiModelCardProps> = ({
  config,
  isActive,
  onSelect,
  onEdit,
  onCopy,
  onTest,
  onDelete,
  providerName,
  isTesting,
  testStatus,
}) => {
  const { t } = useTranslation();
  const [showDeleteConfirm, setShowDeleteConfirm] = React.useState(false);

  return (
    <div
      onClick={() => onSelect(config.id)}
      className={`p-4 rounded-lg border-2 transition-all cursor-pointer ${
        isActive
          ? "bg-logo-primary/20 border-logo-primary/50"
          : "bg-background border-mid-gray/20 hover:border-mid-gray/40"
      }`}
    >
      <div className="flex items-center justify-between gap-4">
        {/* Left side: Config name and details */}
        <div className="flex-1 min-w-0">
          {/* First line: Config name (bold, larger font) */}
          <div className={`text-base font-bold mb-1 ${isActive ? "text-logo-primary" : "text-text"}`}>
            {config.name}
          </div>
          {/* Second line: Platform and model */}
          <div className={`text-sm text-mid-gray flex items-center gap-2`}>
            <span>{providerName?.replace(/\s*\([^)]*\)\s*/g, "").trim() || config.provider_id}</span>
            <span>•</span>
            <span>{config.model}</span>
          </div>
        </div>
        
        {/* Right side: Action buttons */}
        <div className="flex items-center gap-2 flex-shrink-0">
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onEdit(config.id);
            }}
            className={isActive ? "text-logo-primary hover:text-logo-primary" : ""}
            title={t("common.settings") || "设置"}
          >
            <Settings className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onCopy(config);
            }}
            className={isActive ? "text-logo-primary hover:text-logo-primary" : ""}
            title={t("settings.advanced.llmRoles.copyAndModify") || "复制并修改"}
          >
            <Copy className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onTest(config.id);
            }}
            disabled={isTesting}
            className={isActive ? "text-logo-primary hover:text-logo-primary" : ""}
            title={t("settings.advanced.transcriptionApi.test") || "测试"}
          >
            {isTesting ? (
              <div className="h-4 w-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
            ) : (
              <Play className="h-4 w-4" />
            )}
          </Button>
          <div 
            className={`text-xs px-2 py-1 rounded ${
              testStatus === "success" 
                ? "text-green-500 bg-green-500/10" 
                : testStatus.startsWith("error")
                ? "text-red-500 bg-red-500/10"
                : testStatus === "testing"
                ? "text-logo-primary bg-logo-primary/10"
                : testStatus === "untested"
                ? "text-mid-gray bg-mid-gray/10"
                : "text-mid-gray bg-mid-gray/10"
            }`} 
            title={testStatus.startsWith("error:") ? testStatus.substring(6) : undefined}
            onClick={(e) => e.stopPropagation()}
          >
            {testStatus === "success" 
              ? t("settings.advanced.transcriptionApi.status.success") || "连接成功"
              : testStatus.startsWith("error")
              ? t("settings.advanced.transcriptionApi.status.error") || "连接失败" 
              : testStatus === "testing"
              ? t("settings.advanced.transcriptionApi.status.testing") || "测试中..."
              : testStatus === "untested"
              ? t("settings.advanced.transcriptionApi.status.untested") || "未测试"
              : testStatus}
          </div>
          {!showDeleteConfirm ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={(e) => {
                e.stopPropagation();
                setShowDeleteConfirm(true);
              }}
              className={isActive ? "text-logo-primary hover:text-red-600" : "hover:text-red-600"}
              title={t("common.delete") || "删除"}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          ) : (
            <div className="flex items-center gap-1">
              <button
                className="px-2 py-1 text-xs rounded hover:bg-mid-gray/10 transition-colors"
                onClick={(e) => {
                  e.stopPropagation();
                  setShowDeleteConfirm(false);
                }}
              >
                {t("common.cancel") || "取消"}
              </button>
              <button
                className="px-2 py-1 text-xs rounded bg-red-500 text-white hover:bg-red-600 transition-colors"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(config.id);
                  setShowDeleteConfirm(false);
                }}
              >
                {t("common.delete") || "删除"}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export const ModelsPage: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [currentModelId, setCurrentModelId] = useState<string>("");
  const [selectedCategory, setSelectedCategory] =
    useState<ModelCategory>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [downloadProgress, setDownloadProgress] = useState<
    Record<string, number>
  >({});
  
  // Get API configs from settings
  const apiConfigs = (settings?.transcription_api_configs || []) as TranscriptionApiConfig[];
  const selectedApiConfigId = settings?.selected_transcription_api_config_id || null;
  const useTranscriptionApi = settings?.use_transcription_api || false;
  const providers = (settings?.transcription_api_providers || []) as TranscriptionApiProvider[];
  
  // API Config management state
  const [editingApiConfigId, setEditingApiConfigId] = useState<string | null>(null);
  const [isCreatingApiConfig, setIsCreatingApiConfig] = useState(false);
  const [draftApiConfigName, setDraftApiConfigName] = useState("");
  const [draftApiConfigProviderId, setDraftApiConfigProviderId] = useState<string>("");
  const [draftApiConfigApiKey, setDraftApiConfigApiKey] = useState<string>("");
  const [draftApiConfigModel, setDraftApiConfigModel] = useState<string>("");
  const [draftApiConfigApiUrl, setDraftApiConfigApiUrl] = useState<string>("");
  const [draftApiConfigLanguage, setDraftApiConfigLanguage] = useState<string>("");
  const [showApiConfigApiKey, setShowApiConfigApiKey] = useState(false);
  const [testingApiConfigId, setTestingApiConfigId] = useState<string | null>(null);
  const [apiConfigStatuses, setApiConfigStatuses] = useState<Record<string, string>>({});
  const [apiTestError, setApiTestError] = useState<string | null>(null);
  
  const providerOptions = providers.map((p: TranscriptionApiProvider) => ({
    value: p.id,
    label: p.label,
  }));

  useEffect(() => {
    loadModels();
    loadCurrentModel();

    // Listen for model download progress
    const downloadProgressUnlisten = listen<{
      model_id: string;
      percentage: number;
    }>("model-download-progress", (event) => {
      setDownloadProgress((prev) => ({
        ...prev,
        [event.payload.model_id]: event.payload.percentage,
      }));
    });

    // Listen for download completion
    const downloadCompleteUnlisten = listen<string>(
      "model-download-complete",
      (event) => {
        setDownloadProgress((prev) => {
          const newProgress = { ...prev };
          delete newProgress[event.payload];
          return newProgress;
        });
        loadModels();
      },
    );

    // Listen for model import completion
    const modelImportedUnlisten = listen<string>(
      "model-imported",
      () => {
        loadModels();
      },
    );

    return () => {
      downloadProgressUnlisten.then((fn) => fn());
      downloadCompleteUnlisten.then((fn) => fn());
      modelImportedUnlisten.then((fn) => fn());
    };
  }, []);

  const loadModels = async () => {
    try {
      const result = await commands.getAvailableModels();
      if (result.status === "ok") {
        console.log("Loaded models:", result.data.length, "models");
        // Log imported models for debugging
        const importedModels = result.data.filter(m => m.id.startsWith("custom-"));
        if (importedModels.length > 0) {
          console.log("Imported models found:", importedModels.map(m => ({ id: m.id, is_downloaded: m.is_downloaded, name: m.name })));
        }
        setModels(result.data);
      }
    } catch (err) {
      console.error("Failed to load models:", err);
    }
  };

  const handleReloadConfig = async () => {
    try {
      const result = await commands.reloadModelsConfig();
      if (result.status === "ok") {
        await loadModels();
      } else {
        console.error("Failed to reload config:", result.error);
        alert(`重新加载配置失败: ${result.error}`);
      }
    } catch (err) {
      console.error("Failed to reload config:", err);
      alert(`重新加载配置时出错: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const handleImportLocalModel = async () => {
    console.log("handleImportLocalModel called");
    try {
      console.log("Opening folder dialog...");
      const selected = await open({
        directory: true,
        multiple: false,
        title: t("models.importLocalModel.title"),
      });

      console.log("Dialog result:", selected);

      if (!selected) {
        console.log("User cancelled folder selection");
        return; // User cancelled
      }

      // Handle both string and array return types
      const folderPath = typeof selected === "string" ? selected : selected[0];
      if (!folderPath) {
        console.log("No folder path selected");
        return;
      }

      console.log("Importing model from:", folderPath);
      const result = await commands.importLocalModelFolder(folderPath);
      if (result.status === "ok") {
        // Reload models to show the newly imported model
        await loadModels();
        // Optionally show success message
        console.log("Model imported successfully:", result.data);
      } else {
        console.error("Failed to import model:", result.error);
        // Optionally show error message to user
        alert(`导入模型失败: ${result.error}`);
      }
    } catch (err) {
      console.error("Failed to import model:", err);
      alert(`导入模型时出错: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const loadCurrentModel = async () => {
    try {
      const result = await commands.getCurrentModel();
      if (result.status === "ok") {
        setCurrentModelId(result.data);
      }
    } catch (err) {
      console.error("Failed to load current model:", err);
    }
  };

  const handleModelSelect = async (modelId: string) => {
    try {
      console.log("Selecting model:", modelId);
      // Set use_transcription_api to false when selecting local model
      await updateSetting("use_transcription_api", false);
      const result = await commands.setActiveModel(modelId);
      if (result.status === "ok") {
        console.log("Model selected successfully:", modelId);
        setCurrentModelId(modelId);
      } else {
        console.error("Failed to select model:", result.error);
        alert(`选择模型失败: ${result.error}`);
      }
    } catch (err) {
      console.error("Failed to select model:", err);
      alert(`选择模型时出错: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const handleApiConfigSelect = async (configId: string) => {
    try {
      // Set use_transcription_api to true and select the API config
      await updateSetting("use_transcription_api", true);
      await updateSetting("selected_transcription_api_config_id", configId);
    } catch (err) {
      console.error("Failed to select API config:", err);
    }
  };

  // API Config handlers
  const handleStartCreateApiConfig = () => {
    setIsCreatingApiConfig(true);
    setEditingApiConfigId(null);
    setDraftApiConfigName("");
    const firstProviderId = providerOptions[0]?.value || "";
    setDraftApiConfigProviderId(firstProviderId);
    setDraftApiConfigApiKey("");
    setDraftApiConfigModel("");
    setDraftApiConfigLanguage("");
    const provider = providers.find((p) => p.id === firstProviderId);
    setDraftApiConfigApiUrl(provider?.base_url || "");
    setShowApiConfigApiKey(false);
  };

  const handleEditApiConfig = (configId: string) => {
    const config = apiConfigs.find((c) => c.id === configId);
    if (!config) return;
    
    setIsCreatingApiConfig(false);
    setEditingApiConfigId(config.id);
    setDraftApiConfigName(config.name);
    setDraftApiConfigProviderId(config.provider_id);
    setDraftApiConfigApiKey(config.api_key);
    setDraftApiConfigModel(config.model);
    setDraftApiConfigApiUrl(config.api_url || "");
    setDraftApiConfigLanguage(config.language || "");
    setShowApiConfigApiKey(false);
  };

  const handleCancelApiConfig = () => {
    setIsCreatingApiConfig(false);
    setEditingApiConfigId(null);
    setDraftApiConfigName("");
    setDraftApiConfigProviderId("");
    setDraftApiConfigApiKey("");
    setDraftApiConfigModel("");
    setDraftApiConfigApiUrl("");
    setDraftApiConfigLanguage("");
  };

  const handleSaveApiConfig = async () => {
    if (!draftApiConfigName.trim() || !draftApiConfigProviderId || !draftApiConfigApiKey.trim() || !draftApiConfigModel.trim()) {
      return;
    }

    const newConfigs = [...apiConfigs];
    const selectedProvider = providers.find((p) => p.id === draftApiConfigProviderId);
    const shouldSaveApiUrl = draftApiConfigProviderId === "custom" || 
      (selectedProvider && draftApiConfigApiUrl.trim() !== selectedProvider.base_url);

    if (isCreatingApiConfig) {
      const newId = `config_${Date.now()}`;
      newConfigs.push({
        id: newId,
        name: draftApiConfigName.trim(),
        provider_id: draftApiConfigProviderId,
        api_key: draftApiConfigApiKey.trim(),
        model: draftApiConfigModel.trim(),
        status: "inactive",
        api_url: shouldSaveApiUrl ? draftApiConfigApiUrl.trim() : null,
        language: draftApiConfigLanguage.trim() || null,
      });
    } else if (editingApiConfigId) {
      const index = newConfigs.findIndex((c) => c.id === editingApiConfigId);
      if (index !== -1) {
        newConfigs[index] = {
          ...newConfigs[index],
          name: draftApiConfigName.trim(),
          provider_id: draftApiConfigProviderId,
          api_key: draftApiConfigApiKey.trim(),
          model: draftApiConfigModel.trim(),
          api_url: shouldSaveApiUrl ? draftApiConfigApiUrl.trim() : null,
          language: draftApiConfigLanguage.trim() || null,
        };
      }
    }

    await updateSetting("transcription_api_configs", newConfigs);
    handleCancelApiConfig();
  };

  const handleDeleteApiConfig = async (configId: string) => {
    const newConfigs = apiConfigs.filter((c) => c.id !== configId);
    await updateSetting("transcription_api_configs", newConfigs);

    if (selectedApiConfigId === configId) {
      await updateSetting("selected_transcription_api_config_id", null);
      await updateSetting("use_transcription_api", false);
    }

    if (editingApiConfigId === configId) {
      handleCancelApiConfig();
    }
  };

  const handleCopyApiConfig = async (config: TranscriptionApiConfig) => {
    const newId = `config_${Date.now()}`;
    const newConfigs = [...apiConfigs, {
      ...config,
      id: newId,
      name: `${config.name} 副本`,
      status: "inactive",
    }];
    await updateSetting("transcription_api_configs", newConfigs);
  };

  const handleTestApiConfig = async (configId: string) => {
    const config = apiConfigs.find((c) => c.id === configId);
    if (!config) return;

    setTestingApiConfigId(config.id);
    setApiConfigStatuses({ ...apiConfigStatuses, [config.id]: "testing" });
    setApiTestError(null);

    try {
      const result = await commands.testTranscriptionApiWithFile(
        config.provider_id,
        config.api_key,
        config.model,
        config.api_url || null
      );

      if (result && result.status === "ok") {
        setApiConfigStatuses({ ...apiConfigStatuses, [config.id]: "success" });
        setApiTestError(null);
        if (result.data) {
          alert(`${t("settings.advanced.transcriptionApi.testSuccess")}\n\n${t("settings.advanced.transcriptionApi.transcriptionResult")}: ${result.data}`);
        }
      } else {
        const errorMsg = result?.error || "Unknown error";
        setApiConfigStatuses({ ...apiConfigStatuses, [config.id]: `error: ${errorMsg}` });
        setApiTestError(errorMsg);
        alert(`${t("settings.advanced.transcriptionApi.testFailed")}\n\n${errorMsg}`);
      }
    } catch (error: any) {
      const errorMsg = error?.toString() || String(error);
      setApiConfigStatuses({ ...apiConfigStatuses, [config.id]: `error: ${errorMsg}` });
      setApiTestError(errorMsg);
      alert(`${t("settings.advanced.transcriptionApi.testFailed")}\n\n${errorMsg}`);
    } finally {
      setTestingApiConfigId(null);
    }
  };

  const handleApiProviderChange = (providerId: string) => {
    setDraftApiConfigProviderId(providerId);
    const provider = providers.find((p) => p.id === providerId);
    if (provider) {
      setDraftApiConfigApiUrl(provider.base_url);
    } else if (providerId === "custom") {
      setDraftApiConfigApiUrl("");
    }
  };

  const handleModelDownload = async (modelId: string) => {
    try {
      await commands.downloadModel(modelId);
    } catch (err) {
      console.error("Failed to download model:", err);
    }
  };

  const handleModelDelete = async (modelId: string) => {
    try {
      await commands.deleteModel(modelId);
      loadModels();
    } catch (err) {
      console.error("Failed to delete model:", err);
    }
  };

  const handleModelCancel = async (modelId: string) => {
    try {
      await commands.cancelDownload(modelId);
    } catch (err) {
      console.error("Failed to cancel download:", err);
    }
  };

  // Filter models based on category and search
  const filteredLocalModels = models.filter((model) => {
    // For "local" category, show all local-engine models (both downloaded and available for download)
    // Don't filter by is_downloaded - let users see what they can download

    // API models should not appear in local models list
    if (model.engine_type === "Api") {
      return false;
    }

    // Search filter
    if (searchQuery) {
      const modelName = getTranslatedModelName(model, t).toLowerCase();
      const query = searchQuery.toLowerCase();
      if (
        !modelName.includes(query) &&
        !model.description.toLowerCase().includes(query)
      ) {
        return false;
      }
    }
    return true;
  });

  // Filter API configs based on search
  const filteredApiConfigs = apiConfigs.filter((config) => {
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      const provider = providers.find((p) => p.id === config.provider_id);
      const providerName = provider?.label || config.provider_id;
      if (
        !config.name.toLowerCase().includes(query) &&
        !config.model.toLowerCase().includes(query) &&
        !providerName.toLowerCase().includes(query)
      ) {
        return false;
      }
    }
    return true;
  });

  // Get models/configs to display based on category
  const getDisplayItems = () => {
    if (selectedCategory === "local") {
      return { localModels: filteredLocalModels, apiConfigs: [] };
    } else if (selectedCategory === "api") {
      return { localModels: [], apiConfigs: filteredApiConfigs };
    } else {
      // "all" - show local models first, then API configs
      return { localModels: filteredLocalModels, apiConfigs: filteredApiConfigs };
    }
  };

  const { localModels, apiConfigs: displayApiConfigs } = getDisplayItems();

  const categories: { key: ModelCategory; label: string }[] = [
    { key: "all", label: t("models.categories.all") },
    { key: "local", label: t("models.categories.local") },
    { key: "api", label: t("models.categories.api") },
  ];

  // Get current model info for display
  const getCurrentModelDisplay = () => {
    if (useTranscriptionApi && selectedApiConfigId) {
      const config = apiConfigs.find((c) => c.id === selectedApiConfigId);
      if (config) {
        return {
          type: t("models.modelType.api"),
          name: `${config.name} (${config.model})`,
        };
      }
    } else if (currentModelId) {
      const model = models.find((m) => m.id === currentModelId);
      if (model) {
        return {
          type: t("models.modelType.local"),
          name: getTranslatedModelName(model, t),
        };
      }
    }
    return null;
  };

  const currentModelDisplay = getCurrentModelDisplay();

  const getCategoryDescription = () => {
    return t(`models.categoryDescriptions.${selectedCategory}`);
  };


  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold mb-2">{t("models.title")}</h1>
        <p className="text-sm text-mid-gray">{getCategoryDescription()}</p>
      </div>

      {/* Category Tabs and Search */}
      <div className="mb-6">
        <div className="flex items-center gap-2 mb-4 flex-wrap">
          {categories.map((cat) => (
            <button
              key={cat.key}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                selectedCategory === cat.key
                  ? "bg-logo-primary text-white"
                  : "bg-mid-gray/10 text-text hover:bg-mid-gray/20"
              }`}
              onClick={() => setSelectedCategory(cat.key)}
            >
              {cat.label}
            </button>
          ))}
        </div>

        {/* Search Bar */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-mid-gray z-10" />
          <Input
            type="text"
            placeholder={t("models.searchPlaceholder")}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-mid-gray/20 rounded-lg focus:outline-none focus:ring-2 focus:ring-logo-primary/50"
          />
        </div>
      </div>

      {/* Models List */}
      <div className="flex-1 overflow-y-auto space-y-3">
        {/* Import & config buttons - show when viewing local or all (so local models are visible) */}
        {(selectedCategory === "local" || selectedCategory === "all") && (
          <div className="mb-4 flex flex-wrap items-center gap-2">
            <Button
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                handleImportLocalModel();
              }}
              className="flex items-center gap-2"
              type="button"
            >
              <FolderOpen className="w-4 h-4" />
              {t("models.importLocalModel.button")}
            </Button>
            <Button
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                handleReloadConfig();
              }}
              className="flex items-center gap-2"
              type="button"
              variant="secondary"
            >
              <RefreshCw className="w-4 h-4" />
              {t("models.reloadConfig")}
            </Button>
            <Button
              onClick={async (e) => {
                e.preventDefault();
                e.stopPropagation();
                try {
                  const result = await commands.openAppDataDir();
                  if (result.status === "error") {
                    alert(result.error ?? "打开失败");
                  }
                } catch (err) {
                  alert(err instanceof Error ? err.message : String(err));
                }
              }}
              className="flex items-center gap-2"
              type="button"
              variant="secondary"
            >
              <FolderOpen className="w-4 h-4" />
              {t("models.openConfigFolder")}
            </Button>
            <span className="text-xs text-mid-gray ml-1 hidden sm:inline">
              {t("models.configHint")}
            </span>
          </div>
        )}

        {/* Local Models */}
        {localModels.length > 0 && (
          <>
            {localModels.map((model) => (
              <ModelCard
                key={model.id}
                model={model}
                isActive={!useTranscriptionApi && model.id === currentModelId}
                onSelect={handleModelSelect}
                onDownload={handleModelDownload}
                onDelete={handleModelDelete}
                onCancel={handleModelCancel}
                isDownloading={model.id in downloadProgress}
                downloadProgress={downloadProgress[model.id]}
              />
            ))}
          </>
        )}

        {/* API Configs */}
        {displayApiConfigs.map((config) => {
          const provider = providers.find((p) => p.id === config.provider_id);
          const providerName = provider?.label || config.provider_id;
          const isEditing = editingApiConfigId === config.id;
          const testStatus = apiConfigStatuses[config.id] || "untested";
          const isTesting = testingApiConfigId === config.id;
          
          return (
            <div key={config.id}>
              {!isEditing && (
                <ApiModelCard
                  config={config}
                  isActive={useTranscriptionApi && config.id === selectedApiConfigId}
                  onSelect={handleApiConfigSelect}
                  onEdit={handleEditApiConfig}
                  onCopy={handleCopyApiConfig}
                  onTest={handleTestApiConfig}
                  onDelete={handleDeleteApiConfig}
                  providerName={providerName}
                  isTesting={isTesting}
                  testStatus={testStatus}
                />
              )}
              
              {/* Edit Config Form */}
              {isEditing && (
                <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20 space-y-4">
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.configName")}
                    </label>
                    <Input
                      type="text"
                      value={draftApiConfigName}
                      onChange={(e) => setDraftApiConfigName(e.target.value)}
                      placeholder={t("settings.advanced.transcriptionApi.configName")}
                      className="w-full"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.provider")}
                    </label>
                    <Dropdown
                      selectedValue={draftApiConfigProviderId}
                      onSelect={handleApiProviderChange}
                      options={providerOptions}
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.apiUrl")}
                    </label>
                    {(() => {
                      const selectedProvider = providers.find((p) => p.id === draftApiConfigProviderId);
                      const isReadOnly = selectedProvider && !selectedProvider.allow_base_url_edit && draftApiConfigProviderId !== "custom";
                      return (
                        <Input
                          type="text"
                          value={draftApiConfigApiUrl}
                          onChange={(e) => setDraftApiConfigApiUrl(e.target.value)}
                          placeholder={t("settings.advanced.transcriptionApi.apiUrlPlaceholder")}
                          className="w-full"
                          disabled={isReadOnly}
                        />
                      );
                    })()}
                  </div>
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.apiKey")}
                    </label>
                    <div className="relative">
                      <Input
                        type={showApiConfigApiKey ? "text" : "password"}
                        value={draftApiConfigApiKey}
                        onChange={(e) => setDraftApiConfigApiKey(e.target.value)}
                        placeholder={t("settings.advanced.transcriptionApi.apiKey")}
                        className="w-full pr-10"
                      />
                      <button
                        type="button"
                        onClick={() => setShowApiConfigApiKey(!showApiConfigApiKey)}
                        className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-mid-gray hover:text-logo-primary transition-colors"
                      >
                        {showApiConfigApiKey ? (
                          <EyeOff className="h-4 w-4" />
                        ) : (
                          <Eye className="h-4 w-4" />
                        )}
                      </button>
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.model")}
                    </label>
                    <Input
                      type="text"
                      value={draftApiConfigModel}
                      onChange={(e) => setDraftApiConfigModel(e.target.value)}
                      placeholder={t("settings.advanced.transcriptionApi.model")}
                      className="w-full"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.language")}
                    </label>
                    <Input
                      type="text"
                      value={draftApiConfigLanguage}
                      onChange={(e) => setDraftApiConfigLanguage(e.target.value)}
                      placeholder={t("settings.advanced.transcriptionApi.languagePlaceholder")}
                      className="w-full"
                    />
                  </div>
                  <div className="flex gap-2">
                    <Button
                      onClick={handleSaveApiConfig}
                      variant="primary"
                      size="md"
                      disabled={
                        !draftApiConfigName.trim() || 
                        !draftApiConfigProviderId || 
                        !draftApiConfigApiKey.trim() || 
                        !draftApiConfigModel.trim()
                      }
                    >
                      {t("common.save")}
                    </Button>
                    <Button onClick={handleCancelApiConfig} variant="secondary" size="md">
                      {t("common.cancel")}
                    </Button>
                  </div>
                </div>
              )}
            </div>
          );
        })}
        
        {/* Create Config Form */}
        {isCreatingApiConfig && (
          <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20 space-y-4">
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.configName")}
              </label>
              <Input
                type="text"
                value={draftApiConfigName}
                onChange={(e) => setDraftApiConfigName(e.target.value)}
                placeholder={t("settings.advanced.transcriptionApi.configName")}
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.provider")}
              </label>
              <Dropdown
                selectedValue={draftApiConfigProviderId}
                onSelect={handleApiProviderChange}
                options={providerOptions}
              />
            </div>
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.apiUrl")}
              </label>
              {(() => {
                const selectedProvider = providers.find((p) => p.id === draftApiConfigProviderId);
                const isReadOnly = selectedProvider && !selectedProvider.allow_base_url_edit && draftApiConfigProviderId !== "custom";
                return (
                  <Input
                    type="text"
                    value={draftApiConfigApiUrl}
                    onChange={(e) => setDraftApiConfigApiUrl(e.target.value)}
                    placeholder={t("settings.advanced.transcriptionApi.apiUrlPlaceholder")}
                    className="w-full"
                    disabled={isReadOnly}
                  />
                );
              })()}
            </div>
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.apiKey")}
              </label>
              <div className="relative">
                <Input
                  type={showApiConfigApiKey ? "text" : "password"}
                  value={draftApiConfigApiKey}
                  onChange={(e) => setDraftApiConfigApiKey(e.target.value)}
                  placeholder={t("settings.advanced.transcriptionApi.apiKey")}
                  className="w-full pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowApiConfigApiKey(!showApiConfigApiKey)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-mid-gray hover:text-logo-primary transition-colors"
                >
                  {showApiConfigApiKey ? (
                    <EyeOff className="h-4 w-4" />
                  ) : (
                    <Eye className="h-4 w-4" />
                  )}
                </button>
              </div>
            </div>
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.model")}
              </label>
              <Input
                type="text"
                value={draftApiConfigModel}
                onChange={(e) => setDraftApiConfigModel(e.target.value)}
                placeholder={t("settings.advanced.transcriptionApi.model")}
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.language")}
              </label>
              <Input
                type="text"
                value={draftApiConfigLanguage}
                onChange={(e) => setDraftApiConfigLanguage(e.target.value)}
                placeholder={t("settings.advanced.transcriptionApi.languagePlaceholder")}
                className="w-full"
              />
            </div>
            <div className="flex gap-2">
              <Button
                onClick={handleSaveApiConfig}
                variant="primary"
                size="md"
                disabled={
                  !draftApiConfigName.trim() || 
                  !draftApiConfigProviderId || 
                  !draftApiConfigApiKey.trim() || 
                  !draftApiConfigModel.trim()
                }
              >
                {t("common.save")}
              </Button>
              <Button onClick={handleCancelApiConfig} variant="secondary" size="md">
                {t("common.cancel")}
              </Button>
            </div>
          </div>
        )}

        {/* Add Config Button - Show in API tab or All tab */}
        {(selectedCategory === "api" || selectedCategory === "all") && !isCreatingApiConfig && editingApiConfigId === null && (
          <button
            onClick={handleStartCreateApiConfig}
            className="w-full p-4 border-2 border-dashed border-mid-gray/30 rounded-lg bg-mid-gray/5 hover:bg-mid-gray/10 hover:border-mid-gray/50 transition-all duration-200 flex items-center justify-center gap-2 text-mid-gray hover:text-text group"
          >
            <Plus className="h-5 w-5 group-hover:scale-110 transition-transform" />
            <span className="text-sm font-medium">
              {t("settings.advanced.transcriptionApi.addConfig")}
            </span>
          </button>
        )}

        {/* Empty State */}
        {localModels.length === 0 && displayApiConfigs.length === 0 && !isCreatingApiConfig && (
          <div className="text-center py-12 text-mid-gray">
            <p>{t("modelSelector.noModelsAvailable")}</p>
          </div>
        )}
      </div>

      {/* Test Error Display */}
      {apiTestError && (
        <Alert variant="error" className="mt-4">
          <div className="space-y-2">
            <div className="font-semibold">{t("settings.advanced.transcriptionApi.testFailed")}</div>
            <div className="text-xs font-mono whitespace-pre-wrap break-words max-h-60 overflow-y-auto">
              {apiTestError}
            </div>
          </div>
        </Alert>
      )}

    </div>
  );
};
