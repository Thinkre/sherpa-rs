import React from "react";

interface VoiceInputLogoProps {
  width?: number;
  height?: number;
  className?: string;
}

const VoiceInputLogo: React.FC<VoiceInputLogoProps> = ({
  width = 200,
  height,
  className,
}) => {
  const size = width;
  const actualHeight = height || width;

  return (
    <svg
      width={size}
      height={actualHeight}
      viewBox="0 0 512 512"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <defs>
        {/* Main gradient: pink to purple to blue */}
        <linearGradient id="mainGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#FF6B9D" />
          <stop offset="50%" stopColor="#A855F7" />
          <stop offset="100%" stopColor="#3B82F6" />
        </linearGradient>
        {/* Microphone gradient */}
        <linearGradient id="micGradient" x1="50%" y1="0%" x2="50%" y2="100%">
          <stop offset="0%" stopColor="#FF6B9D" />
          <stop offset="100%" stopColor="#3B82F6" />
        </linearGradient>
        {/* Rainbow gradient for sound waves */}
        <linearGradient id="rainbowGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#F59E0B" />
          <stop offset="25%" stopColor="#84CC16" />
          <stop offset="50%" stopColor="#22D3D3" />
          <stop offset="75%" stopColor="#3B82F6" />
          <stop offset="100%" stopColor="#8B5CF6" />
        </linearGradient>
      </defs>
      {/* Speech bubble */}
      <path
        d="M256 60C147.5 60 60 137.5 60 232C60 282 85 326 125 356C125 356 110 410 80 440C80 440 160 420 190 400C210 408 232 412 256 412C364.5 412 452 334.5 452 240C452 145.5 364.5 60 256 60Z"
        stroke="url(#mainGradient)"
        strokeWidth="28"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
      {/* Microphone body */}
      <rect
        x="216"
        y="140"
        width="80"
        height="130"
        rx="40"
        fill="url(#micGradient)"
      />
      {/* Microphone lines */}
      <path
        d="M236 180 L236 220"
        stroke="white"
        strokeWidth="6"
        strokeLinecap="round"
        opacity="0.6"
      />
      <path
        d="M256 170 L256 230"
        stroke="white"
        strokeWidth="6"
        strokeLinecap="round"
        opacity="0.6"
      />
      <path
        d="M276 180 L276 220"
        stroke="white"
        strokeWidth="6"
        strokeLinecap="round"
        opacity="0.6"
      />
      {/* Microphone arc */}
      <path
        d="M180 240 C180 290 213 330 256 330 C299 330 332 290 332 240"
        stroke="url(#micGradient)"
        strokeWidth="16"
        strokeLinecap="round"
        fill="none"
      />
      {/* Microphone stand */}
      <path
        d="M256 330 L256 370"
        stroke="url(#micGradient)"
        strokeWidth="16"
        strokeLinecap="round"
      />
      <path
        d="M216 370 L296 370"
        stroke="url(#micGradient)"
        strokeWidth="16"
        strokeLinecap="round"
      />
      {/* Sound wave bars */}
      <g stroke="url(#rainbowGradient)" strokeWidth="10" strokeLinecap="round">
        {/* Left side */}
        <path d="M90 460 L90 470" />
        <path d="M110 450 L110 480" />
        <path d="M130 440 L130 490" />
        <path d="M150 445 L150 485" />
        <path d="M170 455 L170 475" />
        {/* Center */}
        <path d="M196 460 L196 470" />
        <path d="M216 445 L216 485" />
        <path d="M236 435 L236 495" />
        <path d="M256 430 L256 500" />
        <path d="M276 435 L276 495" />
        <path d="M296 445 L296 485" />
        <path d="M316 460 L316 470" />
        {/* Right side */}
        <path d="M342 455 L342 475" />
        <path d="M362 445 L362 485" />
        <path d="M382 440 L382 490" />
        <path d="M402 450 L402 480" />
        <path d="M422 460 L422 470" />
      </g>
    </svg>
  );
};

export default VoiceInputLogo;
