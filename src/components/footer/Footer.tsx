import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { commands, type TranscriptionApiConfig, type LLMConfig } from "@/bindings";
import { useModelStore } from "../../stores/modelStore";
import { useSettings } from "../../hooks/useSettings";
import { getTranslatedModelName } from "@/lib/utils/modelTranslation";
import UpdateChecker from "../update-checker";

const Footer: React.FC = () => {
  const { t } = useTranslation();
  const [version, setVersion] = useState("");
  const { settings } = useSettings();
  const { currentModel, getModelInfo } = useModelStore();

  // Check if using API transcription
  const useTranscriptionApi = settings?.use_transcription_api || false;
  const selectedApiConfigId = settings?.selected_transcription_api_config_id || null;
  const apiConfigs = (settings?.transcription_api_configs || []) as TranscriptionApiConfig[];
  const providers = settings?.transcription_api_providers || [];

  // Get current model display info
  const getCurrentModelDisplay = () => {
    if (useTranscriptionApi && selectedApiConfigId) {
      const config = apiConfigs.find((c) => c.id === selectedApiConfigId);
      if (config) {
        return {
          type: "API",
          name: `${config.name} (${config.model})`,
        };
      }
    } else if (currentModel) {
      const currentModelInfo = getModelInfo(currentModel);
      const modelName = currentModelInfo
        ? getTranslatedModelName(currentModelInfo, t)
        : currentModel;
      return {
        type: "本地",
        name: modelName,
      };
    }
    return null;
  };

  const modelDisplay = getCurrentModelDisplay();

  useEffect(() => {
    setVersion("0.1.0");
  }, []);

  const itnEnabled = settings?.itn_enabled ?? false;
  const llmEnabled = settings?.llm_enabled ?? true;
  
  // Get LLM model from selected config (only if LLM is enabled)
  const selectedLlmConfigId = settings?.selected_llm_config_id || null;
  const llmConfigs = (settings?.llm_configs || []) as LLMConfig[];
  const selectedLlmConfig = selectedLlmConfigId 
    ? llmConfigs.find((c) => c.id === selectedLlmConfigId)
    : null;
  const llmModel = llmEnabled ? (selectedLlmConfig?.model || settings?.llm_model || null) : null;

  return (
    <div className="w-full border-t border-mid-gray/20 pt-3">
      <div className="flex justify-between items-center text-xs px-4 pb-3 text-text/60">
        {/* Model Info */}
        <div className="flex items-center gap-3">
          {modelDisplay && (
            <>
              <div className="flex items-center gap-1">
                <span className={`font-medium ${
                  modelDisplay.type === "API"
                    ? "text-blue-500" 
                    : "text-text/60"
                }`}>
                  {modelDisplay.type}:
                </span>
                <span>{modelDisplay.name}</span>
              </div>
              {itnEnabled && (
                <>
                  <span>•</span>
                  <span>{t("footer.itnEnabled")}</span>
                </>
              )}
              {llmModel && (
                <>
                  <span>•</span>
                  <div className="flex items-center gap-1">
                    <span className="font-medium">LLM:</span>
                    <span>{llmModel}</span>
                  </div>
                </>
              )}
            </>
          )}
        </div>

        {/* Update Status */}
        <div className="flex items-center gap-1">
          <UpdateChecker />
          <span>•</span>
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span>v{version}</span>
        </div>
      </div>
    </div>
  );
};

export default Footer;
