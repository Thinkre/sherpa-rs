import React from "react";
import { Mic } from "lucide-react";

interface MicrophoneIconProps {
  width?: number;
  height?: number;
  color?: string;
  className?: string;
}

const MicrophoneIcon: React.FC<MicrophoneIconProps> = ({
  width = 24,
  height = 24,
  color = "#8b5cf6",
  className = "",
}) => {
  return (
    <Mic size={width} color={color} className={className} strokeWidth={2} />
  );
};

export default MicrophoneIcon;
