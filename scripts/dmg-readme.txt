KeVoiceInput Installation Guide
════════════════════════════════════════════════════════════════

⚠️ 若双击 Install.command 被阻止：
   → 右键点击 Install.command → 选「打开」→ 在弹窗中点「打开」（无需终端）

════════════════════════════════════════════════════════════════

METHOD 1: Auto Installer (Recommended) 🚀
──────────────────────────────────────────
1. Double-click "Install.command"
2. If macOS blocks it, see "Troubleshooting" below
3. Follow the prompts to complete installation


METHOD 2: Manual Installation (Always Works) ✅
──────────────────────────────────────────
1. Drag "KeVoiceInput.app" to the "Applications" folder
2. Open Terminal and run:
   xattr -cr /Applications/KeVoiceInput.app
3. Right-click the app in Finder → Open
4. Click "Open" in the dialog


TROUBLESHOOTING
──────────────────────────────────────────
Problem: "Install.command" shows security warning

Option A: Right-click → Open (simplest, no Terminal)
1. Right-click "Install.command"
2. Choose "Open"
3. In the dialog, click "Open" again

Option B: Use Terminal
1. Open Terminal (⌘+Space, type "Terminal")
2. Run: /Volumes/KeVoiceInput/Install.command

Option C: Use Manual Installation (METHOD 2 above)

────────────────────────────────────────────────────

Problem: App won't open / 应用无法打开

• First time: Right-click KeVoiceInput.app → Open → click "Open" in dialog
• If still blocked: Open Terminal, run:
  xattr -cr /Applications/KeVoiceInput.app
  Then right-click the app → Open again
• Still fails: Run in Terminal to see the real error:
  /Applications/KeVoiceInput.app/Contents/MacOS/kevoiceinput
  (Copy the error message for support. Often: "Library not loaded" = reinstall from latest DMG)

────────────────────────────────────────────────────

Problem: App crashes on launch ("unexpectedly quit")

This was FIXED in DMG built on 2026-02-22 or later.

If using an older DMG:
1. Download the latest version
2. Reinstall using Install.command

To verify your installation:
1. Open Terminal
2. Run:
   ls -la /Applications/KeVoiceInput.app/Contents/Frameworks/
3. Should see 4 .dylib files

If missing, reinstall with latest DMG


FIRST LAUNCH
──────────────────────────────────────────
After installation:
1. Grant Accessibility permission when prompted
   (Required for typing transcription results)
2. Grant Microphone permission
3. Download a speech recognition model
4. Start using voice input!


SUPPORT
──────────────────────────────────────────
For issues, visit:
https://github.com/thinkre/KeVoiceInput/issues

════════════════════════════════════════════════════════════════
Thank you for using KeVoiceInput!
