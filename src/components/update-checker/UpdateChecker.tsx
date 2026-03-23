import React, { useState, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { listen } from "@tauri-apps/api/event";
import { platform } from "@tauri-apps/plugin-os";
import { ProgressBar } from "../shared";
import { useSettings } from "../../hooks/useSettings";

interface UpdateCheckerProps {
  className?: string;
}

const UpdateChecker: React.FC<UpdateCheckerProps> = ({ className = "" }) => {
  const { t } = useTranslation();
  // Update checking state
  const [isChecking, setIsChecking] = useState(false);
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [pendingUpdate, setPendingUpdate] = useState<Update | null>(null);
  const [showUpdateDialog, setShowUpdateDialog] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [showUpToDate, setShowUpToDate] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);
  const [restartRequired, setRestartRequired] = useState(false);

  const { settings, isLoading } = useSettings();
  const settingsLoaded = !isLoading && settings !== null;
  const updateChecksEnabled = settings?.update_checks_enabled ?? false;

  const upToDateTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const isManualCheckRef = useRef(false);
  const downloadedBytesRef = useRef(0);
  const contentLengthRef = useRef(0);

  useEffect(() => {
    // Wait for settings to load before doing anything
    if (!settingsLoaded) return;

    if (!updateChecksEnabled) {
      if (upToDateTimeoutRef.current) {
        clearTimeout(upToDateTimeoutRef.current);
      }
      setIsChecking(false);
      setUpdateAvailable(false);
      setPendingUpdate(null);
      setShowUpdateDialog(false);
      setShowUpToDate(false);
      return;
    }

    checkForUpdates();

    // Listen for update check events
    const updateUnlisten = listen("check-for-updates", () => {
      handleManualUpdateCheck();
    });

    return () => {
      if (upToDateTimeoutRef.current) {
        clearTimeout(upToDateTimeoutRef.current);
      }
      updateUnlisten.then((fn) => fn());
    };
  }, [settingsLoaded, updateChecksEnabled]);

  // Update checking functions
  const checkForUpdates = async () => {
    if (!updateChecksEnabled || isChecking) return;

    try {
      setIsChecking(true);
      const update = await check();

      if (update) {
        setPendingUpdate(update);
        setUpdateAvailable(true);
        setShowUpToDate(false);
      } else {
        setPendingUpdate(null);
        setUpdateAvailable(false);

        if (isManualCheckRef.current) {
          setShowUpToDate(true);
          if (upToDateTimeoutRef.current) {
            clearTimeout(upToDateTimeoutRef.current);
          }
          upToDateTimeoutRef.current = setTimeout(() => {
            setShowUpToDate(false);
          }, 3000);
        }
      }
    } catch (error) {
      console.error("Failed to check for updates:", error);
    } finally {
      setIsChecking(false);
      isManualCheckRef.current = false;
    }
  };

  const handleManualUpdateCheck = () => {
    if (!updateChecksEnabled) return;
    isManualCheckRef.current = true;
    checkForUpdates();
  };

  const installUpdate = async (updateToInstall?: Update | null) => {
    // 弹窗内点击「更新」时已传入 pendingUpdate，不依赖「更新检查」开关
    if (!updateToInstall && !updateChecksEnabled) return;
    setInstallError(null);
    setShowUpdateDialog(false);
    try {
      setIsInstalling(true);
      setDownloadProgress(0);
      downloadedBytesRef.current = 0;
      contentLengthRef.current = 0;
      const update = updateToInstall ?? (await check());

      if (!update) {
        setUpdateAvailable(false);
        setPendingUpdate(null);
        return;
      }

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            downloadedBytesRef.current = 0;
            contentLengthRef.current = event.data.contentLength ?? 0;
            break;
          case "Progress":
            downloadedBytesRef.current += event.data.chunkLength;
            const progress =
              contentLengthRef.current > 0
                ? Math.round(
                    (downloadedBytesRef.current / contentLengthRef.current) *
                      100,
                  )
                : 0;
            setDownloadProgress(Math.min(progress, 100));
            break;
        }
      });
      // macOS 上立即 relaunch 可能导致闪退，先短暂延迟再尝试；若失败则提示用户手动重启
      const isMac = (await platform()) === "macos";
      if (isMac) {
        await new Promise((r) => setTimeout(r, 2000));
      }
      try {
        await relaunch();
      } catch {
        setRestartRequired(true);
      }
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error);
      console.error("Failed to install update:", error);
      setInstallError(message);
      setShowUpdateDialog(true);
    } finally {
      setIsInstalling(false);
      setDownloadProgress(0);
      downloadedBytesRef.current = 0;
      contentLengthRef.current = 0;
    }
  };

  // Update status functions
  const getUpdateStatusText = () => {
    if (!updateChecksEnabled) {
      return t("footer.updateCheckingDisabled");
    }
    if (isInstalling) {
      return downloadProgress > 0 && downloadProgress < 100
        ? t("footer.downloading", {
            progress: downloadProgress.toString().padStart(3),
          })
        : downloadProgress === 100
          ? t("footer.installing")
          : t("footer.preparing");
    }
    if (isChecking) return t("footer.checkingUpdates");
    if (showUpToDate) return t("footer.upToDate");
    if (updateAvailable) return t("footer.updateAvailableShort");
    return t("footer.checkForUpdates");
  };

  const getUpdateStatusAction = () => {
    if (!updateChecksEnabled) return undefined;
    if (updateAvailable && !isInstalling) return () => setShowUpdateDialog(true);
    if (!isChecking && !isInstalling && !updateAvailable)
      return handleManualUpdateCheck;
    return undefined;
  };

  const isUpdateDisabled = !updateChecksEnabled || isChecking || isInstalling;
  const isUpdateClickable =
    !isUpdateDisabled && (updateAvailable || (!isChecking && !showUpToDate));

  return (
    <div className={`flex items-center gap-3 ${className}`}>
      {isUpdateClickable ? (
        <button
          onClick={getUpdateStatusAction()}
          disabled={isUpdateDisabled}
          className={`transition-colors disabled:opacity-50 tabular-nums ${
            updateAvailable
              ? "text-logo-primary hover:text-logo-primary/80 font-medium"
              : "text-text/60 hover:text-text/80"
          }`}
        >
          {getUpdateStatusText()}
        </button>
      ) : (
        <span className="text-text/60 tabular-nums">
          {getUpdateStatusText()}
        </span>
      )}

      {isInstalling && downloadProgress > 0 && downloadProgress < 100 && (
        <ProgressBar
          progress={[
            {
              id: "update",
              percentage: downloadProgress,
            },
          ]}
          size="large"
        />
      )}

      {/* 更新弹窗：不透明背景 + 高对比度文字 */}
      {(showUpdateDialog && pendingUpdate) || restartRequired ? (
        <div
          className="fixed inset-0 bg-black/60 flex items-center justify-center z-[100]"
          onClick={() => {
            setShowUpdateDialog(false);
            setInstallError(null);
            setRestartRequired(false);
          }}
          role="dialog"
          aria-modal="true"
          aria-labelledby="update-dialog-title"
        >
          <div
            className="w-full max-w-md mx-4 max-h-[85vh] flex flex-col rounded-xl shadow-2xl p-6 border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900"
            style={{ background: 'var(--color-background)' }}
            onClick={(e) => e.stopPropagation()}
          >
            <h2
              id="update-dialog-title"
              className="text-lg font-semibold mb-3 text-gray-900 dark:text-gray-100"
              style={{ color: 'var(--color-text)' }}
            >
              {restartRequired
                ? t("footer.updateAvailable", { version: pendingUpdate?.version ?? "" })
                : t("footer.updateAvailable", { version: pendingUpdate!.version })}
            </h2>
            {installError && (
              <p className="text-red-600 dark:text-red-400 text-sm mb-3">
                {installError}
              </p>
            )}
            {restartRequired && (
              <p className="text-green-600 dark:text-green-400 text-sm mb-6">
                {t("footer.updateInstalledRestartManually")}
              </p>
            )}
            {!restartRequired && (
              <div
                className="flex-1 overflow-y-auto text-sm whitespace-pre-wrap mb-6 min-h-[4rem] text-gray-700 dark:text-gray-300"
                style={{ color: 'var(--color-text)' }}
              >
                {pendingUpdate?.body?.trim() || t("footer.noReleaseNotes")}
              </div>
            )}
            <div className="flex justify-end gap-3">
              <button
                type="button"
                onClick={() => {
                  setShowUpdateDialog(false);
                  setInstallError(null);
                  setRestartRequired(false);
                }}
                className="px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-gray-100 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
              >
                {restartRequired ? t("common.close") : t("common.cancel")}
              </button>
              {!restartRequired && (
                <button
                  type="button"
                  onClick={() => pendingUpdate && installUpdate(pendingUpdate)}
                  className="px-4 py-2 rounded-lg bg-logo-primary text-white hover:bg-logo-primary/90 transition-colors"
                >
                  {t("common.update")}
                </button>
              )}
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
};

export default UpdateChecker;
