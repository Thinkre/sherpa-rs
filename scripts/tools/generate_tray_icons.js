#!/usr/bin/env node
/**
 * Generate tray icons for macOS menu bar
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Output directory
const outputDir = path.join(__dirname, "..", "src-tauri", "resources");

// Ensure output directory exists
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

// SVG templates for each state
const createIdleSVG = (isDark) => `
<svg width="44" height="44" viewBox="0 0 44 44" xmlns="http://www.w3.org/2000/svg">
    <rect x="16" y="12" width="12" height="18" rx="6" fill="${isDark ? "rgba(255,255,255,0.8)" : "rgba(0,0,0,0.7)"}"/>
    <line x1="22" y1="30" x2="22" y2="36" stroke="${isDark ? "rgba(255,255,255,0.8)" : "rgba(0,0,0,0.7)"}" stroke-width="2" stroke-linecap="round"/>
    <line x1="17" y1="36" x2="27" y2="36" stroke="${isDark ? "rgba(255,255,255,0.8)" : "rgba(0,0,0,0.7)"}" stroke-width="2" stroke-linecap="round"/>
</svg>
`;

const createRecordingSVG = (isDark) => `
<svg width="44" height="44" viewBox="0 0 44 44" xmlns="http://www.w3.org/2000/svg">
    <rect x="16" y="12" width="12" height="18" rx="6" fill="${isDark ? "rgb(255,100,100)" : "rgb(220,50,50)"}"/>
    <line x1="22" y1="30" x2="22" y2="36" stroke="${isDark ? "rgb(255,100,100)" : "rgb(220,50,50)"}" stroke-width="2" stroke-linecap="round"/>
    <line x1="17" y1="36" x2="27" y2="36" stroke="${isDark ? "rgb(255,100,100)" : "rgb(220,50,50)"}" stroke-width="2" stroke-linecap="round"/>
    <circle cx="22" cy="7" r="2" fill="${isDark ? "rgb(255,80,80)" : "rgb(255,50,50)"}"/>
    <path d="M 10 17 Q 8 22 10 27" stroke="${isDark ? "rgb(255,100,100)" : "rgb(220,50,50)"}" stroke-width="2" fill="none" stroke-linecap="round" opacity="0.7"/>
    <path d="M 34 17 Q 36 22 34 27" stroke="${isDark ? "rgb(255,100,100)" : "rgb(220,50,50)"}" stroke-width="2" fill="none" stroke-linecap="round" opacity="0.7"/>
</svg>
`;

const createTranscribingSVG = (isDark) => `
<svg width="44" height="44" viewBox="0 0 44 44" xmlns="http://www.w3.org/2000/svg">
    <rect x="16" y="12" width="12" height="18" rx="6" fill="${isDark ? "rgb(167,139,250)" : "rgb(139,92,246)"}"/>
    <line x1="22" y1="30" x2="22" y2="36" stroke="${isDark ? "rgb(167,139,250)" : "rgb(139,92,246)"}" stroke-width="2" stroke-linecap="round"/>
    <line x1="17" y1="36" x2="27" y2="36" stroke="${isDark ? "rgb(167,139,250)" : "rgb(139,92,246)"}" stroke-width="2" stroke-linecap="round"/>
    <path d="M 8 17 Q 6 22 8 27" stroke="${isDark ? "rgb(167,139,250)" : "rgb(139,92,246)"}" stroke-width="2" fill="none" stroke-linecap="round" opacity="0.6"/>
    <path d="M 36 17 Q 38 22 36 27" stroke="${isDark ? "rgb(167,139,250)" : "rgb(139,92,246)"}" stroke-width="2" fill="none" stroke-linecap="round" opacity="0.6"/>
    <path d="M 12 19 Q 10 22 12 25" stroke="${isDark ? "rgb(167,139,250)" : "rgb(139,92,246)"}" stroke-width="1.5" fill="none" stroke-linecap="round" opacity="0.4"/>
    <path d="M 32 19 Q 34 22 32 25" stroke="${isDark ? "rgb(167,139,250)" : "rgb(139,92,246)"}" stroke-width="1.5" fill="none" stroke-linecap="round" opacity="0.4"/>
</svg>
`;

// Function to save SVG (which macOS can render directly)
function saveSVG(filename, svgContent) {
  const outputPath = path.join(outputDir, filename);
  fs.writeFileSync(outputPath, svgContent.trim());
  console.log(`Created: ${outputPath}`);
}

// Main function
function main() {
  console.log("Generating tray icons...\n");

  // Light mode icons
  saveSVG("tray_idle.svg", createIdleSVG(false));
  saveSVG("tray_recording.svg", createRecordingSVG(false));
  saveSVG("tray_transcribing.svg", createTranscribingSVG(false));

  // Dark mode icons
  saveSVG("tray_idle_dark.svg", createIdleSVG(true));
  saveSVG("tray_recording_dark.svg", createRecordingSVG(true));
  saveSVG("tray_transcribing_dark.svg", createTranscribingSVG(true));

  console.log("\nAll tray icons generated successfully!");
  console.log(`Icons saved to: ${outputDir}`);
  console.log(
    "\nNote: SVG icons work on macOS. To convert to PNG, open the HTML file in scripts/tray_icons.html",
  );
}

main();
