import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { type } from "@tauri-apps/plugin-os";

// Conditional import for macOS-only permissions
let checkAccessibilityPermission: (() => Promise<boolean>) | null = null;
let requestAccessibilityPermission: (() => Promise<unknown>) | null = null;

// Only load macOS permissions on macOS
if (type() === "macos") {
  import("tauri-plugin-macos-permissions-api").then((mod) => {
    checkAccessibilityPermission = mod.checkAccessibilityPermission;
    requestAccessibilityPermission = mod.requestAccessibilityPermission;
  });
}

// Define permission state type
type PermissionState = "request" | "verify" | "granted";

// Define button configuration type
interface ButtonConfig {
  text: string;
  className: string;
}

const AccessibilityPermissions: React.FC = () => {
  const { t } = useTranslation();
  const [hasAccessibility, setHasAccessibility] = useState<boolean>(false);
  const [permissionState, setPermissionState] =
    useState<PermissionState>("request");
  const [osType] = useState<string>(type());

  // Check permissions without requesting
  const checkPermissions = async (): Promise<boolean> => {
    // Windows and Linux don't require accessibility permissions
    if (osType !== "macos") {
      setHasAccessibility(true);
      setPermissionState("granted");
      return true;
    }

    // macOS requires accessibility permissions
    if (!checkAccessibilityPermission) {
      console.warn("macOS permissions API not loaded");
      return false;
    }

    const hasPermissions: boolean = await checkAccessibilityPermission();
    setHasAccessibility(hasPermissions);
    setPermissionState(hasPermissions ? "granted" : "verify");
    return hasPermissions;
  };

  // Handle the unified button action based on current state
  const handleButtonClick = async (): Promise<void> => {
    // Non-macOS platforms don't need this
    if (osType !== "macos") {
      return;
    }

    if (permissionState === "request") {
      try {
        if (requestAccessibilityPermission) {
          await requestAccessibilityPermission();
        }
        // After system prompt, transition to verification state
        setPermissionState("verify");
      } catch (error) {
        console.error("Error requesting permissions:", error);
        setPermissionState("verify");
      }
    } else if (permissionState === "verify") {
      // State is "verify" - check if permission was granted
      await checkPermissions();
    }
  };

  // On app boot - check permissions
  useEffect(() => {
    const initialSetup = async (): Promise<void> => {
      // Windows and Linux automatically have "granted" state
      if (osType !== "macos") {
        setHasAccessibility(true);
        setPermissionState("granted");
        return;
      }

      // macOS requires checking permissions
      if (!checkAccessibilityPermission) {
        console.warn("macOS permissions API not loaded");
        return;
      }

      const hasPermissions: boolean = await checkAccessibilityPermission();
      setHasAccessibility(hasPermissions);
      setPermissionState(hasPermissions ? "granted" : "request");
    };

    initialSetup();
  }, [osType]);

  if (hasAccessibility) {
    return null;
  }

  // Configure button text and style based on state
  const buttonConfig: Record<PermissionState, ButtonConfig | null> = {
    request: {
      text: t("accessibility.openSettings"),
      className:
        "px-2 py-1 text-sm font-semibold bg-mid-gray/10 border  border-mid-gray/80 hover:bg-logo-primary/10 rounded cursor-pointer hover:border-logo-primary",
    },
    verify: {
      text: t("accessibility.openSettings"),
      className:
        "bg-gray-100 hover:bg-gray-200 text-gray-800 font-medium py-1 px-3 rounded text-sm flex items-center justify-center cursor-pointer",
    },
    granted: null,
  };

  const config = buttonConfig[permissionState] as ButtonConfig;

  return (
    <div className="p-4 w-full rounded-lg border border-mid-gray">
      <div className="flex justify-between items-center gap-2">
        <div className="">
          <p className="text-sm font-medium">
            {t("accessibility.permissionsDescription")}
          </p>
        </div>
        <button
          onClick={handleButtonClick}
          className={`min-h-10 ${config.className}`}
        >
          {config.text}
        </button>
      </div>
    </div>
  );
};

export default AccessibilityPermissions;
