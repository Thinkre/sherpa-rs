import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";
import { commands } from "@/bindings";
import { Loader2 } from "lucide-react";

interface PunctuationEnabledProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const PunctuationEnabled: React.FC<PunctuationEnabledProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const [isLoading, setIsLoading] = useState(false);
    const [status, setStatus] = useState<"idle" | "downloading" | "loading" | "error">("idle");
    const [errorMessage, setErrorMessage] = useState<string | null>(null);

    const punctuationEnabled = getSetting("punctuation_enabled") ?? false;

    // When enabled, check and download/load model if needed
    useEffect(() => {
      if (punctuationEnabled && !isLoading) {
        setIsLoading(true);
        setStatus("loading");
        setErrorMessage(null);

        commands
          .ensurePunctuationModel()
          .then(() => {
            setStatus("idle");
            setIsLoading(false);
          })
          .catch((err) => {
            console.error("Failed to ensure punctuation model:", err);
            setStatus("error");
            setErrorMessage(err?.toString() || "Unknown error");
            setIsLoading(false);
          });
      } else if (!punctuationEnabled) {
        setStatus("idle");
        setErrorMessage(null);
        setIsLoading(false);
      }
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [punctuationEnabled]);

    const handleToggle = async (enabled: boolean) => {
      await updateSetting("punctuation_enabled", enabled);
    };

    return (
      <div className="space-y-2">
        <ToggleSwitch
          checked={punctuationEnabled}
          onChange={handleToggle}
          isUpdating={isUpdating("punctuation_enabled") || isLoading}
          label={t("settings.advanced.punctuationEnabled.label")}
          description={t("settings.advanced.punctuationEnabled.description")}
          descriptionMode={descriptionMode}
          grouped={grouped}
        />
        {punctuationEnabled && isLoading && (
          <div className="flex items-center gap-2 text-sm text-muted-foreground ml-1">
            <Loader2 className="h-4 w-4 animate-spin" />
            <span>
              {status === "downloading"
                ? t("settings.advanced.punctuationEnabled.downloading")
                : t("settings.advanced.punctuationEnabled.loading")}
            </span>
          </div>
        )}
        {punctuationEnabled && status === "error" && errorMessage && (
          <div className="text-sm text-destructive ml-1">
            {t("settings.advanced.punctuationEnabled.error")}: {errorMessage}
          </div>
        )}
      </div>
    );
  },
);

PunctuationEnabled.displayName = "PunctuationEnabled";
