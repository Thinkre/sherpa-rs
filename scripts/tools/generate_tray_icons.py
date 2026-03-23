#!/usr/bin/env python3
"""
Generate tray icons for macOS menu bar
Creates simple, clean icons suitable for menu bar display
"""

from PIL import Image, ImageDraw
import os

# Output directory
output_dir = os.path.join(os.path.dirname(__file__), '..', 'src-tauri', 'resources')
os.makedirs(output_dir, exist_ok=True)

# Icon size for macOS menu bar (22x22 @ 2x = 44x44)
SIZE = 44
CENTER = SIZE // 2

def create_base_microphone(draw, color, opacity=255):
    """Draw a simple microphone icon"""
    # Microphone capsule
    mic_width = 12
    mic_height = 18
    mic_left = CENTER - mic_width // 2
    mic_top = CENTER - mic_height // 2 - 4
    mic_right = CENTER + mic_width // 2
    mic_bottom = mic_top + mic_height

    # Draw microphone body (rounded rectangle)
    draw.rounded_rectangle(
        [mic_left, mic_top, mic_right, mic_bottom],
        radius=6,
        fill=(*color, opacity)
    )

    # Microphone stand
    stand_y = mic_bottom + 2
    draw.line([CENTER, mic_bottom, CENTER, stand_y + 6], fill=(*color, opacity), width=2)

    # Base
    base_width = 10
    draw.line(
        [CENTER - base_width//2, stand_y + 6, CENTER + base_width//2, stand_y + 6],
        fill=(*color, opacity),
        width=2
    )

def create_sound_waves(draw, color, opacity=255, side='left', offset=0):
    """Draw sound waves on one side"""
    wave_opacity = int(opacity * 0.6)

    if side == 'left':
        x = CENTER - 12 - offset
        # Inner wave
        draw.arc([x-3, CENTER-5, x, CENTER+5], 270, 90, fill=(*color, wave_opacity), width=2)
        # Outer wave
        draw.arc([x-6, CENTER-8, x, CENTER+8], 270, 90, fill=(*color, int(wave_opacity*0.7)), width=2)
    else:  # right
        x = CENTER + 12 + offset
        # Inner wave
        draw.arc([x, CENTER-5, x+3, CENTER+5], 90, 270, fill=(*color, wave_opacity), width=2)
        # Outer wave
        draw.arc([x, CENTER-8, x+6, CENTER+8], 90, 270, fill=(*color, int(wave_opacity*0.7)), width=2)

def create_tray_icon(filename, state, is_dark=False):
    """
    Create a tray icon
    state: 'idle', 'recording', 'transcribing'
    is_dark: True for dark mode icons
    """
    # Create image with transparency
    img = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Color scheme
    if is_dark:
        # For dark mode menu bar (light icons)
        base_color = (255, 255, 255)
    else:
        # For light mode menu bar (dark icons)
        base_color = (0, 0, 0)

    if state == 'idle':
        # Simple microphone, no waves
        create_base_microphone(draw, base_color, opacity=200)

    elif state == 'recording':
        # Microphone with red tint and waves
        if is_dark:
            mic_color = (255, 100, 100)  # Light red for dark mode
        else:
            mic_color = (220, 50, 50)  # Dark red for light mode

        create_base_microphone(draw, mic_color, opacity=255)
        create_sound_waves(draw, mic_color, opacity=255, side='left')
        create_sound_waves(draw, mic_color, opacity=255, side='right')

        # Add a small recording indicator (dot)
        dot_radius = 2
        draw.ellipse(
            [CENTER - dot_radius, mic_top - 6, CENTER + dot_radius, mic_top - 2],
            fill=(255, 50, 50, 255)
        )

    elif state == 'transcribing':
        # Microphone with blue/purple tint and animated waves
        if is_dark:
            mic_color = (167, 139, 250)  # Light purple for dark mode
        else:
            mic_color = (139, 92, 246)  # Darker purple for light mode

        create_base_microphone(draw, mic_color, opacity=255)
        create_sound_waves(draw, mic_color, opacity=255, side='left', offset=2)
        create_sound_waves(draw, mic_color, opacity=255, side='right', offset=2)

    # Save the icon
    output_path = os.path.join(output_dir, filename)
    img.save(output_path, 'PNG')
    print(f"Created: {output_path}")

    # Fix reference to mic_top
    mic_height = 18
    mic_top = CENTER - mic_height // 2 - 4

# Generate all tray icons
def main():
    print("Generating tray icons...")

    # Light mode icons
    create_tray_icon('tray_idle.png', 'idle', is_dark=False)
    create_tray_icon('tray_recording.png', 'recording', is_dark=False)
    create_tray_icon('tray_transcribing.png', 'transcribing', is_dark=False)

    # Dark mode icons
    create_tray_icon('tray_idle_dark.png', 'idle', is_dark=True)
    create_tray_icon('tray_recording_dark.png', 'recording', is_dark=True)
    create_tray_icon('tray_transcribing_dark.png', 'transcribing', is_dark=True)

    print("\nAll tray icons generated successfully!")
    print(f"Icons saved to: {output_dir}")

if __name__ == '__main__':
    main()
