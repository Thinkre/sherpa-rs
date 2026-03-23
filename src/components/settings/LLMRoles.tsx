import React, { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Plus,
  Trash2,
  Eye,
  EyeOff,
  Check,
  Copy,
  Sparkles,
  CheckCircle2,
  Globe,
  Code,
  Briefcase,
  Flame,
  Zap,
  Crown,
  Wifi,
  Heart,
  Users,
  Code2,
  BookOpen,
  Mic,
  Wrench,
  Folder,
  Settings,
  Play,
  Edit,
  type LucideIcon,
} from "lucide-react";
import { useSettings } from "../../hooks/useSettings";
import { SettingContainer, SettingsGroup } from "../ui";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { Textarea } from "../ui/Textarea";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { Dropdown } from "../ui/Dropdown";
import { ProviderSelect } from "./PostProcessingSettingsApi/ProviderSelect";
import { ApiKeyField } from "./PostProcessingSettingsApi/ApiKeyField";
import { commands } from "@/bindings";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import type { PostProcessProvider, LLMConfig } from "@/bindings";

// Icon mapping for lucide-react icons
const iconMap: Record<string, LucideIcon> = {
  sparkles: Sparkles,
  globe: Globe,
  code: Code,
  briefcase: Briefcase,
  flame: Flame,
  zap: Zap,
  crown: Crown,
  wifi: Wifi,
  heart: Heart,
  users: Users,
  code2: Code2,
  bookOpen: BookOpen,
  mic: Mic,
  wrench: Wrench,
  folder: Folder,
};

const availableIcons = [
  { name: "sparkles", label: "✨" },
  { name: "globe", label: "🌐" },
  { name: "code", label: "💻" },
  { name: "briefcase", label: "💼" },
  { name: "flame", label: "🔥" },
  { name: "zap", label: "⚡" },
  { name: "crown", label: "👑" },
  { name: "wifi", label: "📶" },
  { name: "heart", label: "❤️" },
  { name: "users", label: "👥" },
  { name: "code2", label: "</>" },
  { name: "bookOpen", label: "📖" },
  { name: "mic", label: "🎤" },
  { name: "wrench", label: "🔧" },
  { name: "folder", label: "📁" },
];


export const LLMRoles: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating, refreshSettings, settings } =
    useSettings();
  
  // LLM Configs state
  const llmConfigs = (getSetting("llm_configs") || []) as LLMConfig[];
  const selectedConfigId = getSetting("selected_llm_config_id") || null;
  const [editingConfigId, setEditingConfigId] = useState<string | null>(null);
  const [isCreatingConfig, setIsCreatingConfig] = useState(false);
  const [draftConfigName, setDraftConfigName] = useState("");
  const [draftConfigProviderId, setDraftConfigProviderId] = useState<string>("");
  const [draftConfigApiKey, setDraftConfigApiKey] = useState<string>("");
  const [draftConfigModel, setDraftConfigModel] = useState<string>("");
  const [draftConfigApiUrl, setDraftConfigApiUrl] = useState<string>("");
  const [showConfigApiKey, setShowConfigApiKey] = useState(false);
  const [testingConfigId, setTestingConfigId] = useState<string | null>(null);
  const [configStatuses, setConfigStatuses] = useState<Record<string, string>>({});

  // LLM Roles state
  const roles = getSetting("llm_roles") || [];
  const selectedRoleIdFromSettings = getSetting("selected_llm_role_id") || null;
  const [selectedRoleId, setSelectedRoleId] = useState<string | null>(selectedRoleIdFromSettings);
  const [editingRoleId, setEditingRoleId] = useState<string | null>(null);
  const [isCreatingRole, setIsCreatingRole] = useState(false);
  const [draftRoleName, setDraftRoleName] = useState("");
  const [draftRolePrompt, setDraftRolePrompt] = useState("");
  const [draftRoleIcon, setDraftRoleIcon] = useState<string>("sparkles");
  const [draftRoleConfigId, setDraftRoleConfigId] = useState<string | null>(null);

  const providers = settings?.post_process_providers || [];
  const providerOptions = [
    ...providers.map((p: PostProcessProvider) => ({
      value: p.id,
      label: p.label,
    })),
    { value: "custom", label: t("settings.advanced.llm.globalConfig.provider.custom") || "自定义" },
  ];

  // Get selected role (for editing form)
  const selectedRole = useMemo(() => {
    if (selectedRoleId) {
      return roles.find((r: any) => r.id === selectedRoleId) || null;
    }
    return null;
  }, [selectedRoleId, roles]);

  // Get config name by ID
  const getConfigName = (configId: string | null | undefined) => {
    if (!configId) {
      // Use selected config or show default text
      if (selectedConfigId) {
        const config = llmConfigs.find((c) => c.id === selectedConfigId);
        return config?.name || t("settings.advanced.llmRoles.useDefaultConfig");
      }
      return t("settings.advanced.llmRoles.useDefaultConfig");
    }
    const config = llmConfigs.find((c) => c.id === configId);
    return config?.name || configId;
  };

  // Check if role is a default role (cannot be edited/deleted)
  const isDefaultRole = (roleId: string) => {
    return roleId === "polish" || roleId === "translate";
  };

  // Sync selected role from settings
  useEffect(() => {
    const currentSelectedRoleId = selectedRoleIdFromSettings;
    if (currentSelectedRoleId !== selectedRoleId) {
      setSelectedRoleId(currentSelectedRoleId);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedRoleIdFromSettings]);

  // Handle role selection (toggle)
  const handleSelectRole = async (roleId: string) => {
    const currentSelectedRoleId = getSetting("selected_llm_role_id") || null;
    const newSelectedRoleId = currentSelectedRoleId === roleId ? null : roleId;
    
    await updateSetting("selected_llm_role_id", newSelectedRoleId);
    
    // If selecting a role, enable LLM processing
    if (newSelectedRoleId && !llmEnabled) {
      await updateSetting("llm_enabled", true);
    }
    
    setSelectedRoleId(newSelectedRoleId);
  };

  // LLM Config handlers
  const handleSelectConfig = async (configId: string) => {
    await updateSetting("selected_llm_config_id", configId);
  };

  const handleStartCreateConfig = () => {
    setIsCreatingConfig(true);
    setEditingConfigId(null);
    setDraftConfigName("");
    const firstProviderId = providerOptions[0]?.value || "";
    setDraftConfigProviderId(firstProviderId);
    setDraftConfigApiKey("");
    setDraftConfigModel("");
    // Set initial API URL from provider's base_url
    if (firstProviderId === "custom") {
      setDraftConfigApiUrl("");
    } else {
      const provider = providers.find((p: PostProcessProvider) => p.id === firstProviderId);
      setDraftConfigApiUrl(provider?.base_url || "");
    }
    setShowConfigApiKey(false);
  };

  const handleCancelConfig = () => {
    setIsCreatingConfig(false);
    setEditingConfigId(null);
  };

  const handleSaveConfig = async () => {
    if (!draftConfigName.trim() || !draftConfigProviderId || !draftConfigApiKey.trim() || !draftConfigModel.trim()) {
      return;
    }
    
    // Require API URL for all providers
    if (!draftConfigApiUrl.trim()) {
      return;
    }

    const newConfigs = [...llmConfigs];
    const selectedProvider = providers.find((p: PostProcessProvider) => p.id === draftConfigProviderId);
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
        status: "active",
        api_url: shouldSaveApiUrl ? draftConfigApiUrl.trim() : null,
      });
      await updateSetting("llm_configs", newConfigs);
      setIsCreatingConfig(false);
      await handleSelectConfig(newId);
    } else if (editingConfigId) {
      const index = newConfigs.findIndex((c) => c.id === editingConfigId);
      if (index >= 0) {
        newConfigs[index] = {
          ...newConfigs[index],
          name: draftConfigName.trim(),
          provider_id: draftConfigProviderId,
          api_key: draftConfigApiKey.trim(),
          model: draftConfigModel.trim(),
          api_url: shouldSaveApiUrl ? draftConfigApiUrl.trim() : null,
        };
        await updateSetting("llm_configs", newConfigs);
        setEditingConfigId(null);
      }
    }
  };

  const handleDeleteConfig = async (id: string) => {
    const newConfigs = llmConfigs.filter((c) => c.id !== id);
    await updateSetting("llm_configs", newConfigs);
    if (selectedConfigId === id) {
      await updateSetting("selected_llm_config_id", newConfigs.length > 0 ? newConfigs[0].id : null);
    }
  };

  const handleEditConfig = (id: string) => {
    const config = llmConfigs.find((c) => c.id === id);
    if (config) {
      setEditingConfigId(id);
      setIsCreatingConfig(false);
      setDraftConfigName(config.name);
      setDraftConfigProviderId(config.provider_id);
      setDraftConfigApiKey(config.api_key);
      setDraftConfigModel(config.model);
      // If config has api_url, use it; otherwise use provider's base_url
      if (config.api_url) {
        setDraftConfigApiUrl(config.api_url);
      } else {
        const provider = providers.find((p: PostProcessProvider) => p.id === config.provider_id);
        setDraftConfigApiUrl(provider?.base_url || "");
      }
      setShowConfigApiKey(false);
    }
  };

  const handleCopyConfig = (id: string) => {
    const config = llmConfigs.find((c) => c.id === id);
    if (config) {
      setIsCreatingConfig(true);
      setEditingConfigId(null);
      // Add "副本" suffix to the name
      setDraftConfigName(`${config.name} 副本`);
      setDraftConfigProviderId(config.provider_id);
      setDraftConfigApiKey(config.api_key);
      setDraftConfigModel(config.model);
      // If config has api_url, use it; otherwise use provider's base_url
      if (config.api_url) {
        setDraftConfigApiUrl(config.api_url);
      } else {
        const provider = providers.find((p: PostProcessProvider) => p.id === config.provider_id);
        setDraftConfigApiUrl(provider?.base_url || "");
      }
      setShowConfigApiKey(false);
    }
  };

  const handleTestConfig = async (id: string) => {
    setTestingConfigId(id);
    setConfigStatuses((prev) => ({ ...prev, [id]: "testing" }));
    
    try {
      const result = await (commands as any).testLlmConfig(id);
      if (result.status === "ok") {
        setConfigStatuses((prev) => ({ ...prev, [id]: "success" }));
      } else {
        const errorMsg = result.error || "连接失败";
        setConfigStatuses((prev) => ({ ...prev, [id]: `error:${errorMsg}` }));
        
        // Extract curl command from error message
        const curlMatch = errorMsg.match(/Curl command:\n([\s\S]+)/);
        if (curlMatch && curlMatch[1]) {
          const curlCommand = curlMatch[1].trim();
          try {
            await writeText(curlCommand);
            // Show alert with curl command copied message
            alert(t("settings.advanced.llm.globalConfig.testFailedCurlCopied") || `测试失败，curl命令已复制到剪贴板：\n\n${curlCommand}`);
          } catch (copyError) {
            console.error("Failed to copy curl command:", copyError);
            // Show alert without copy confirmation
            alert(t("settings.advanced.llm.globalConfig.testFailed") || `测试失败：\n\n${errorMsg}`);
          }
        } else {
          // No curl command found, just show error
          alert(t("settings.advanced.llm.globalConfig.testFailed") || `测试失败：\n\n${errorMsg}`);
        }
      }
    } catch (error) {
      console.error("Test config failed:", error);
      const errorMsg = error instanceof Error ? error.message : String(error);
      setConfigStatuses((prev) => ({ ...prev, [id]: `error:${errorMsg}` }));
      
      // Extract curl command from error message
      const curlMatch = errorMsg.match(/Curl command:\n([\s\S]+)/);
      if (curlMatch && curlMatch[1]) {
        const curlCommand = curlMatch[1].trim();
        try {
          await writeText(curlCommand);
          alert(t("settings.advanced.llm.globalConfig.testFailedCurlCopied") || `测试失败，curl命令已复制到剪贴板：\n\n${curlCommand}`);
        } catch (copyError) {
          console.error("Failed to copy curl command:", copyError);
          alert(t("settings.advanced.llm.globalConfig.testFailed") || `测试失败：\n\n${errorMsg}`);
        }
      } else {
        alert(t("settings.advanced.llm.globalConfig.testFailed") || `测试失败：\n\n${errorMsg}`);
      }
    } finally {
      setTestingConfigId(null);
    }
  };

  // LLM Role handlers
  const handleStartCreateRole = () => {
    setIsCreatingRole(true);
    setEditingRoleId(null);
    setDraftRoleName("");
    setDraftRolePrompt("");
    setDraftRoleIcon("sparkles");
    setDraftRoleConfigId(null);
  };

  const handleCancelRoleEdit = () => {
    setEditingRoleId(null);
    setIsCreatingRole(false);
    setDraftRoleName("");
    setDraftRolePrompt("");
    setDraftRoleIcon("sparkles");
    setDraftRoleConfigId(null);
  };

  const handleEditRole = (id: string) => {
    const role = roles.find((r: any) => r.id === id);
    if (role) {
      setEditingRoleId(id);
      setIsCreatingRole(false);
      setDraftRoleName(role.name);
      setDraftRolePrompt(role.prompt);
      setDraftRoleIcon(role.icon || "sparkles");
      setDraftRoleConfigId(role.llm_config_id || null);
    }
  };

  const handleCopyRole = (id: string) => {
    const role = roles.find((r: any) => r.id === id);
    if (role) {
      setIsCreatingRole(true);
      setEditingRoleId(null);
      // Add "副本" suffix to the name
      setDraftRoleName(`${role.name} 副本`);
      setDraftRolePrompt(role.prompt);
      setDraftRoleIcon(role.icon || "sparkles");
      setDraftRoleConfigId(role.llm_config_id || null);
    }
  };

  const handleSaveRole = async () => {
    if (!draftRoleName.trim() || !draftRolePrompt.trim()) return;

    const newRoles = [...roles];

    if (isCreatingRole) {
      // Add new role
      const newId = `role_${Date.now()}`;
      newRoles.push({
        id: newId,
        name: draftRoleName.trim(),
        prompt: draftRolePrompt.trim(),
        icon: null, // 不再使用图标
        llm_config_id: draftRoleConfigId,
        enable_read_selection: false,
        output_method: "direct",
        provider_id: null,
        api_key: null,
        model: null,
      });
      await updateSetting("llm_roles", newRoles);
      setIsCreatingRole(false);
      setDraftRoleName("");
      setDraftRolePrompt("");
      setDraftRoleIcon("sparkles");
      setDraftRoleConfigId(null);
    } else if (editingRoleId) {
      // Update existing role
      const index = newRoles.findIndex((r: any) => r.id === editingRoleId);
      if (index >= 0) {
        const isDefault = isDefaultRole(editingRoleId);
        newRoles[index] = {
          ...newRoles[index],
          name: isDefault ? newRoles[index].name : draftRoleName.trim(), // 内置角色不能修改名称
          prompt: isDefault ? newRoles[index].prompt : draftRolePrompt.trim(), // 内置角色不能修改提示词
          llm_config_id: draftRoleConfigId, // 内置角色可以修改LLM配置
        };
        await updateSetting("llm_roles", newRoles);
        setEditingRoleId(null);
        setDraftRoleName("");
        setDraftRolePrompt("");
        setDraftRoleIcon("sparkles");
        setDraftRoleConfigId(null);
      }
    }
  };

  const handleDeleteRole = async (id: string) => {
    if (isDefaultRole(id)) return;
    
    const newRoles = roles.filter((r: any) => r.id !== id);
    await updateSetting("llm_roles", newRoles);
    if (editingRoleId === id) {
      setEditingRoleId(null);
      setDraftRoleName("");
      setDraftRolePrompt("");
    }
  };

  const renderIcon = (iconName: string | null | undefined, size: string = "w-6 h-6") => {
    const name = iconName || "sparkles";
    const IconComponent = iconMap[name] || Sparkles;
    return <IconComponent className={size} />;
  };

  // Get LLM enabled setting
  const llmEnabled = getSetting("llm_enabled") ?? true;
  const isLoading = isUpdating("llm_enabled");

  const handleToggleLlmEnabled = async (enabled: boolean) => {
    await updateSetting("llm_enabled", enabled);
  };

  return (
    <>
      {/* Enable LLM Toggle */}
      <SettingsGroup title={t("settings.advanced.llm.title")}>
        <ToggleSwitch
          checked={llmEnabled}
          onChange={handleToggleLlmEnabled}
          disabled={isLoading}
          label={t("settings.advanced.llm.enabled")}
          description={t("settings.advanced.llm.enabledDescription")}
          grouped={true}
        />
      </SettingsGroup>

      {/* Global LLM Configuration */}
      <SettingsGroup title={t("settings.advanced.llm.globalConfig.title")}>
        <div className="space-y-4">
          {/* Config List */}
          {llmConfigs.map((config) => {
            const isSelected = selectedConfigId === config.id;
            const isEditing = editingConfigId === config.id;
            const provider = providers.find((p) => p.id === config.provider_id);
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
                          title="设置"
                        >
                          <Settings className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleCopyConfig(config.id);
                          }}
                          className={isSelected ? "text-logo-primary hover:text-logo-primary" : ""}
                          title="复制并修改"
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
                          title="测试"
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
                            ? "连接成功" 
                            : testStatus.startsWith("error")
                            ? "连接失败" 
                            : testStatus === "testing"
                            ? "测试中..."
                            : testStatus === "untested"
                            ? "未测试"
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
                          title="删除"
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
                        {t("settings.advanced.llm.globalConfig.name")}
                      </label>
                      <Input
                        type="text"
                        value={draftConfigName}
                        onChange={(e) => setDraftConfigName(e.target.value)}
                        placeholder={t("settings.advanced.llm.globalConfig.name")}
                        variant="compact"
                        className="w-full"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-semibold text-text mb-2">
                        {t("settings.advanced.llm.globalConfig.provider.title")}
                      </label>
                      <ProviderSelect
                        options={providerOptions}
                        value={draftConfigProviderId}
                        onChange={(value: string) => {
                          setDraftConfigProviderId(value || "");
                          // Update API URL when provider changes
                          if (value === "custom") {
                            setDraftConfigApiUrl("");
                          } else {
                            const provider = providers.find((p: PostProcessProvider) => p.id === value);
                            setDraftConfigApiUrl(provider?.base_url || "");
                          }
                        }}
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-semibold text-text mb-2">
                        {t("settings.advanced.llm.globalConfig.apiUrl.title") || "API URL"}
                      </label>
                      {(() => {
                        const selectedProvider = providers.find((p: PostProcessProvider) => p.id === draftConfigProviderId);
                        const isReadOnly = selectedProvider && !selectedProvider.allow_base_url_edit && draftConfigProviderId !== "custom";
                        return (
                          <Input
                            type="text"
                            value={draftConfigApiUrl}
                            onChange={(e) => setDraftConfigApiUrl(e.target.value)}
                            placeholder={t("settings.advanced.llm.globalConfig.apiUrl.placeholder") || "https://api.example.com/v1"}
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
                        {t("settings.advanced.llm.globalConfig.apiKey.title")}
                      </label>
                      <div className="relative">
                        <Input
                          type={showConfigApiKey ? "text" : "password"}
                          value={draftConfigApiKey}
                          onChange={(e) => setDraftConfigApiKey(e.target.value)}
                          placeholder={t("settings.advanced.llm.globalConfig.apiKey.placeholder")}
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
                        {t("settings.advanced.llm.globalConfig.model.title")}
                      </label>
                      <Input
                        type="text"
                        value={draftConfigModel}
                        onChange={(e) => setDraftConfigModel(e.target.value)}
                        placeholder={t("settings.advanced.llm.globalConfig.model.placeholder")}
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
                          !draftConfigModel.trim() ||
                          !draftConfigApiUrl.trim()
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
                  {t("settings.advanced.llm.globalConfig.name")}
                </label>
                <Input
                  type="text"
                  value={draftConfigName}
                  onChange={(e) => setDraftConfigName(e.target.value)}
                  placeholder={t("settings.advanced.llm.globalConfig.name")}
                  variant="compact"
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-semibold text-text mb-2">
                  {t("settings.advanced.llm.globalConfig.provider.title")}
                </label>
                <ProviderSelect
                  options={providerOptions}
                  value={draftConfigProviderId}
                  onChange={(value: string) => {
                    setDraftConfigProviderId(value || "");
                    // Update API URL when provider changes
                    if (value === "custom") {
                      setDraftConfigApiUrl("");
                    } else {
                      const provider = providers.find((p: PostProcessProvider) => p.id === value);
                      setDraftConfigApiUrl(provider?.base_url || "");
                    }
                  }}
                />
              </div>
              <div>
                <label className="block text-sm font-semibold text-text mb-2">
                  {t("settings.advanced.llm.globalConfig.apiUrl.title") || "API URL"}
                </label>
                {(() => {
                  const selectedProvider = providers.find((p: PostProcessProvider) => p.id === draftConfigProviderId);
                  const isReadOnly = selectedProvider && !selectedProvider.allow_base_url_edit && draftConfigProviderId !== "custom";
                  return (
                    <Input
                      type="text"
                      value={draftConfigApiUrl}
                      onChange={(e) => setDraftConfigApiUrl(e.target.value)}
                      placeholder={t("settings.advanced.llm.globalConfig.apiUrl.placeholder") || "https://api.example.com/v1"}
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
                  {t("settings.advanced.llm.globalConfig.apiKey.title")}
                </label>
                <div className="relative">
                  <Input
                    type={showConfigApiKey ? "text" : "password"}
                    value={draftConfigApiKey}
                    onChange={(e) => setDraftConfigApiKey(e.target.value)}
                    placeholder={t("settings.advanced.llm.globalConfig.apiKey.placeholder")}
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
                  {t("settings.advanced.llm.globalConfig.model.title")}
                </label>
                <Input
                  type="text"
                  value={draftConfigModel}
                  onChange={(e) => setDraftConfigModel(e.target.value)}
                  placeholder={t("settings.advanced.llm.globalConfig.model.placeholder")}
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
                    !draftConfigModel.trim() ||
                    !draftConfigApiUrl.trim()
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
          {!isCreatingConfig && (
            <button
              onClick={handleStartCreateConfig}
              className="w-full p-4 border-2 border-dashed border-mid-gray/30 rounded-lg bg-mid-gray/5 hover:bg-mid-gray/10 hover:border-mid-gray/50 transition-all duration-200 flex items-center justify-center gap-2 text-mid-gray hover:text-text group"
            >
              <Plus className="h-5 w-5 group-hover:scale-110 transition-transform" />
              <span className="text-sm font-medium">
                {t("settings.advanced.llm.globalConfig.addConfig")}
              </span>
            </button>
          )}
        </div>
      </SettingsGroup>

      {/* LLM Roles */}
      <SettingsGroup title={t("settings.advanced.llmRoles.title")}>
        <div className="space-y-3">
          {roles.map((role: any) => {
            const isEditing = editingRoleId === role.id;
            const isDefault = isDefaultRole(role.id);
            const isSelected = selectedRoleId === role.id;
            
            return (
            <div
              key={role.id}
              className={`p-4 rounded-lg border-2 transition-all cursor-pointer ${
                isSelected
                  ? "border-logo-primary bg-logo-primary/5"
                  : "border-mid-gray/20 hover:border-mid-gray/40 bg-background"
              }`}
              onClick={() => {
                if (!isEditing) {
                  handleSelectRole(role.id);
                }
              }}
            >
              {!isEditing && (
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="flex-1 min-w-0">
                      <div className={`font-semibold text-sm flex items-center gap-2 ${
                        isSelected ? "text-logo-primary" : "text-text"
                      }`}>
                        {role.name}
                        {isDefault && (
                          <span className="text-xs text-mid-gray bg-mid-gray/10 px-1.5 py-0.5 rounded">
                            {t("settings.advanced.llmRoles.builtIn")}
                          </span>
                        )}
                        {isSelected && (
                          <Check className="h-4 w-4 text-logo-primary" />
                        )}
                      </div>
                      <div className="text-xs text-mid-gray mt-1">
                        {t("settings.advanced.llmRoles.config")}: {getConfigName(role.llm_config_id)}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleEditRole(role.id)}
                      title={t("common.edit")}
                      className={isSelected ? "text-logo-primary hover:text-logo-primary" : ""}
                    >
                      <Edit className="h-4 w-4 mr-2" />
                      {t("common.edit")}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleCopyRole(role.id)}
                      title={t("settings.advanced.llmRoles.copyAndModify")}
                      className={isSelected ? "text-logo-primary hover:text-logo-primary" : ""}
                    >
                      <Copy className="h-4 w-4 mr-2" />
                      {t("settings.advanced.llmRoles.copyAndModify")}
                    </Button>
                    {!isDefault && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleDeleteRole(role.id)}
                        className={isSelected ? "text-logo-primary hover:text-red-600" : "hover:text-red-600"}
                        title={t("common.delete")}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    )}
                  </div>
                </div>
              )}

              {/* Edit Role Form */}
              {isEditing && editingRoleId && (
                <div className="space-y-4">
                  {!isDefaultRole(editingRoleId) && (
                    <div>
                      <label className="block text-sm font-semibold text-text mb-2">
                        {t("settings.advanced.llmRoles.roleName")}
                      </label>
                      <Input
                        type="text"
                        value={draftRoleName}
                        onChange={(e) => setDraftRoleName(e.target.value)}
                        placeholder={t("settings.advanced.llmRoles.roleName")}
                        variant="compact"
                        className="w-full"
                      />
                    </div>
                  )}
                  {isDefaultRole(editingRoleId) && (
                    <div>
                      <label className="block text-sm font-semibold text-text mb-2">
                        {t("settings.advanced.llmRoles.roleName")}
                      </label>
                      <Input
                        type="text"
                        value={draftRoleName}
                        disabled
                        variant="compact"
                        className="w-full opacity-60"
                      />
                    </div>
                  )}
                  {!isDefaultRole(editingRoleId) && (
                    <div>
                      <label className="block text-sm font-semibold text-text mb-2">
                        {t("settings.advanced.llmRoles.rolePrompt")}
                      </label>
                      <Textarea
                        value={draftRolePrompt}
                        onChange={(e) => setDraftRolePrompt(e.target.value)}
                        placeholder="Use ${input} to insert the input text"
                        className="w-full min-h-[100px] bg-mid-gray/5 border-mid-gray/20"
                      />
                    </div>
                  )}
                  {isDefaultRole(editingRoleId) && (
                    <div>
                      <label className="block text-sm font-semibold text-text mb-2">
                        {t("settings.advanced.llmRoles.rolePrompt")}
                      </label>
                      <Textarea
                        value={draftRolePrompt}
                        disabled
                        className="w-full min-h-[100px] bg-mid-gray/5 border-mid-gray/20 opacity-60"
                      />
                      <p className="text-xs text-mid-gray mt-1">
                        {t("settings.advanced.llmRoles.builtInPromptCannotEdit")}
                      </p>
                    </div>
                  )}
                  <div>
                    <label className="block text-sm font-semibold text-text mb-2">
                      {t("settings.advanced.llmRoles.config")}
                    </label>
                    <Dropdown
                      selectedValue={draftRoleConfigId}
                      options={[
                        { value: "", label: t("settings.advanced.llmRoles.useDefaultConfig") },
                        ...llmConfigs.map((c) => ({ value: c.id, label: c.name })),
                      ]}
                      onSelect={(value) => setDraftRoleConfigId(value || null)}
                      className="w-full"
                    />
                  </div>
                  <div className="flex gap-2">
                    <Button
                      onClick={handleSaveRole}
                      variant="primary"
                      size="md"
                      disabled={!isDefaultRole(editingRoleId) && (!draftRoleName.trim() || !draftRolePrompt.trim())}
                    >
                      {t("common.save")}
                    </Button>
                    <Button onClick={handleCancelRoleEdit} variant="secondary" size="md">
                      {t("common.cancel")}
                    </Button>
                  </div>
                </div>
              )}
            </div>
            );
          })}

          {/* Create Role Form */}
          {isCreatingRole && (
            <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20 space-y-4">
              <div>
                <label className="block text-sm font-semibold text-text mb-2">
                  {t("settings.advanced.llmRoles.roleName")}
                </label>
                <Input
                  type="text"
                  value={draftRoleName}
                  onChange={(e) => setDraftRoleName(e.target.value)}
                  placeholder={t("settings.advanced.llmRoles.roleName")}
                  variant="compact"
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm font-semibold text-text mb-2">
                  {t("settings.advanced.llmRoles.rolePrompt")}
                </label>
                <Textarea
                  value={draftRolePrompt}
                  onChange={(e) => setDraftRolePrompt(e.target.value)}
                  placeholder="Use ${input} to insert the input text"
                  className="w-full min-h-[100px] bg-mid-gray/5 border-mid-gray/20"
                />
              </div>
              <div>
                <label className="block text-sm font-semibold text-text mb-2">
                  {t("settings.advanced.llmRoles.config")}
                </label>
                <Dropdown
                  selectedValue={draftRoleConfigId}
                  options={[
                    { value: "", label: t("settings.advanced.llmRoles.useDefaultConfig") },
                    ...llmConfigs.map((c) => ({ value: c.id, label: c.name })),
                  ]}
                  onSelect={(value) => setDraftRoleConfigId(value || null)}
                  className="w-full"
                />
              </div>
              <div className="flex gap-2">
                <Button
                  onClick={handleSaveRole}
                  variant="primary"
                  size="md"
                  disabled={!draftRoleName.trim() || !draftRolePrompt.trim()}
                >
                  {t("common.save")}
                </Button>
                <Button onClick={handleCancelRoleEdit} variant="secondary" size="md">
                  {t("common.cancel")}
                </Button>
              </div>
            </div>
          )}

          {roles.length === 0 && !isCreatingRole && (
            <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20 text-center">
              <p className="text-sm text-mid-gray">
                {t("settings.advanced.llmRoles.noRoles")}
              </p>
            </div>
          )}

          {/* Add Role Button */}
          {!isCreatingRole && (
            <button
              onClick={handleStartCreateRole}
              className="w-full p-3 border-2 border-dashed border-mid-gray/30 rounded-lg bg-mid-gray/5 hover:bg-mid-gray/10 hover:border-mid-gray/50 transition-all duration-200 flex items-center justify-center gap-2 text-mid-gray hover:text-text group"
            >
              <Plus className="h-4 w-4 group-hover:scale-110 transition-transform" />
              <span className="text-sm font-medium">
                {t("settings.advanced.llmRoles.addRole")}
              </span>
            </button>
          )}
        </div>
      </SettingsGroup>
    </>
  );
};
