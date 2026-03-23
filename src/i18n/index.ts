import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { locale } from "@tauri-apps/plugin-os";
import { LANGUAGE_METADATA } from "./languages";
import { commands } from "@/bindings";

// Auto-discover translation files using Vite's glob import
const localeModules = import.meta.glob<{ default: Record<string, unknown> }>(
  "./locales/*/translation.json",
  { eager: true },
);

// Build resources from discovered locale files
const resources: Record<string, { translation: Record<string, unknown> }> = {};
for (const [path, module] of Object.entries(localeModules)) {
  const langCode = path.match(/\.\/locales\/(.+)\/translation\.json/)?.[1];
  if (langCode) {
    resources[langCode] = { translation: module.default };
  }
}

// Build supported languages list from discovered locales + metadata
export const SUPPORTED_LANGUAGES = Object.keys(resources)
  .map((code) => {
    const meta = LANGUAGE_METADATA[code];
    if (!meta) {
      console.warn(`Missing metadata for locale "${code}" in languages.ts`);
      return { code, name: code, nativeName: code, priority: undefined };
    }
    return {
      code,
      name: meta.name,
      nativeName: meta.nativeName,
      priority: meta.priority,
    };
  })
  .sort((a, b) => {
    // Sort by priority first (lower = higher), then alphabetically
    if (a.priority !== undefined && b.priority !== undefined) {
      return a.priority - b.priority;
    }
    if (a.priority !== undefined) return -1;
    if (b.priority !== undefined) return 1;
    return a.name.localeCompare(b.name);
  });

export type SupportedLanguageCode = string;

// Check if a language code is supported
const getSupportedLanguage = (
  langCode: string | null | undefined,
): SupportedLanguageCode | null => {
  if (!langCode) return null;
  // Normalize language code: extract base code and convert to lowercase
  const normalizedCode = langCode
    .split("-")[0]
    .split("_")[0]
    .toLowerCase()
    .trim();
  // Find case-insensitive match
  const supported = SUPPORTED_LANGUAGES.find(
    (lang) => lang.code.toLowerCase() === normalizedCode,
  );
  if (supported) {
    console.log(`Language matched: ${langCode} -> ${supported.code}`);
    return supported.code;
  }
  console.warn(
    `Language not supported: ${langCode} (normalized: ${normalizedCode})`,
  );
  return null;
};

// Initialize i18n with English as default
// Language will be synced from settings after init
i18n.use(initReactI18next).init({
  resources,
  lng: "en",
  fallbackLng: "en",
  interpolation: {
    escapeValue: false, // React already escapes values
  },
  react: {
    useSuspense: false, // Disable suspense for SSR compatibility
  },
});

// Sync language from app settings
export const syncLanguageFromSettings = async () => {
  try {
    const result = await commands.getAppSettings();
    if (result.status === "ok" && result.data.app_language) {
      const supported = getSupportedLanguage(result.data.app_language);
      if (supported) {
        // Always change language if we have a valid supported language
        // This ensures the UI updates even if the language is the same
        console.log(
          `Syncing language from settings: ${result.data.app_language} -> ${supported}`,
        );
        await i18n.changeLanguage(supported);
        // Force a re-render by emitting a language changed event
        window.dispatchEvent(new Event("languagechanged"));
      } else {
        console.warn(`Unsupported language code: ${result.data.app_language}`);
      }
    } else {
      // Fall back to system locale detection if no saved preference
      const systemLocale = await locale();
      const supported = getSupportedLanguage(systemLocale);
      if (supported && supported !== i18n.language) {
        console.log(
          `Syncing language from system: ${systemLocale} -> ${supported}`,
        );
        await i18n.changeLanguage(supported);
        window.dispatchEvent(new Event("languagechanged"));
      }
    }
  } catch (e) {
    console.warn("Failed to sync language from settings:", e);
  }
};

// Run language sync on init
syncLanguageFromSettings();

export default i18n;
