import { useEffect, useState, useRef } from "react";
import { Toaster } from "sonner";
import "./App.css";
import AccessibilityPermissions from "./components/AccessibilityPermissions";
import Footer from "./components/footer";
import { AccessibilityOnboarding } from "./components/onboarding";
import { Sidebar, SidebarSection, SECTIONS_CONFIG } from "./components/Sidebar";
import { HomePage } from "./components/pages/HomePage";
import { useSettings } from "./hooks/useSettings";
import { useSettingsStore } from "./stores/settingsStore";
import { commands } from "@/bindings";

type OnboardingStep = "accessibility" | "model" | "done";

const renderSettingsContent = (
  section: SidebarSection,
  onNavigateToHistory?: () => void,
) => {
  const ActiveComponent =
    SECTIONS_CONFIG[section]?.component || SECTIONS_CONFIG.general.component;
  if (section === "home" && onNavigateToHistory) {
    return <HomePage onNavigateToHistory={onNavigateToHistory} />;
  }
  return <ActiveComponent />;
};

function App() {
  const [onboardingStep, setOnboardingStep] = useState<OnboardingStep | null>(
    null,
  );
  const [currentSection, setCurrentSection] = useState<SidebarSection>("home");
  const [initError, setInitError] = useState<string | null>(null);
  
  const { settings, updateSetting } = useSettings();
  const refreshAudioDevices = useSettingsStore(
    (state) => state.refreshAudioDevices,
  );
  const refreshOutputDevices = useSettingsStore(
    (state) => state.refreshOutputDevices,
  );
  const hasCompletedPostOnboardingInit = useRef(false);

  useEffect(() => {
    // Add timeout to prevent infinite loading
    const timeout = setTimeout(() => {
      if (onboardingStep === null) {
        console.warn("Onboarding check timed out, defaulting to accessibility step");
        setOnboardingStep("accessibility");
      }
    }, 5000); // 5 second timeout
    
    checkOnboardingStatus().catch((error) => {
      console.error("Failed to check onboarding status:", error);
      setInitError(error instanceof Error ? error.message : String(error));
      setOnboardingStep("accessibility");
    });
    
    return () => clearTimeout(timeout);
  }, []);

  // Initialize Enigo and refresh audio devices when main app loads
  useEffect(() => {
    if (onboardingStep === "done" && !hasCompletedPostOnboardingInit.current) {
      hasCompletedPostOnboardingInit.current = true;
      commands.initializeEnigo().catch((e) => {
        console.warn("Failed to initialize Enigo:", e);
      });
      refreshAudioDevices();
      refreshOutputDevices();
    }
  }, [onboardingStep, refreshAudioDevices, refreshOutputDevices]);

  // Handle keyboard shortcuts for debug mode toggle
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Don't intercept keyboard events when user is typing in input fields
      const target = event.target as HTMLElement;
      const isInputField =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable ||
          target.closest("input, textarea, [contenteditable]"));

      // Always allow input field events to pass through completely
      // This includes all keyboard shortcuts like Ctrl+V / Cmd+V, Ctrl+C / Cmd+C, etc.
      if (isInputField) {
        return; // Don't intercept, let default behavior work
      }

      // Check for Ctrl+Shift+D (Windows/Linux) or Cmd+Shift+D (macOS)
      const isDebugShortcut =
        event.shiftKey &&
        event.key.toLowerCase() === "d" &&
        (event.ctrlKey || event.metaKey);

      if (isDebugShortcut) {
        event.preventDefault();
        event.stopPropagation();
        const currentDebugMode = settings?.debug_mode ?? false;
        updateSetting("debug_mode", !currentDebugMode);
      }
      // Allow all other keyboard events to work normally
    };

    // Add event listener when component mounts - use bubble phase, not capture
    // This ensures input fields handle events first
    document.addEventListener("keydown", handleKeyDown, false);

    // Cleanup event listener when component unmounts
    return () => {
      document.removeEventListener("keydown", handleKeyDown, false);
    };
  }, [settings?.debug_mode, updateSetting]);

  const checkOnboardingStatus = async () => {
    try {
      console.log("Checking onboarding status...");
      console.log("Tauri commands available:", typeof commands !== "undefined");
      
      // Check if Tauri is available
      if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__) {
        console.log("Tauri is available");
      } else {
        console.warn("Tauri may not be fully initialized yet");
      }
      
      // Check if they have any models available
      const result = await commands.hasAnyModelsAvailable();
      console.log("hasAnyModelsAvailable result:", result);
      if (result.status === "ok") {
        // If they have models/downloads, they're done. Otherwise start permissions step.
        setOnboardingStep(result.data ? "done" : "accessibility");
      } else {
        console.warn("hasAnyModelsAvailable returned error:", result.error);
        setOnboardingStep("accessibility");
      }
    } catch (error) {
      console.error("Failed to check onboarding status:", error);
      console.error("Error details:", {
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
        name: error instanceof Error ? error.name : undefined,
      });
      // Always set a step to avoid blank page - default to accessibility
      // This ensures the page doesn't stay blank if Tauri backend is not available
      setOnboardingStep("accessibility");
    }
  };

  const handleAccessibilityComplete = () => {
    // Skip model selection page, go directly to main app
    setOnboardingStep("done");
  };

  const handleModelSelected = () => {
    // Transition to main app - user has started a download
    setOnboardingStep("done");
  };

  // Show error state if initialization failed
  if (initError) {
    return (
      <div
        className="h-screen w-screen flex items-center justify-center bg-background"
        style={{ backgroundColor: "#fafafa", padding: "20px" }}
      >
        <div className="flex flex-col items-center gap-4 max-w-md">
          <h1 style={{ color: "#dc2626", fontSize: "18px", fontWeight: "bold" }}>
            初始化错误
          </h1>
          <p style={{ color: "#6b7280", textAlign: "center" }}>
            {initError}
          </p>
          <button
            onClick={() => window.location.reload()}
            style={{
              padding: "8px 16px",
              backgroundColor: "#8b5cf6",
              color: "white",
              border: "none",
              borderRadius: "6px",
              cursor: "pointer",
            }}
          >
            重新加载
          </button>
        </div>
      </div>
    );
  }

  // Still checking onboarding status - show loading state
  if (onboardingStep === null) {
    return (
      <div
        className="h-screen w-screen flex items-center justify-center bg-background"
        style={{ backgroundColor: "#fafafa" }}
      >
        <div className="flex flex-col items-center gap-4">
          <div
            className="w-8 h-8 border-4 border-logo-primary border-t-transparent rounded-full animate-spin"
            style={{ borderColor: "#8b5cf6", borderTopColor: "transparent" }}
          />
          <p className="text-text/70 text-sm" style={{ color: "#6b7280" }}>
            正在加载...
          </p>
          <p
            className="text-xs text-mid-gray"
            style={{ color: "#9ca3af", marginTop: "8px" }}
          >
            如果长时间停留在此页面，请检查浏览器控制台的错误信息
          </p>
        </div>
      </div>
    );
  }

  if (onboardingStep === "accessibility") {
    return <AccessibilityOnboarding onComplete={handleAccessibilityComplete} />;
  }

  // Model selection page removed - users can select models from the main app
  // if (onboardingStep === "model") {
  //   return <Onboarding onModelSelected={handleModelSelected} />;
  // }

  return (
    <div className="h-screen flex flex-col select-none cursor-default">
      <Toaster
        theme="system"
        toastOptions={{
          unstyled: true,
          classNames: {
            toast:
              "bg-background border border-mid-gray/20 rounded-lg shadow-lg px-4 py-3 flex items-center gap-3 text-sm",
            title: "font-medium",
            description: "text-mid-gray",
          },
        }}
      />
      {/* Main content area that takes remaining space */}
      <div className="flex-1 flex overflow-hidden">
        <Sidebar
          activeSection={currentSection}
          onSectionChange={setCurrentSection}
        />
        {/* Scrollable content area */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <div className="flex flex-col p-4">
              {currentSection === "home" ||
              currentSection === "dictionary" ||
              currentSection === "models" ||
              currentSection === "contact" ? (
                renderSettingsContent(currentSection, () =>
                  setCurrentSection("history"),
                )
              ) : (
                <div className="flex flex-col items-center gap-4">
                  <AccessibilityPermissions />
                  {renderSettingsContent(currentSection)}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
      {/* Fixed footer at bottom */}
      <Footer />
    </div>
  );
}

export default App;
