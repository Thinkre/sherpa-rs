import React from "react";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  variant?: "default" | "compact";
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className = "", variant = "default", disabled, onKeyDown: customOnKeyDown, value, onChange, ...props }, ref) => {
    const baseClasses =
      "px-2 py-1 text-sm font-semibold bg-mid-gray/10 border border-mid-gray/80 rounded text-left transition-all duration-150";

    const interactiveClasses = disabled
      ? "opacity-60 cursor-not-allowed bg-mid-gray/10 border-mid-gray/40"
      : "hover:bg-logo-primary/10 hover:border-logo-primary focus:outline-none focus:bg-logo-primary/20 focus:border-logo-primary";

    const variantClasses = {
      default: "px-3 py-2",
      compact: "px-2 py-1",
    } as const;

    const handleKeyDown = async (e: React.KeyboardEvent<HTMLInputElement>) => {
      // Handle Ctrl+V / Cmd+V manually using Tauri clipboard API
      // This is needed because Tauri's global shortcut system may interfere with paste
      const isPasteKey = (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "v";
      
      // Handle Ctrl+C / Cmd+C manually using Tauri clipboard API
      // This is needed because Tauri's global shortcut system may interfere with copy
      const isCopyKey = (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "c";


      // Handle paste FIRST (before calling custom onKeyDown) to ensure it works
      if (isPasteKey) {
        // Try to get the input element from ref or event target
        const inputElement = (ref && "current" in ref && ref.current) 
          ? ref.current 
          : (e.currentTarget as HTMLInputElement);

        if (inputElement) {
          try {
            const clipboardText = await readText();
            

            if (clipboardText && onChange) {
              // Only prevent default if we can handle it
              e.preventDefault();
              e.stopPropagation();
              
              const start = inputElement.selectionStart || 0;
              const end = inputElement.selectionEnd || 0;
              const currentValue = typeof value === "string" ? value : inputElement.value;
              const newValue =
                currentValue.substring(0, start) +
                clipboardText +
                currentValue.substring(end);
              
              // Create a synthetic event for onChange
              const syntheticEvent = {
                target: { value: newValue },
                currentTarget: inputElement,
              } as React.ChangeEvent<HTMLInputElement>;
              
              onChange(syntheticEvent);

              // Update cursor position
              setTimeout(() => {
                if (inputElement) {
                  const newCursorPos = start + clipboardText.length;
                  inputElement.setSelectionRange(newCursorPos, newCursorPos);
                }
              }, 0);
              
              return;
            } else {
              // Allow default paste behavior if we can't handle it
              // Don't prevent default - let browser handle it
              // Don't call customOnKeyDown here - let default browser behavior handle it
              return;
            }
          } catch (error) {
            console.error("Failed to paste from clipboard:", error);
            // Allow default paste behavior on error
            // Don't call customOnKeyDown here - let default browser behavior handle it
            return;
          }
        }
      }

      // Handle copy FIRST (before calling custom onKeyDown) to ensure it works
      if (isCopyKey) {
        
        // Try to get the input element from ref or event target
        const inputElement = (ref && "current" in ref && ref.current) 
          ? ref.current 
          : (e.currentTarget as HTMLInputElement);


        if (inputElement) {
          try {
            const start = inputElement.selectionStart || 0;
            const end = inputElement.selectionEnd || 0;
            
            
            if (start !== end) {
              // Only prevent default if we have a selection to copy
              e.preventDefault();
              e.stopPropagation();
              
              const currentValue = typeof value === "string" ? value : inputElement.value;
              const selectedText = currentValue.substring(start, end);
              
              
              if (selectedText) {
                await writeText(selectedText);
              }
              return;
            } else {
              // No selection - allow default copy behavior (copy all text)
              // Don't prevent default - let browser handle it
              // Don't call customOnKeyDown here - let default browser behavior handle it
              return;
            }
          } catch (error) {
            console.error("Failed to copy to clipboard:", error);
            // Allow default copy behavior on error
            // Don't call customOnKeyDown here - let default browser behavior handle it
            return;
          }
        }
      }

      // Call custom handler for other keys (but only if we didn't handle copy/paste)
      if (customOnKeyDown && !isPasteKey && !isCopyKey) {
        customOnKeyDown(e);
      }
    };

    // Extract onKeyDown from props to prevent it from overriding our handler
    const { onKeyDown: _ignoredOnKeyDown, ...restProps } = props as any;
    
    return (
      <input
        ref={ref}
        className={`${baseClasses} ${variantClasses[variant]} ${interactiveClasses} ${className}`}
        disabled={disabled}
        value={value}
        onChange={onChange}
        {...restProps}
        onKeyDown={handleKeyDown}
      />
    );
  },
);

Input.displayName = "Input";
