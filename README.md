# VST3-HelloWorld-Rust
Experimental project for migrating to the Rust programming language for VST3 audio plug-ins.

# Rust VST3 + HTML (Wry/WebView) Starter

This is a **scaffold** for a cross-platform VST3 plugin written in **Rust** with an **HTML5 UI** rendered by a WebView:

- **Windows**: WebView2 via **wry** (under the hood) and a child TAO window parented to the host\'s HWND.
- **macOS**: **WKWebView** via **wry** (child webview supported); attach the view under the host\'s NSView.
- **Linux**: WebKitGTK via wry (X11).

> ⚠️ This repository is a starter scaffold intended to help you wire up the essential bits. It includes TODOs for the platform-specific attach logic and the VST3 COM/class factory export. Use the docs/citations below to complete those parts.

---

## Prerequisites

- Rust (MSVC toolchain on Windows)
- Edge **WebView2 Runtime** (Windows) — Evergreen or fixed. See Microsoft Edge WebView2 docs.  
- Steinberg **VST3 SDK** (for reference/testing tools like Validator & VST3Inspector).  

**References:**
- Wry crate supports macOS, Windows, Linux and child webviews: 
  - <https://docs.rs/wry/latest/wry/>  
  - <https://github.com/tauri-apps/wry>
- TAO Windows parent window API: 
  - <https://docs.rs/tao/latest/i686-pc-windows-msvc/tao/platform/windows/trait.WindowBuilderExtWindows.html>
- WebView2 overview & Win32 getting started: 
  - <https://developer.microsoft.com/en-us/microsoft-edge/webview2>
  - <https://learn.microsoft.com/en-us/microsoft-edge/webview2/get-started/win32>
- VST3 interfaces, editor view lifecycle and packaging:
  - IPlugView lifecycle (Windows HWND / macOS NSView): <https://steinbergmedia.github.io/vst3_doc/base/classSteinberg_1_1IPlugView.html>
  - Module factory / GetPluginFactory: <https://steinbergmedia.github.io/vst3_doc/base/group__pluginBase.html>
  - Bundle format and `moduleinfo.json`: <https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Plugin+Format.html>
  - SDK repo (MIT): <https://github.com/steinbergmedia/vst3sdk>

---

## Repository layout

```
my_html_vst3/
├─ Cargo.toml
├─ build.rs                     # optional, reserved for future bundling tasks
├─ src/
│  ├─ lib.rs                    # VST3 entry + component/controller stubs
│  ├─ processor.rs              # real-time audio skeleton
│  ├─ controller.rs             # parameters + automation
│  └─ view.rs                   # IPlugView skeleton + Wry/Tao child webview
├─ Resources/
│  ├─ index.html
│  ├─ main.css
│  ├─ ui.js
│  └─ moduleinfo.json           # VST3 bundle metadata (example)
├─ scripts/
│  ├─ win-bundle.ps1            # creates a Windows .vst3 bundle folder
│  └─ install-win.ps1           # installs the bundle to Common Files\VST3
└─ .gitignore
```

---

## Building the DLL

```bash
# Windows (MSVC)
cargo build --release
```
The produced DLL (e.g. `target\\release\\my_html_vst3.dll`) will be copied into a VST3 bundle by the script below.

---

## Create the Windows .vst3 bundle

```powershell
# From repository root
./scripts/win-bundle.ps1
```
This produces:

```
MyHtmlVst3.vst3/Contents/
  x86_64-win/MyHtmlVst3.vst3   # the actual DLL, renamed with .vst3 extension
  Resources/                   # web assets + moduleinfo.json
```

The structure follows Steinberg\'s Windows bundle format for VST3. See docs: <https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Plugin+Format.html>

---

## Install for testing (Windows)

```powershell
./scripts/install-win.ps1
```
Installs the bundle to `C:\\Program Files\\Common Files\\VST3\\` (the standard path DAWs scan). See: <https://helpcenter.steinberg.de/hc/en-us/articles/115000177084-VST-plug-in-locations-on-Windows>

---

## Test in **Reaper** and **Cubase**

1. Launch the DAW, force a plugin rescan if necessary.
2. Insert the effect/synth; open the editor — you should see the scaffold HTML UI.
3. Use the slider in the UI to send/receive JSON messages; the controller stub shows where to translate these to parameter edits (begin/perform/end edit) that hosts can automate. See the VST3 editor/parameter docs:  
   - <https://steinbergmedia.github.io/vst3_doc/base/classSteinberg_1_1IPlugView.html>  
   - <https://steinbergmedia.github.io/vst3_dev_portal/pages/index.html>

---

## macOS notes

- VST3 hosts call `IPlugView::attached(parent, kPlatformTypeNSView)` and pass an NSView. The scaffold includes a macOS section with TODOs to attach the Wry **child webview** into that NSView. Child webviews are supported on macOS by wry.  
  - <https://docs.rs/wry/latest/wry/>  

- Packaging differs (Mach‑O bundle with `Contents/MacOS/` etc.). Refer to the SDK docs for bundle structure and signing/hardening for distribution.

---

## Messaging between HTML and Rust

- Use `window.postMessage` (wry bridge) / `evaluate_script` to exchange JSON messages. On Windows WebView2, the underlying pattern is identical (post JSON from JS, handle in rust and vice versa).  
  - WebView2 basics: <https://learn.microsoft.com/en-us/microsoft-edge/webview2/get-started/win32>

- Translate UI messages into VST3 `beginEdit/performEdit/endEdit` calls on your parameter controller so hosts can automate.

---

## License

You own whatever you build using this scaffold. External crates referenced are under their own licenses:
- `wry` (MIT/Apache-2.0)
- `tao` (MIT/Apache-2.0)
- `vst3` (MIT/Apache-2.0), bindings based on Steinberg VST3 SDK (MIT)

---

## Disclaimer
This scaffold intentionally keeps the VST3 COM/class factory export minimal with TODOs. Use the VST3 SDK docs and the `vst3` crate docs to complete your factory metadata and class registration (the host calls `GetPluginFactory()` to instantiate components).  
- <https://steinbergmedia.github.io/vst3_doc/base/group__pluginBase.html>  
- <https://docs.rs/vst3/latest/vst3/>