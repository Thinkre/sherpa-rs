import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { Dropdown } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";
import {
  SUPPORTED_LANGUAGES,
  type SupportedLanguageCode,
  syncLanguageFromSettings,
} from "../../i18n";
import { useSettings } from "@/hooks/useSettings";

interface AppLanguageSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const AppLanguageSelector: React.FC<AppLanguageSelectorProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t, i18n } = useTranslation();
  const { settings, updateSetting } = useSettings();
  const [forceUpdate, setForceUpdate] = useState(0);

  // Get the normalized language code from settings or i18n
  const getCurrentLanguage = (): SupportedLanguageCode => {
    const settingLang = settings?.app_language;
    if (settingLang) {
      const normalized = settingLang
        .toLowerCase()
        .split("-")[0]
        .split("_")[0]
        .trim();
      const supported = SUPPORTED_LANGUAGES.find(
        (lang) => lang.code.toLowerCase() === normalized,
      );
      if (supported) {
        console.log(
          `Current language from settings: ${settingLang} -> ${supported.code}`,
        );
        return supported.code;
      }
    }
    const i18nLang = i18n.language
      .toLowerCase()
      .split("-")[0]
      .split("_")[0]
      .trim();
    const supported = SUPPORTED_LANGUAGES.find(
      (lang) => lang.code.toLowerCase() === i18nLang,
    );
    if (supported) {
      console.log(
        `Current language from i18n: ${i18n.language} -> ${supported.code}`,
      );
      return supported.code;
    }
    return "en";
  };

  const currentLanguage = getCurrentLanguage();

  const languageOptions = SUPPORTED_LANGUAGES.map((lang) => ({
    value: lang.code,
    label: `${lang.nativeName} (${lang.name})`,
  }));

  // Listen for settings changes and language changes to sync language
  useEffect(() => {
    const setupListeners = async () => {
      // Listen for Tauri settings-changed event
      const unlistenSettings = await listen("settings-changed", (event) => {
        const payload = event.payload as { setting?: string; value?: unknown };
        if (payload.setting === "app_language") {
          console.log(
            "Settings changed event received for app_language:",
            payload.value,
          );
          syncLanguageFromSettings().then(() => {
            setForceUpdate((prev) => prev + 1);
          });
        }
      });

      // Listen for custom languagechanged event
      const handleLanguageChanged = () => {
        console.log("Language changed event received");
        setForceUpdate((prev) => prev + 1);
      };
      window.addEventListener("languagechanged", handleLanguageChanged);

      // Listen for i18n language changes
      i18n.on("languageChanged", (lng) => {
        console.log("i18n language changed to:", lng);
        setForceUpdate((prev) => prev + 1);
      });

      return () => {
        unlistenSettings();
        window.removeEventListener("languagechanged", handleLanguageChanged);
        i18n.off("languageChanged", handleLanguageChanged);
      };
    };
    let cleanupFn: (() => void) | null = null;
    setupListeners().then((fn) => {
      cleanupFn = fn;
    });
    return () => {
      if (cleanupFn) {
        cleanupFn();
      }
    };
  }, [i18n]);

  const handleLanguageChange = async (langCode: string) => {
    try {
      console.log(`Language change requested: ${langCode}`);
      // Normalize the language code
      const normalized = langCode
        .toLowerCase()
        .split("-")[0]
        .split("_")[0]
        .trim();
      const supportedLang = SUPPORTED_LANGUAGES.find(
        (lang) => lang.code.toLowerCase() === normalized,
      );

      if (!supportedLang) {
        console.error(`Unsupported language code: ${langCode}`);
        return;
      }

      const languageToUse = supportedLang.code;
      console.log(`Changing language to: ${languageToUse}`);

      // Update setting first
      await updateSetting("app_language", languageToUse);

      // Then change language immediately for better UX
      await i18n.changeLanguage(languageToUse);

      // Force a re-render by emitting a language changed event
      window.dispatchEvent(new Event("languagechanged"));

      // Also trigger a state update
      setForceUpdate((prev) => prev + 1);
    } catch (error) {
      console.error("Failed to change language:", error);
      // Try to sync from settings as fallback
      await syncLanguageFromSettings();
    }
  };

  return (
    <SettingContainer
      title={t("appLanguage.title")}
      description={t("appLanguage.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <Dropdown
        options={languageOptions}
        selectedValue={currentLanguage}
        onSelect={handleLanguageChange}
      />
    </SettingContainer>
  );
};
