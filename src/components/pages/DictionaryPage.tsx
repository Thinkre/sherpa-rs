import React, { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Search, Plus, X, AlertCircle, CheckCircle2, Upload, Eye, Trash2, Check, Edit2, Save } from "lucide-react";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { useSettings } from "../../hooks/useSettings";
import { open } from "@tauri-apps/plugin-dialog";
import { commands, type HotwordFile } from "@/bindings";

export const DictionaryPage: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const customWords = getSetting("custom_words") || [];
  const [inputValue, setInputValue] = useState("");
  const [hotwordFiles, setHotwordFiles] = useState<HotwordFile[]>([]);
  const [viewingFile, setViewingFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string>("");
  const [editingContent, setEditingContent] = useState<string>("");
  const [isEditing, setIsEditing] = useState(false);
  const [loading, setLoading] = useState(false);

  // Load hotword files on mount
  useEffect(() => {
    loadHotwordFiles();
  }, []);

  const loadHotwordFiles = async () => {
    try {
      const result = await commands.getHotwordFiles();
      console.log("Loaded hotword files result:", result);
      if (result.status === "ok") {
        setHotwordFiles(result.data);
      } else {
        console.error("Failed to load hotword files:", result.error);
      }
    } catch (error) {
      console.error("Failed to load hotword files:", error);
    }
  };

  // Check if input word already exists
  const trimmedInput = inputValue.trim();
  const sanitizedInput = trimmedInput.replace(/[<>"'&]/g, "");
  const wordExists = sanitizedInput && customWords.includes(sanitizedInput);
  const canAdd =
    sanitizedInput &&
    !sanitizedInput.includes(" ") &&
    sanitizedInput.length <= 50 &&
    !wordExists;

  // Filter words based on input (for search)
  const filteredWords = useMemo(() => {
    if (!sanitizedInput) {
      return customWords;
    }
    const query = sanitizedInput.toLowerCase();
    return customWords.filter((word) => word.toLowerCase().includes(query));
  }, [customWords, sanitizedInput]);

  const handleAddWord = async () => {
    if (canAdd) {
      // Add to UserDefault.txt
      try {
        const result = await commands.addWordToUserDefault(sanitizedInput);
        if (result.status === "ok") {
          // Also add to custom_words for backward compatibility
          updateSetting("custom_words", [...customWords, sanitizedInput]);
          setInputValue("");
          // Reload hotword files to refresh UserDefault.txt
          await loadHotwordFiles();
        } else {
          console.error("Failed to add word to UserDefault.txt:", result.error);
          // Fallback: just add to custom_words
          updateSetting("custom_words", [...customWords, sanitizedInput]);
          setInputValue("");
        }
      } catch (error) {
        console.error("Failed to add word to UserDefault.txt:", error);
        // Fallback: just add to custom_words
        updateSetting("custom_words", [...customWords, sanitizedInput]);
        setInputValue("");
      }
    }
  };

  const handleRemoveWord = (wordToRemove: string) => {
    updateSetting(
      "custom_words",
      customWords.filter((word) => word !== wordToRemove),
    );
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && canAdd) {
      e.preventDefault();
      handleAddWord();
    }
  };

  const handleUploadFile = async () => {
    console.log("handleUploadFile called");
    try {
      console.log("Opening file dialog...");
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Text Files",
            extensions: ["txt"],
          },
        ],
        title: "选择热词文件",
      });

      console.log("File dialog result:", selected);

      if (!selected) {
        console.log("User cancelled file selection");
        return;
      }

      setLoading(true);
      try {
        console.log("Uploading file:", selected);
        const result = await commands.uploadHotwordFile(selected as string);
        console.log("Upload file result:", result);
        if (result.status === "ok") {
          console.log("Uploaded file successfully:", result.data);
          await loadHotwordFiles();
          // Show success message
          alert(`文件上传成功: ${result.data.name}`);
        } else {
          console.error("Failed to upload file:", result.error);
          alert(`上传失败: ${result.error}`);
        }
      } catch (error) {
        console.error("Exception during upload:", error);
        alert(`上传失败: ${error instanceof Error ? error.message : String(error)}`);
      } finally {
        setLoading(false);
      }
    } catch (error) {
      console.error("Failed to open file dialog or upload:", error);
      alert(`操作失败: ${error instanceof Error ? error.message : String(error)}`);
      setLoading(false);
    }
  };

  const handleDeleteFile = async (filePath: string) => {
    if (!confirm("确定要删除这个文件吗？")) {
      return;
    }

    try {
      const result = await commands.deleteHotwordFile(filePath);
      if (result.status === "ok") {
        await loadHotwordFiles();
      } else {
        console.error("Failed to delete file:", result.error);
        alert(`删除失败: ${result.error}`);
      }
    } catch (error) {
      console.error("Failed to delete file:", error);
      alert(`删除失败: ${error}`);
    }
  };

  const handleViewFile = async (filePath: string) => {
    try {
      const result = await commands.readHotwordFile(filePath);
      if (result.status === "ok") {
        setFileContent(result.data);
        setEditingContent(result.data);
        setViewingFile(filePath);
        setIsEditing(false);
      } else {
        console.error("Failed to read file:", result.error);
        alert(`读取文件失败: ${result.error}`);
      }
    } catch (error) {
      console.error("Failed to read file:", error);
      alert(`读取文件失败: ${error}`);
    }
  };

  const handleEditFile = () => {
    setIsEditing(true);
    setEditingContent(fileContent);
  };

  const handleSaveFile = async () => {
    if (!viewingFile) return;

    try {
      setLoading(true);
      const result = await commands.writeHotwordFile(viewingFile, editingContent);
      if (result.status === "ok") {
        setFileContent(editingContent);
        setIsEditing(false);
        alert("文件保存成功");
      } else {
        console.error("Failed to save file:", result.error);
        alert(`保存文件失败: ${result.error}`);
      }
    } catch (error) {
      console.error("Failed to save file:", error);
      alert(`保存文件失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setEditingContent(fileContent);
  };

  const handleToggleSelection = async (filePath: string) => {
    try {
      const result = await commands.toggleHotwordFileSelection(filePath);
      if (result.status === "ok") {
        await loadHotwordFiles();
      } else {
        console.error("Failed to toggle selection:", result.error);
      }
    } catch (error) {
      console.error("Failed to toggle selection:", error);
    }
  };

  return (
    <div className="max-w-4xl w-full mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
          词典
        </h1>
        <Button
          onClick={(e) => {
            e.preventDefault();
            console.log("Upload button clicked");
            handleUploadFile();
          }}
          disabled={loading}
          variant="primary"
          size="sm"
          className="flex items-center gap-2"
        >
          <Upload className="w-4 h-4" />
          上传热词文件
        </Button>
      </div>

      {/* Search/Add Input Box */}
      <div className="space-y-3">
        <div
          className="relative rounded-lg overflow-hidden"
          style={{
            background: "rgba(255, 255, 255, 0.15)",
            backdropFilter: "blur(25px) saturate(200%)",
            WebkitBackdropFilter: "blur(25px) saturate(200%)",
            border: "1px solid rgba(139, 92, 246, 0.25)",
            boxShadow: "0 4px 16px 0 rgba(139, 92, 246, 0.1)",
          }}
        >
          <div className="flex items-center gap-2 p-3">
            <Search className="w-5 h-5 text-gray-400 flex-shrink-0" />
            <Input
              type="text"
              placeholder="搜索或添加自定义词汇（将添加到 UserDefault.txt）..."
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={handleKeyPress}
              className="flex-1 bg-transparent text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none text-sm border-0"
              style={{ background: "transparent" }}
              disabled={isUpdating("custom_words")}
            />
            {canAdd && (
              <Button
                onClick={handleAddWord}
                disabled={isUpdating("custom_words")}
                variant="primary"
                size="sm"
                className="flex items-center gap-1 flex-shrink-0"
              >
                <Plus className="w-4 h-4" />
                添加
              </Button>
            )}
          </div>
        </div>

        {/* Status Messages */}
        {sanitizedInput && (
          <div
            className="relative rounded-lg p-3 overflow-hidden"
            style={{
              background: wordExists
                ? "rgba(239, 68, 68, 0.1)"
                : canAdd
                  ? "rgba(34, 197, 94, 0.1)"
                  : "rgba(255, 255, 255, 0.1)",
              backdropFilter: "blur(20px) saturate(180%)",
              WebkitBackdropFilter: "blur(20px) saturate(180%)",
              border: `1px solid ${wordExists ? "rgba(239, 68, 68, 0.3)" : canAdd ? "rgba(34, 197, 94, 0.3)" : "rgba(139, 92, 246, 0.2)"}`,
            }}
          >
            <div className="flex items-start gap-2">
              {wordExists ? (
                <>
                  <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
                  <div className="flex-1">
                    <p className="text-sm font-medium text-red-600 dark:text-red-400 mb-1">
                      词汇已存在
                    </p>
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                      该词汇已在词典中，下方显示匹配的搜索结果
                    </p>
                  </div>
                </>
              ) : trimmedInput.includes(" ") ? (
                <>
                  <AlertCircle className="w-5 h-5 text-yellow-500 flex-shrink-0 mt-0.5" />
                  <div className="flex-1">
                    <p className="text-sm font-medium text-yellow-600 dark:text-yellow-400 mb-1">
                      词汇不能包含空格
                    </p>
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                      自定义词汇必须是单个词语，不能包含空格
                    </p>
                  </div>
                </>
              ) : sanitizedInput.length > 50 ? (
                <>
                  <AlertCircle className="w-5 h-5 text-yellow-500 flex-shrink-0 mt-0.5" />
                  <div className="flex-1">
                    <p className="text-sm font-medium text-yellow-600 dark:text-yellow-400 mb-1">
                      词汇过长
                    </p>
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                      词汇长度不能超过50个字符
                    </p>
                  </div>
                </>
              ) : canAdd ? (
                <>
                  <CheckCircle2 className="w-5 h-5 text-green-500 flex-shrink-0 mt-0.5" />
                  <div className="flex-1">
                    <p className="text-sm font-medium text-green-600 dark:text-green-400 mb-1">
                      可以添加
                    </p>
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                      按 Enter 键或点击"添加"按钮添加此词汇到 UserDefault.txt
                    </p>
                  </div>
                </>
              ) : null}
            </div>
          </div>
        )}
      </div>

      {/* Hotword Files Section */}
      <div className="space-y-3">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          热词文件
        </h2>
        {hotwordFiles.length === 0 ? (
          <div
            className="text-center py-8 text-gray-500 dark:text-gray-400 rounded-lg"
            style={{
              background: "rgba(255, 255, 255, 0.05)",
              backdropFilter: "blur(10px)",
              WebkitBackdropFilter: "blur(10px)",
            }}
          >
            暂无热词文件，点击"上传热词文件"按钮上传
          </div>
        ) : (
          <div className="space-y-2">
            {hotwordFiles.map((file) => (
              <div
                key={file.path}
                className={`group relative rounded-lg px-4 py-3 overflow-hidden transition-all duration-200 ${
                  file.selected ? "ring-2 ring-purple-500" : ""
                }`}
                style={{
                  background: file.selected
                    ? "rgba(139, 92, 246, 0.15)"
                    : "rgba(255, 255, 255, 0.15)",
                  backdropFilter: "blur(25px) saturate(200%)",
                  WebkitBackdropFilter: "blur(25px) saturate(200%)",
                  border: file.selected
                    ? "1px solid rgba(139, 92, 246, 0.4)"
                    : "1px solid rgba(139, 92, 246, 0.25)",
                  boxShadow: "0 4px 16px 0 rgba(139, 92, 246, 0.1)",
                }}
              >
                <div className="relative z-10 flex items-center justify-between">
                  <div className="flex items-center gap-3 flex-1">
                    <button
                      onClick={() => handleToggleSelection(file.path)}
                      className={`w-5 h-5 rounded border-2 flex items-center justify-center transition-colors ${
                        file.selected
                          ? "bg-purple-500 border-purple-500"
                          : "border-gray-300 dark:border-gray-600"
                      }`}
                    >
                      {file.selected && <Check className="w-3 h-3 text-white" />}
                    </button>
                    <span className="text-gray-900 dark:text-gray-100 font-medium flex-1">
                      {file.name}
                    </span>
                    {file.selected && (
                      <span className="text-xs text-purple-600 dark:text-purple-400 px-2 py-1 rounded bg-purple-500/10">
                        已选中
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleViewFile(file.path)}
                      className="p-1.5 text-gray-400 hover:text-blue-500 transition-colors rounded"
                      title="查看/编辑"
                    >
                      <Eye className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => handleDeleteFile(file.path)}
                      className="p-1.5 text-gray-400 hover:text-red-500 transition-colors rounded"
                      title="删除"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* File Content Modal */}
      {viewingFile && (
        <div
          className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
          onClick={() => {
            setViewingFile(null);
            setFileContent("");
          }}
        >
          <div
            className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[80vh] overflow-auto"
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "rgba(255, 255, 255, 0.95)",
              backdropFilter: "blur(20px)",
              WebkitBackdropFilter: "blur(20px)",
            }}
          >
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                {hotwordFiles.find((f) => f.path === viewingFile)?.name || "文件内容"}
              </h3>
              <div className="flex items-center gap-2">
                {!isEditing ? (
                  <button
                    onClick={handleEditFile}
                    className="p-1.5 text-gray-400 hover:text-blue-500 transition-colors rounded"
                    title="编辑"
                  >
                    <Edit2 className="w-5 h-5" />
                  </button>
                ) : (
                  <>
                    <button
                      onClick={handleSaveFile}
                      disabled={loading}
                      className="p-1.5 text-gray-400 hover:text-green-500 transition-colors rounded disabled:opacity-50"
                      title="保存"
                    >
                      <Save className="w-5 h-5" />
                    </button>
                    <button
                      onClick={handleCancelEdit}
                      disabled={loading}
                      className="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors rounded disabled:opacity-50"
                      title="取消"
                    >
                      <X className="w-5 h-5" />
                    </button>
                  </>
                )}
                {!isEditing && (
                  <button
                    onClick={() => {
                      setViewingFile(null);
                      setFileContent("");
                      setIsEditing(false);
                    }}
                    className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    title="关闭"
                  >
                    <X className="w-5 h-5" />
                  </button>
                )}
              </div>
            </div>
            {isEditing ? (
              <textarea
                value={editingContent}
                onChange={(e) => setEditingContent(e.target.value)}
                className="w-full h-96 p-3 text-sm text-gray-900 dark:text-gray-100 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg font-mono resize-none focus:outline-none focus:ring-2 focus:ring-purple-500"
                placeholder="输入热词，每行一个..."
                disabled={loading}
              />
            ) : (
              <pre className="text-sm text-gray-900 dark:text-gray-100 whitespace-pre-wrap font-mono bg-gray-50 dark:bg-gray-900 p-3 rounded-lg max-h-[60vh] overflow-auto">
                {fileContent}
              </pre>
            )}
          </div>
        </div>
      )}

      {/* Dictionary Words */}
      <div className="space-y-3">
        {filteredWords.length === 0 ? (
          <div
            className="text-center py-12 text-gray-500 dark:text-gray-400 rounded-lg"
            style={{
              background: "rgba(255, 255, 255, 0.05)",
              backdropFilter: "blur(10px)",
              WebkitBackdropFilter: "blur(10px)",
            }}
          >
            {sanitizedInput
              ? "没有找到匹配的词汇"
              : "词典为空，在上方输入框输入词汇开始添加"}
          </div>
        ) : (
          <>
            {sanitizedInput && (
              <div className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                找到 {filteredWords.length} 个匹配的词汇
                {wordExists && "（包含已存在的词汇）"}
              </div>
            )}
            <div className="flex flex-wrap gap-2">
              {filteredWords.map((word) => {
                const isExactMatch =
                  word.toLowerCase() === sanitizedInput.toLowerCase();
                return (
                  <div
                    key={word}
                    className={`group relative rounded-lg px-4 py-2 overflow-hidden transition-all duration-500 hover:scale-105 ${
                      isExactMatch && wordExists ? "ring-2 ring-red-400" : ""
                    }`}
                    style={{
                      background:
                        isExactMatch && wordExists
                          ? "rgba(239, 68, 68, 0.15)"
                          : "rgba(255, 255, 255, 0.15)",
                      backdropFilter: "blur(25px) saturate(200%)",
                      WebkitBackdropFilter: "blur(25px) saturate(200%)",
                      border:
                        isExactMatch && wordExists
                          ? "1px solid rgba(239, 68, 68, 0.4)"
                          : "1px solid rgba(139, 92, 246, 0.25)",
                      boxShadow: "0 4px 16px 0 rgba(139, 92, 246, 0.1)",
                    }}
                  >
                    {/* Glass reflection effect */}
                    <div
                      className="absolute top-0 left-0 w-full h-1/2 opacity-10 group-hover:opacity-20 transition-opacity duration-500"
                      style={{
                        background:
                          "linear-gradient(180deg, rgba(255, 255, 255, 0.4) 0%, transparent 100%)",
                      }}
                    />

                    <div className="relative z-10 flex items-center gap-2">
                      <span className="text-gray-900 dark:text-gray-100 font-medium">
                        {word}
                      </span>
                      {isExactMatch && wordExists && (
                        <span className="text-xs text-red-500 dark:text-red-400 px-1.5 py-0.5 rounded bg-red-500/10">
                          已存在
                        </span>
                      )}
                      <button
                        onClick={() => handleRemoveWord(word)}
                        disabled={isUpdating("custom_words")}
                        className="p-1 text-gray-400 hover:text-red-500 transition-colors rounded"
                        title="删除"
                      >
                        <X className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </>
        )}
      </div>
    </div>
  );
};
