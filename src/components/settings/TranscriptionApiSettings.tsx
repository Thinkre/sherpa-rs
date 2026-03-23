import React, { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Plus,
  Trash2,
  Eye,
  EyeOff,
  Check,
  Copy,
  Settings,
  Play,
} from "lucide-react";
import { useSettings } from "../../hooks/useSettings";
import { SettingsGroup } from "../ui";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { Dropdown } from "../ui/Dropdown";
import { Alert } from "../ui/Alert";
import { commands } from "@/bindings";
import type { TranscriptionApiProvider, TranscriptionApiConfig } from "@/bindings";

export const TranscriptionApiSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating, refreshSettings, settings } =
    useSettings();

  // API Configs state - read directly from settings for reactivity
  const apiConfigs = (settings?.transcription_api_configs || []) as TranscriptionApiConfig[];
  const selectedConfigId = settings?.selected_transcription_api_config_id || null;
  const useTranscriptionApi = settings?.use_transcription_api || false;
  const [editingConfigId, setEditingConfigId] = useState<string | null>(null);
  const [isCreatingConfig, setIsCreatingConfig] = useState(false);
  const [draftConfigName, setDraftConfigName] = useState("");
  const [draftConfigProviderId, setDraftConfigProviderId] = useState<string>("");
  const [draftConfigApiKey, setDraftConfigApiKey] = useState<string>("");
  const [draftConfigModel, setDraftConfigModel] = useState<string>("");
  const [draftConfigApiUrl, setDraftConfigApiUrl] = useState<string>("");
  const [draftConfigLanguage, setDraftConfigLanguage] = useState<string>("");
  const [showConfigApiKey, setShowConfigApiKey] = useState(false);
  const [testingConfigId, setTestingConfigId] = useState<string | null>(null);
  const [configStatuses, setConfigStatuses] = useState<Record<string, string>>({});
  const [testError, setTestError] = useState<string | null>(null);

  const providers = settings?.transcription_api_providers || [];
  const providerOptions = providers.map((p: TranscriptionApiProvider) => ({
    value: p.id,
    label: p.label,
  }));

  // Get selected provider
  const selectedProvider = useMemo(() => {
    if (draftConfigProviderId) {
      return providers.find((p: TranscriptionApiProvider) => p.id === draftConfigProviderId);
    }
    return null;
  }, [draftConfigProviderId, providers]);

  const isLoading = isUpdating("transcription_api_configs");

  // Removed handleToggleUseApi - API selection is now done via clicking on API configs in ModelsPage

  const handleSelectConfig = async (configId: string) => {
    await updateSetting("selected_transcription_api_config_id", configId);
  };

  const handleStartCreateConfig = () => {
    setIsCreatingConfig(true);
    setEditingConfigId(null);
    setDraftConfigName("");
    const firstProviderId = providerOptions[0]?.value || "";
    setDraftConfigProviderId(firstProviderId);
    setDraftConfigApiKey("");
    setDraftConfigModel("");
    setDraftConfigLanguage("");
    // Set initial API URL from provider's base_url
    const provider = providers.find((p: TranscriptionApiProvider) => p.id === firstProviderId);
    setDraftConfigApiUrl(provider?.base_url || "");
    setShowConfigApiKey(false);
  };

  const handleEditConfig = (configId: string) => {
    const config = apiConfigs.find((c) => c.id === configId);
    if (!config) return;
    
    setIsCreatingConfig(false);
    setEditingConfigId(config.id);
    setDraftConfigName(config.name);
    setDraftConfigProviderId(config.provider_id);
    setDraftConfigApiKey(config.api_key);
    setDraftConfigModel(config.model);
    setDraftConfigApiUrl(config.api_url || "");
    setDraftConfigLanguage(config.language || "");
    setShowConfigApiKey(false);
  };

  const handleCancelConfig = () => {
    setIsCreatingConfig(false);
    setEditingConfigId(null);
    setDraftConfigName("");
    setDraftConfigProviderId("");
    setDraftConfigApiKey("");
    setDraftConfigModel("");
    setDraftConfigApiUrl("");
    setDraftConfigLanguage("");
  };

  const handleSaveConfig = async () => {
    if (!draftConfigName.trim() || !draftConfigProviderId || !draftConfigApiKey.trim() || !draftConfigModel.trim()) {
      return;
    }

    const newConfigs = [...apiConfigs];
    const selectedProvider = providers.find((p: TranscriptionApiProvider) => p.id === draftConfigProviderId);
    // Only save api_url if it's different from provider's default base_url or if provider is custom
    const shouldSaveApiUrl = draftConfigProviderId === "custom" || 
      (selectedProvider && draftConfigApiUrl.trim() !== selectedProvider.base_url);

    if (isCreatingConfig) {
      const newId = `config_${Date.now()}`;
      newConfigs.push({
        id: newId,
        name: draftConfigName.trim(),
        provider_id: draftConfigProviderId,
        api_key: draftConfigApiKey.trim(),
        model: draftConfigModel.trim(),
        status: "inactive",
        api_url: shouldSaveApiUrl ? draftConfigApiUrl.trim() : null,
        language: draftConfigLanguage.trim() || null,
      });
    } else if (editingConfigId) {
      const index = newConfigs.findIndex((c) => c.id === editingConfigId);
      if (index !== -1) {
        newConfigs[index] = {
          ...newConfigs[index],
          name: draftConfigName.trim(),
          provider_id: draftConfigProviderId,
          api_key: draftConfigApiKey.trim(),
          model: draftConfigModel.trim(),
          api_url: shouldSaveApiUrl ? draftConfigApiUrl.trim() : null,
          language: draftConfigLanguage.trim() || null,
        };
      }
    }

    await updateSetting("transcription_api_configs", newConfigs);
    setIsCreatingConfig(false);
    setEditingConfigId(null);
    // Clear draft fields
    setDraftConfigName("");
    setDraftConfigProviderId("");
    setDraftConfigApiKey("");
    setDraftConfigModel("");
    setDraftConfigApiUrl("");
    setDraftConfigLanguage("");
    await refreshSettings();
  };

  const handleDeleteConfig = async (configId: string) => {
    const newConfigs = apiConfigs.filter((c) => c.id !== configId);
    await updateSetting("transcription_api_configs", newConfigs);

    // If deleted config was selected, clear selection
    if (selectedConfigId === configId) {
      await updateSetting("selected_transcription_api_config_id", null);
    }

    // If deleted config was being edited, cancel editing
    if (editingConfigId === configId) {
      setEditingConfigId(null);
      handleCancelConfig();
    }

    await refreshSettings();
  };

  const handleCopyConfig = async (config: TranscriptionApiConfig) => {
    const newId = `config_${Date.now()}`;
    const newConfigs = [...apiConfigs, {
      ...config,
      id: newId,
      name: `${config.name} 副本`,
      status: "inactive",
    }];
    await updateSetting("transcription_api_configs", newConfigs);
    await refreshSettings();
  };

  const handleTestConfig = async (configId: string) => {
    const config = apiConfigs.find((c) => c.id === configId);
    if (!config) return;

    setTestingConfigId(config.id);
    setConfigStatuses({ ...configStatuses, [config.id]: "testing" });
    setTestError(null);

    try {
      const result = await commands.testTranscriptionApiWithFile(
        config.provider_id,
        config.api_key,
        config.model,
        config.api_url || null
      );

      if (result && result.status === "ok") {
        setConfigStatuses({ ...configStatuses, [config.id]: "success" });
        setTestError(null);
        // Show success message with transcription result if available
        if (result.data) {
          alert(`${t("settings.advanced.transcriptionApi.testSuccess")}\n\n${t("settings.advanced.transcriptionApi.transcriptionResult")}: ${result.data}`);
        }
      } else {
        const errorMsg = result?.error || "Unknown error";
        setConfigStatuses({ ...configStatuses, [config.id]: `error: ${errorMsg}` });
        setTestError(errorMsg);
        // Show detailed error in alert for debugging
        alert(`${t("settings.advanced.transcriptionApi.testFailed")}\n\n${errorMsg}`);
        console.error("Transcription API test failed:", errorMsg);
      }
    } catch (error: any) {
      const errorMsg = error?.toString() || String(error);
      const fullErrorMsg = error?.message || errorMsg;
      setConfigStatuses({ ...configStatuses, [config.id]: `error: ${fullErrorMsg}` });
      setTestError(fullErrorMsg);
      // Show detailed error in alert for debugging
      alert(`${t("settings.advanced.transcriptionApi.testFailed")}\n\n${fullErrorMsg}`);
      console.error("Transcription API test failed:", error);
      // Also log to stderr for terminal visibility
      console.error("=== Transcription API Test Error ===");
      console.error("Config:", config);
      console.error("Error:", fullErrorMsg);
      console.error("================================");
    } finally {
      setTestingConfigId(null);
    }
  };

  const handleProviderChange = (providerId: string) => {
    setDraftConfigProviderId(providerId);
    const provider = providers.find((p: TranscriptionApiProvider) => p.id === providerId);
    if (provider) {
      setDraftConfigApiUrl(provider.base_url);
    } else if (providerId === "custom") {
      setDraftConfigApiUrl("");
    }
  };

  return (
    <SettingsGroup
      title={t("settings.advanced.transcriptionApi.title")}
      description={t("settings.advanced.transcriptionApi.description")}
    >
      {/* Test Error Display */}
      {testError && (
        <Alert variant="error" className="mt-4">
          <div className="space-y-2">
            <div className="font-semibold">{t("settings.advanced.transcriptionApi.testFailed")}</div>
            <div className="text-xs font-mono whitespace-pre-wrap break-words max-h-60 overflow-y-auto">
              {testError}
            </div>
          </div>
        </Alert>
      )}

      {/* API Configurations List */}
      <div className="space-y-4 mt-6">
        {/* Config List */}
        {apiConfigs.map((config) => {
          const isSelected = selectedConfigId === config.id;
          const isEditing = editingConfigId === config.id;
          const provider = providers.find((p: TranscriptionApiProvider) => p.id === config.provider_id);
          const testStatus = configStatuses[config.id] || "untested";
          const isTesting = testingConfigId === config.id;
          
          return (
            <div key={config.id}>
              {/* Config Row */}
              {!isEditing && (
                <div
                  onClick={() => handleSelectConfig(config.id)}
                  className={`p-4 rounded-lg border-2 transition-all cursor-pointer ${
                    isSelected
                      ? "bg-logo-primary/20 border-logo-primary/50"
                      : "bg-background border-mid-gray/20 hover:border-mid-gray/40"
                  }`}
                >
                  <div className="flex items-center justify-between gap-4">
                    {/* Left side: Config name and details */}
                    <div className="flex-1 min-w-0">
                      {/* First line: Config name (bold, larger font) */}
                      <div className={`text-base font-bold mb-1 ${isSelected ? "text-logo-primary" : "text-text"}`}>
                        {config.name}
                      </div>
                      {/* Second line: Platform and model */}
                      <div className={`text-sm text-mid-gray flex items-center gap-2`}>
                        <span>{provider?.label?.replace(/\s*\([^)]*\)\s*/g, "").trim() || config.provider_id}</span>
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
                          handleEditConfig(config.id);
                        }}
                        className={isSelected ? "text-logo-primary hover:text-logo-primary" : ""}
                        title={t("common.settings") || "设置"}
                      >
                        <Settings className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleCopyConfig(config);
                        }}
                        className={isSelected ? "text-logo-primary hover:text-logo-primary" : ""}
                        title={t("settings.advanced.llmRoles.copyAndModify") || "复制并修改"}
                      >
                        <Copy className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleTestConfig(config.id);
                        }}
                        disabled={isTesting}
                        className={isSelected ? "text-logo-primary hover:text-logo-primary" : ""}
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
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteConfig(config.id);
                        }}
                        className={isSelected ? "text-logo-primary hover:text-red-600" : "hover:text-red-600"}
                        title={t("common.delete") || "删除"}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </div>
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
                      value={draftConfigName}
                      onChange={(e) => setDraftConfigName(e.target.value)}
                      placeholder={t("settings.advanced.transcriptionApi.configName")}
                      variant="compact"
                      className="w-full"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.provider")}
                    </label>
                    <Dropdown
                      selectedValue={draftConfigProviderId}
                      onSelect={handleProviderChange}
                      options={providerOptions}
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.apiUrl")}
                    </label>
                    {(() => {
                      const selectedProvider = providers.find((p: TranscriptionApiProvider) => p.id === draftConfigProviderId);
                      const isReadOnly = selectedProvider && !selectedProvider.allow_base_url_edit && draftConfigProviderId !== "custom";
                      return (
                        <Input
                          type="text"
                          value={draftConfigApiUrl}
                          onChange={(e) => setDraftConfigApiUrl(e.target.value)}
                          placeholder={t("settings.advanced.transcriptionApi.apiUrlPlaceholder")}
                          variant="compact"
                          className="w-full"
                          disabled={isReadOnly}
                          title={isReadOnly ? t("settings.postProcessing.api.baseUrl.description") || "此提供商的API URL不可编辑" : undefined}
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
                        type={showConfigApiKey ? "text" : "password"}
                        value={draftConfigApiKey}
                        onChange={(e) => setDraftConfigApiKey(e.target.value)}
                        placeholder={t("settings.advanced.transcriptionApi.apiKey")}
                        variant="compact"
                        className="w-full pr-10"
                      />
                      <button
                        type="button"
                        onClick={() => setShowConfigApiKey(!showConfigApiKey)}
                        className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-mid-gray hover:text-logo-primary transition-colors"
                      >
                        {showConfigApiKey ? (
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
                      value={draftConfigModel}
                      onChange={(e) => setDraftConfigModel(e.target.value)}
                      placeholder={t("settings.advanced.transcriptionApi.model")}
                      variant="compact"
                      className="w-full"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.transcriptionApi.language")}
                    </label>
                    <Input
                      type="text"
                      value={draftConfigLanguage}
                      onChange={(e) => setDraftConfigLanguage(e.target.value)}
                      placeholder={t("settings.advanced.transcriptionApi.languagePlaceholder")}
                      variant="compact"
                      className="w-full"
                    />
                  </div>
                  <div className="flex gap-2">
                    <Button
                      onClick={handleSaveConfig}
                      variant="primary"
                      size="md"
                      disabled={
                        !draftConfigName.trim() || 
                        !draftConfigProviderId || 
                        !draftConfigApiKey.trim() || 
                        !draftConfigModel.trim()
                      }
                    >
                      {t("common.save")}
                    </Button>
                    <Button onClick={handleCancelConfig} variant="secondary" size="md">
                      {t("common.cancel")}
                    </Button>
                  </div>
                </div>
              )}
            </div>
          );
        })}

        {/* Create Config Form */}
        {isCreatingConfig && (
          <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20 space-y-4">
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.configName")}
              </label>
              <Input
                type="text"
                value={draftConfigName}
                onChange={(e) => setDraftConfigName(e.target.value)}
                placeholder={t("settings.advanced.transcriptionApi.configName")}
                variant="compact"
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.provider")}
              </label>
              <Dropdown
                selectedValue={draftConfigProviderId}
                onSelect={handleProviderChange}
                options={providerOptions}
              />
            </div>
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.apiUrl")}
              </label>
              {(() => {
                const selectedProvider = providers.find((p: TranscriptionApiProvider) => p.id === draftConfigProviderId);
                const isReadOnly = selectedProvider && !selectedProvider.allow_base_url_edit && draftConfigProviderId !== "custom";
                return (
                  <Input
                    type="text"
                    value={draftConfigApiUrl}
                    onChange={(e) => setDraftConfigApiUrl(e.target.value)}
                    placeholder={t("settings.advanced.transcriptionApi.apiUrlPlaceholder")}
                    variant="compact"
                    className="w-full"
                    disabled={isReadOnly}
                    title={isReadOnly ? t("settings.postProcessing.api.baseUrl.description") || "此提供商的API URL不可编辑" : undefined}
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
                  type={showConfigApiKey ? "text" : "password"}
                  value={draftConfigApiKey}
                  onChange={(e) => setDraftConfigApiKey(e.target.value)}
                  placeholder={t("settings.advanced.transcriptionApi.apiKey")}
                  variant="compact"
                  className="w-full pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowConfigApiKey(!showConfigApiKey)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-mid-gray hover:text-logo-primary transition-colors"
                >
                  {showConfigApiKey ? (
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
                value={draftConfigModel}
                onChange={(e) => setDraftConfigModel(e.target.value)}
                placeholder={t("settings.advanced.transcriptionApi.model")}
                variant="compact"
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-semibold text-text mb-2">
                {t("settings.advanced.transcriptionApi.language")}
              </label>
              <Input
                type="text"
                value={draftConfigLanguage}
                onChange={(e) => setDraftConfigLanguage(e.target.value)}
                placeholder={t("settings.advanced.transcriptionApi.languagePlaceholder")}
                variant="compact"
                className="w-full"
              />
            </div>
            <div className="flex gap-2">
              <Button
                onClick={handleSaveConfig}
                variant="primary"
                size="md"
                disabled={
                  !draftConfigName.trim() || 
                  !draftConfigProviderId || 
                  !draftConfigApiKey.trim() || 
                  !draftConfigModel.trim()
                }
              >
                {t("common.save")}
              </Button>
              <Button onClick={handleCancelConfig} variant="secondary" size="md">
                {t("common.cancel")}
              </Button>
            </div>
          </div>
        )}

        {/* Add Config Button */}
        {!isCreatingConfig && editingConfigId === null && (
          <div
            onClick={handleStartCreateConfig}
            className="p-4 rounded-lg border-2 border-dashed border-mid-gray/30 bg-mid-gray/5 hover:border-mid-gray/50 hover:bg-mid-gray/10 transition-all cursor-pointer"
          >
            <div className="flex items-center justify-end">
              <Button variant="ghost" size="sm" className="text-mid-gray hover:text-logo-primary">
                <Plus className="h-4 w-4 mr-1" />
                {t("settings.advanced.transcriptionApi.addConfig")}
              </Button>
            </div>
          </div>
        )}
      </div>
    </SettingsGroup>
  );
};
