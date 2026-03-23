import React from "react";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";

interface TextareaProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  variant?: "default" | "compact";
}

export const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className = "", variant = "default", onKeyDown, value, onChange, ...props }, ref) => {
    const baseClasses =
      "px-2 py-1 text-sm font-semibold bg-mid-gray/10 border border-mid-gray/80 rounded text-left transition-[background-color,border-color] duration-150 hover:bg-logo-primary/10 hover:border-logo-primary focus:outline-none focus:bg-logo-primary/10 focus:border-logo-primary resize-y";

    const variantClasses = {
      default: "px-3 py-2 min-h-[100px]",
      compact: "px-2 py-1 min-h-[80px]",
    };

    const handleKeyDown = async (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      // Handle Ctrl+V / Cmd+V manually using Tauri clipboard API
      // This is needed because Tauri's global shortcut system may interfere with paste
      const isPasteKey = (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "v";
      
      // Handle Ctrl+C / Cmd+C manually using Tauri clipboard API
      // This is needed because Tauri's global shortcut system may interfere with copy
      const isCopyKey = (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "c";


      // Handle paste
      if (isPasteKey) {
        // Try to get the textarea element from ref or event target
        const textareaElement = (ref && "current" in ref && ref.current) 
          ? ref.current 
          : (e.currentTarget as HTMLTextAreaElement);

        if (textareaElement) {
          try {
            const clipboardText = await readText();
            

            if (clipboardText && onChange) {
              // Only prevent default if we can handle it
              e.preventDefault();
              e.stopPropagation();
              
              const start = textareaElement.selectionStart || 0;
              const end = textareaElement.selectionEnd || 0;
              const currentValue = typeof value === "string" ? value : textareaElement.value;
              const newValue =
                currentValue.substring(0, start) +
                clipboardText +
                currentValue.substring(end);
              
              // Create a synthetic event for onChange
              const syntheticEvent = {
                target: { value: newValue },
                currentTarget: textareaElement,
              } as React.ChangeEvent<HTMLTextAreaElement>;
              
              onChange(syntheticEvent);

              // Update cursor position
              setTimeout(() => {
                if (textareaElement) {
                  const newCursorPos = start + clipboardText.length;
                  textareaElement.setSelectionRange(newCursorPos, newCursorPos);
                }
              }, 0);
              
              return;
            } else {
              // Allow default paste behavior if we can't handle it
              // Don't prevent default - let browser handle it
              if (onKeyDown) {
                onKeyDown(e);
              }
              return;
            }
          } catch (error) {
            console.error("Failed to paste from clipboard:", error);
            // Allow default paste behavior on error
            if (onKeyDown) {
              onKeyDown(e);
            }
            return;
          }
        }
      }

      // Handle copy
      if (isCopyKey) {
        
        // Try to get the textarea element from ref or event target
        const textareaElement = (ref && "current" in ref && ref.current) 
          ? ref.current 
          : (e.currentTarget as HTMLTextAreaElement);


        if (textareaElement) {
          try {
            const start = textareaElement.selectionStart || 0;
            const end = textareaElement.selectionEnd || 0;
            
            
            if (start !== end) {
              // Only prevent default if we have a selection to copy
              e.preventDefault();
              e.stopPropagation();
              
              const currentValue = typeof value === "string" ? value : textareaElement.value;
              const selectedText = currentValue.substring(start, end);
              
              
              if (selectedText) {
                await writeText(selectedText);
              }
              return;
            } else {
              // No selection - allow default copy behavior (copy all text)
              // Don't prevent default - let browser handle it
              if (onKeyDown) {
                onKeyDown(e);
              }
              return;
            }
          } catch (error) {
            console.error("Failed to copy to clipboard:", error);
            // Allow default copy behavior on error
            if (onKeyDown) {
              onKeyDown(e);
            }
            return;
          }
        }
      }

      // Call original handler for other keys
      if (onKeyDown) {
        onKeyDown(e);
      }
    };

    return (
      <textarea
        ref={ref}
        className={`${baseClasses} ${variantClasses[variant]} ${className}`}
        value={value}
        onChange={onChange}
        onKeyDown={handleKeyDown}
        {...props}
      />
    );
  },
);

Textarea.displayName = "Textarea";
