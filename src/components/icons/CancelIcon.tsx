import React from "react";
import { X } from "lucide-react";

interface CancelIconProps {
  width?: number;
  height?: number;
  color?: string;
  className?: string;
}

const CancelIcon: React.FC<CancelIconProps> = ({
  width = 24,
  height = 24,
  color = "#8b5cf6",
  className = "",
}) => {
  return <X size={width} color={color} className={className} strokeWidth={2} />;
};

export default CancelIcon;
