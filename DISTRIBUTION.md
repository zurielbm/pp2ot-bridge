# Distributing PP2OT Bridge for macOS

This guide explains how to build and distribute the app without an Apple Developer account.

## Building for Release

### 1. Build the app bundle
```bash
dx bundle --release
```

The app will be created at:
```
target/dx/pp2ot-bridge/release/macos/Pp2OtBridge.app
```

### 2. Sign the app (ad-hoc)
```bash
codesign --force --deep --sign - "target/dx/pp2ot-bridge/release/macos/Pp2OtBridge.app"
```

This creates an ad-hoc signature that allows the app to run after bypassing Gatekeeper.

### 3. Create a distributable zip
```bash
cd target/dx/pp2ot-bridge/release/macos
ditto -c -k --sequesterRsrc --keepParent "Pp2OtBridge.app" "../../../../../PP2OT-Bridge-mac-arm.zip"
```

> **Note**: Use `ditto` instead of `zip` - it preserves macOS extended attributes and code signatures.

---

## Instructions for Recipients

When sharing the app, include these instructions:

### After downloading and extracting the zip:

1. Open **Terminal**
2. Run this command (adjust the path to where you extracted the app):
   ```bash
   xattr -cr ~/Downloads/Pp2OtBridge.app
   ```
3. Now you can open the app normally

### Why is this needed?

macOS Gatekeeper blocks apps that aren't signed with an Apple Developer certificate and notarized. The `xattr -cr` command removes the "quarantine" flag that macOS adds to downloaded files, allowing the app to run.

---

## Quick Reference

| Step | Command |
|------|---------|
| Build | `dx bundle --release` |
| Sign | `codesign --force --deep --sign - "target/dx/pp2ot-bridge/release/macos/Pp2OtBridge.app"` |
| Zip | `ditto -c -k --sequesterRsrc --keepParent "Pp2OtBridge.app" "PP2OT-Bridge-mac-arm.zip"` |
| Recipient | `xattr -cr /path/to/Pp2OtBridge.app` |

---

## Automated Workflow

You can also use the workflow command:
```
/release-app
```

This will run all the build, sign, and zip steps automatically.
