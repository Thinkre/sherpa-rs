import React from "react";
import { Loader2 } from "lucide-react";

interface TranscriptionIconProps {
  width?: number;
  height?: number;
  color?: string;
  className?: string;
}

const TranscriptionIcon: React.FC<TranscriptionIconProps> = ({
  width = 24,
  height = 24,
  color = "#8b5cf6",
  className = "",
}) => {
  return (
    <Loader2
      size={width}
      color={color}
      className={`${className} animate-spin`}
      strokeWidth={2}
    />
  );
};

export default TranscriptionIcon;
