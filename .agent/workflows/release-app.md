---
description: Build and release the macOS app for distribution (without Apple Developer account)
---

# Release App Workflow

This workflow creates a distributable macOS app bundle with ad-hoc signing.

## Prerequisites
- Dioxus CLI installed (`cargo install dioxus-cli`)
- App icons in `assets/icon.icns`

## Steps

// turbo
### 1. Build the release bundle
```bash
dx bundle --release
```
This creates the `.app` bundle in `target/dx/pp2ot-bridge/bundle/macos/`.

// turbo
### 2. Sign the app with ad-hoc signature
```bash
codesign --force --deep --sign - "target/dx/pp2ot-bridge/bundle/macos/PP2OT Bridge.app"
```
Ad-hoc signing allows the app to run on other Macs after removing the quarantine attribute.

// turbo
### 3. Create the distributable zip
```bash
cd target/dx/pp2ot-bridge/bundle/macos && ditto -c -k --sequesterRsrc --keepParent "PP2OT Bridge.app" "../../../../../PP2OT-Bridge-mac-arm.zip"
```
Uses `ditto` instead of `zip` to preserve macOS metadata and code signatures.

// turbo
### 4. Verify the zip was created
```bash
ls -la PP2OT-Bridge-mac-arm.zip
```

## Distribution Instructions for Recipients

**IMPORTANT**: Include these instructions when sharing the app:

Recipients must run this command before launching the app:
```bash
xattr -cr "/path/to/PP2OT Bridge.app"
```

This removes the quarantine attribute that macOS adds to downloaded files.

### Example message to send with the app:
> After extracting the zip, open Terminal and run:
> `xattr -cr "/path/to/PP2OT Bridge.app"`
> Then you can open the app normally.

## Notes
- Without an Apple Developer account ($99/year), you cannot notarize the app
- Recipients will always need to run the `xattr` command to bypass Gatekeeper
- The app will work normally after removing the quarantine attribute
