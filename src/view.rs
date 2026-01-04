
use vst3::Steinberg::*;
use crate::util; use crate::lib::PLUGIN_NAME;
use serde_json::json;
const EDITOR_W: i32 = 800; const EDITOR_H: i32 = 600;
#[derive(Default)] pub struct VstPlugView { pub inner: HtmlView }
impl IPlugViewTrait for VstPlugView {
    unsafe fn isPlatformTypeSupported(&self, _type: FIDString) -> tresult { kResultOk }
    unsafe fn attached(&self, parent: *mut core::ffi::c_void, _type: FIDString) -> tresult { let parent_ptr = parent as isize; let url = util::bundle_resource_url(PLUGIN_NAME, "index.html"); let _ = self.inner.attached(parent_ptr, EDITOR_W, EDITOR_H, &url); kResultOk }
    unsafe fn removed(&self) -> tresult { kResultOk }
    unsafe fn onWheel(&self, _distance: f32) -> tresult { kResultOk }
    unsafe fn onKeyDown(&self, _key: char16, _keyCode: int16, _modifiers: int16) -> tresult { kResultOk }
    unsafe fn onKeyUp(&self, _key: char16, _keyCode: int16, _modifiers: int16) -> tresult { kResultOk }
    unsafe fn getSize(&self, size: *mut ViewRect) -> tresult { if let Some(out) = size.as_mut() { *out = ViewRect { left: 0, top: 0, right: EDITOR_W, bottom: EDITOR_H }; } kResultOk }
    unsafe fn onSize(&self, new_size: *mut ViewRect) -> tresult { if let Some(rect) = new_size.as_ref() { let w = rect.right - rect.left; let h = rect.bottom - rect.top; #[cfg(target_os = "windows")] { self.inner.set_bounds(w, h); } return kResultOk; } kInvalidArgument }
    unsafe fn onFocus(&self, _state: TBool) -> tresult { kResultOk }
    unsafe fn setFrame(&self, _frame: *mut IPlugFrame) -> tresult { kResultOk }
    unsafe fn canResize(&self) -> tresult { kResultOk }
    unsafe fn checkSizeConstraint(&self, _rect: *mut ViewRect) -> tresult { kResultOk }
}

#[cfg(target_os = "windows")]
mod windows_view {
    use windows::Win32::Foundation::HWND; use windows::core::PCWSTR;
    use webview2_com::Microsoft::Web::WebView2::Win32::{ CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2Environment, ICoreWebView2Controller, ICoreWebView2 };
    use serde_json::Value;
    pub struct HtmlView { pub parent_hwnd: HWND, pub env: Option<ICoreWebView2Environment>, pub controller: Option<ICoreWebView2Controller>, pub webview: Option<ICoreWebView2>, pub on_message: Option<Box<dyn FnMut(String)>> }
    impl Default for HtmlView { fn default() -> Self { Self { parent_hwnd: HWND(0), env: None, controller: None, webview: None, on_message: None } } }
    impl HtmlView {
        pub fn new() -> Self { Self::default() }
        pub fn attached(&mut self, parent: isize, width: i32, height: i32, url: &str) -> Result<(), String> {
            self.parent_hwnd = HWND(parent);
            let mut env: Option<ICoreWebView2Environment> = None;
            unsafe {
                CreateCoreWebView2EnvironmentWithOptions(None, None, None, &mut env).ok().map_err(|e| format!("env error: {e}"))?;
                let env = env.unwrap(); let mut controller: Option<ICoreWebView2Controller> = None;
                env.CreateCoreWebView2Controller(self.parent_hwnd, &mut controller).ok().map_err(|e| format!("controller error: {e}"))?;
                let controller = controller.unwrap(); let mut webview: Option<ICoreWebView2> = None;
                controller.get_CoreWebView2(&mut webview).ok().map_err(|e| format!("webview error: {e}"))?; let webview = webview.unwrap();
                controller.put_Bounds(windows::Win32::Foundation::RECT { left: 0, top: 0, right: width, bottom: height }).ok().ok();
                webview.Navigate(PCWSTR::from(&*windows::w!(url))).ok().ok();
                if let Some(mut cb) = self.on_message.take() {
                    let _ = webview.add_WebMessageReceived(webview2_com::callback!(move |_sender, args| {
                        if let Some(args) = args { let mut json: *mut u16 = std::ptr::null_mut(); unsafe { args.get_WebMessageAsJson(&mut json).ok().ok(); } if !json.is_null() { let msg = unsafe { windows::core::PWSTR(json).to_string().unwrap_or_default() }; cb(msg); } }
                        Ok(())
                    }));
                    self.on_message = Some(cb);
                }
                self.env = Some(env); self.controller = Some(controller); self.webview = Some(webview);
            }
            Ok(())
        }
        pub fn eval_set_gain(&self, norm: f64) { self.eval_js(&format!("window.setGainSlider({});", norm)); }
        pub fn eval_js(&self, js: &str) { if let Some(wv) = &self.webview { unsafe { wv.ExecuteScript(PCWSTR::from(&*windows::w!(js))).ok().ok(); } } }
        pub fn post_message(&self, _value: Value) {}
        pub fn set_message_handler<F: FnMut(String) + 'static>(&mut self, f: F) { self.on_message = Some(Box::new(f)); }
        pub fn set_bounds(&self, width: i32, height: i32) { if let Some(ctrl) = &self.controller { let _ = unsafe { ctrl.put_Bounds(windows::Win32::Foundation::RECT { left: 0, top: 0, right: width, bottom: height }) }; } }
    }
}

#[cfg(target_os = "macos")]
mod macos_view {
    use wry::{WebView, WebViewBuilder};
    use wry::raw_window_handle::{AppKitDisplayHandle, AppKitWindowHandle, HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};
    struct NsViewParent { ns_view: *mut core::ffi::c_void }
    impl HasRawWindowHandle for NsViewParent { fn raw_window_handle(&self) -> RawWindowHandle { let mut h = AppKitWindowHandle::empty(); h.ns_view = self.ns_view; RawWindowHandle::AppKit(h) } }
    impl HasRawDisplayHandle for NsViewParent { fn raw_display_handle(&self) -> RawDisplayHandle { RawDisplayHandle::AppKit(AppKitDisplayHandle::empty()) } }
    pub struct HtmlView { pub webview: Option<WebView>, pub on_message: Option<Box<dyn FnMut(String)>> }
    impl Default for HtmlView { fn default() -> Self { Self { webview: None, on_message: None } } }
    impl HtmlView {
        pub fn new() -> Self { Self::default() }
        pub fn attached(&mut self, nsview_ptr: isize, _width: i32, _height: i32, url: &str) -> Result<(), String> {
            let parent = NsViewParent { ns_view: nsview_ptr as *mut core::ffi::c_void };
            let mut builder = WebViewBuilder::new_as_child(&parent).with_url(url);
            if self.on_message.is_some() { let mut cb = self.on_message.take().unwrap(); builder = builder.with_ipc_handler(move |_win, arg| { cb(arg.to_string()); }); self.on_message = Some(cb); }
            let wv = builder.build().map_err(|e| format!("wry build error: {e}"))?; self.webview = Some(wv); Ok(())
        }
        pub fn eval_set_gain(&self, norm: f64) { self.eval_js(&format!("window.setGainSlider({});", norm)); }
        pub fn eval_js(&self, js: &str) { if let Some(wv) = &self.webview { let _ = wv.evaluate_script(js); } }
        pub fn post_message(&self, _value: serde_json::Value) {}
        pub fn set_message_handler<F: FnMut(String) + 'static>(&mut self, f: F) { self.on_message = Some(Box::new(f)); }
    }
}

#[cfg(target_os = "windows")]
pub use windows_view::HtmlView;
#[cfg(target_os = "macos")]
pub use macos_view::HtmlView;

pub fn send_example_ping(view: &HtmlView) { let payload = json!({"kind":"ping","message":"hello from Rust"}); let _ = payload; }
