import React, { useState } from "react";
import { Eye, EyeOff } from "lucide-react";
import { Input } from "../../ui/Input";

interface ApiKeyFieldProps {
  value: string;
  onBlur: (value: string) => void;
  disabled: boolean;
  placeholder?: string;
  className?: string;
}

export const ApiKeyField: React.FC<ApiKeyFieldProps> = React.memo(
  ({ value, onBlur, disabled, placeholder, className = "" }) => {
    const [localValue, setLocalValue] = useState(value);
    const [showPassword, setShowPassword] = useState(false);
    const inputRef = React.useRef<HTMLInputElement>(null);

    // Sync with prop changes
    React.useEffect(() => {
      setLocalValue(value);
    }, [value]);

    // Handle paste event to ensure it works properly
    const handlePaste = (event: React.ClipboardEvent<HTMLInputElement>) => {
      event.preventDefault();
      const pastedText = event.clipboardData.getData("text/plain");
      if (pastedText) {
        const trimmedText = pastedText.trim();
        setLocalValue(trimmedText);
        // Update input value directly
        if (inputRef.current) {
          inputRef.current.value = trimmedText;
        }
        // Save immediately
        onBlur(trimmedText);
      }
    };

    return (
      <div className="relative flex-1 min-w-[320px]">
        <Input
          ref={inputRef}
          type={showPassword ? "text" : "password"}
          value={localValue}
          onChange={(event) => setLocalValue(event.target.value)}
          onPaste={handlePaste}
          onBlur={() => onBlur(localValue)}
          placeholder={placeholder}
          variant="compact"
          disabled={disabled}
          className={`w-full pr-10 ${className}`}
        />
        <button
          type="button"
          onClick={() => setShowPassword(!showPassword)}
          disabled={disabled}
          className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-mid-gray hover:text-logo-primary transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          aria-label={showPassword ? "Hide API key" : "Show API key"}
        >
          {showPassword ? (
            <EyeOff className="h-4 w-4" />
          ) : (
            <Eye className="h-4 w-4" />
          )}
        </button>
      </div>
    );
  },
);

ApiKeyField.displayName = "ApiKeyField";
