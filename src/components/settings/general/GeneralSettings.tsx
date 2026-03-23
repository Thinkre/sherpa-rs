import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { MicrophoneSelector } from "../MicrophoneSelector";
import { LanguageSelector } from "../LanguageSelector";
import { HandyShortcut } from "../HandyShortcut";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { SettingContainer } from "../../ui/SettingContainer";
import { OutputDeviceSelector } from "../OutputDeviceSelector";
import { PushToTalk } from "../PushToTalk";
import { AudioFeedback } from "../AudioFeedback";
import { useSettings } from "../../../hooks/useSettings";
import { useModelStore } from "../../../stores/modelStore";
import { VolumeSlider } from "../VolumeSlider";
import { ShowOverlay } from "../ShowOverlay";
import { TranslateToEnglish } from "../TranslateToEnglish";
import { ModelUnloadTimeoutSetting } from "../ModelUnloadTimeout";
import { StartHidden } from "../StartHidden";
import { AutostartToggle } from "../AutostartToggle";
import { PasteMethodSetting } from "../PasteMethod";
import { ClipboardHandlingSetting } from "../ClipboardHandling";
import { ItnEnabled } from "../ItnEnabled";
import { PunctuationEnabled } from "../PunctuationEnabled";
import { AppLanguageSelector } from "../AppLanguageSelector";

export const GeneralSettings: React.FC = () => {
  const { t } = useTranslation();
  const { audioFeedbackEnabled } = useSettings();
  const { currentModel, getModelInfo } = useModelStore();
  const currentModelInfo = getModelInfo(currentModel);
  const showLanguageSelector = currentModelInfo?.engine_type === "Whisper";
  const showTranslateToEnglish =
    currentModelInfo?.engine_type === "Whisper" && currentModel !== "turbo";
  const [version, setVersion] = useState("");


  useEffect(() => {
    setVersion("0.1.0");
  }, []);

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.general.title")}>
        <HandyShortcut shortcutId="transcribe" grouped={true} />
        {showLanguageSelector && (
          <LanguageSelector descriptionMode="tooltip" grouped={true} />
        )}
        <PushToTalk descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>
      <SettingsGroup title={t("settings.sound.title")}>
        <MicrophoneSelector descriptionMode="tooltip" grouped={true} />
        <AudioFeedback descriptionMode="tooltip" grouped={true} />
        <OutputDeviceSelector
          descriptionMode="tooltip"
          grouped={true}
          disabled={!audioFeedbackEnabled}
        />
        <VolumeSlider disabled={!audioFeedbackEnabled} />
      </SettingsGroup>
      <SettingsGroup title={t("settings.advanced.title")}>
        <StartHidden descriptionMode="tooltip" grouped={true} />
        <AutostartToggle descriptionMode="tooltip" grouped={true} />
        <ShowOverlay descriptionMode="tooltip" grouped={true} />
        <PasteMethodSetting descriptionMode="tooltip" grouped={true} />
        <ClipboardHandlingSetting descriptionMode="tooltip" grouped={true} />
        {showTranslateToEnglish && (
          <TranslateToEnglish descriptionMode="tooltip" grouped={true} />
        )}
        <ModelUnloadTimeoutSetting descriptionMode="tooltip" grouped={true} />
        <ItnEnabled descriptionMode="tooltip" grouped={true} />
        <PunctuationEnabled descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>
      <SettingsGroup title={t("settings.about.title")}>
        <AppLanguageSelector descriptionMode="tooltip" grouped={true} />
        <SettingContainer
          title={t("settings.about.version.title")}
          description={t("settings.about.version.description")}
          grouped={true}
        >
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span className="text-sm font-mono">v{version}</span>
        </SettingContainer>
      </SettingsGroup>
    </div>
  );
};
