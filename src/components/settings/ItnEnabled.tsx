import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface ItnEnabledProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const ItnEnabled: React.FC<ItnEnabledProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const itnEnabled = getSetting("itn_enabled") ?? true;


    return (
      <ToggleSwitch
        checked={itnEnabled}
        onChange={(enabled) => updateSetting("itn_enabled", enabled)}
        isUpdating={isUpdating("itn_enabled")}
        label={t("settings.advanced.itnEnabled.label")}
        description={t("settings.advanced.itnEnabled.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);

ItnEnabled.displayName = "ItnEnabled";
