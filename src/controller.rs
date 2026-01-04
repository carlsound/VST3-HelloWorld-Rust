
use vst3::Steinberg::*;
use vst3::Steinberg::ComPtr;
use serde_json::json;
use std::sync::Arc;
use crate::shared::SharedParams;
use crate::view::{HtmlView, VstPlugView};
use crate::util; use crate::lib::PLUGIN_NAME;
use crate::lib::PROCESSOR_CLASS_ID as PROCESSOR_CID;

pub const GAIN_PARAM_ID: u32 = 1;

pub struct EditController {
    pub shared: Arc<SharedParams>,
    pub gain_norm: f64,
    pub view: Option<HtmlView>,
    pub handler: Option<ComPtr<IComponentHandler>>,
}

impl EditController {
    pub fn new(shared: Arc<SharedParams>) -> Self { Self { shared, gain_norm: 0.8333333, view: None, handler: None } } // ~0 dB default
    pub fn push_gain_to_ui(&self) { if let Some(view) = &self.view { view.eval_set_gain(self.gain_norm); } }
}

impl IPluginBaseTrait for EditController { unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult { kResultOk } unsafe fn terminate(&self) -> tresult { kResultOk } }

impl IEditControllerTrait for EditController {
    unsafe fn setComponentState(&self, _state: *mut IBStream) -> tresult { kResultOk }
    unsafe fn getComponentState(&self, _state: *mut IBStream) -> tresult { kResultOk }

    unsafe fn getParameterCount(&self) -> int32 { 1 }
    unsafe fn getParameterInfo(&self, tag: int32, info: *mut ParameterInfo) -> tresult {
        if tag != 0 { return kInvalidArgument; }
        if let Some(out) = info.as_mut() {
            out.id = GAIN_PARAM_ID;
            out.title = *b"Gain\0" as *const u8 as *const i8;
            out.units = *b"dB\0" as *const u8 as *const i8;
            out.stepCount = 0; // continuous
            out.defaultNormalizedValue = 0.8333333; // 0 dB
            out.flags = (ParameterInfo_kCanAutomate | ParameterInfo_kIsProgramChange as int32) as int32;
        }
        kResultOk
    }

    unsafe fn getParamStringByValue(&self, tag: ParamID, value_normalized: ParamValue, string: *mut String128) -> tresult {
        if tag != GAIN_PARAM_ID { return kInvalidArgument; }
        let min_db = -60.0; let max_db = 12.0; let db = min_db + value_normalized * (max_db - min_db);
        if let Some(out) = string.as_mut() {
            let s = format!("{:+.2} dB\0", db);
            let bytes = s.as_bytes(); out[..bytes.len()].copy_from_slice(bytes);
        }
        kResultOk
    }

    unsafe fn setParamNormalized(&self, tag: ParamID, value: ParamValue) -> tresult {
        if tag == GAIN_PARAM_ID {
            let this = self as *const _ as *mut EditController;
            if let Some(this_mut) = this.as_mut() { this_mut.gain_norm = value; this_mut.shared.set_gain_normalized(value); this_mut.push_gain_to_ui(); }
            return kResultOk;
        }
        kInvalidArgument
    }
    unsafe fn getParamNormalized(&self, tag: ParamID) -> ParamValue { if tag == GAIN_PARAM_ID { self.gain_norm } else { 0.0 } }

    unsafe fn setComponentHandler(&self, handler: *mut IComponentHandler) -> tresult {
        let this = self as *const _ as *mut EditController;
        if let Some(this_mut) = this.as_mut() { if let Some(h) = handler.as_mut() { this_mut.handler = Some(ComPtr::from_raw(h)); } }
        kResultOk
    }

    unsafe fn getComponentClassId(&self, class_id: *mut TUID) -> tresult { if let Some(cid) = class_id.as_mut() { *cid = PROCESSOR_CID; } kResultOk }

    unsafe fn createView(&self, _name: *const i8) -> *mut IPlugView {
        let mut html = HtmlView::new();
        html.set_message_handler({ let this_ptr = self as *const _ as *mut EditController; move |msg| {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg) {
                let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                let pid = v.get("param_id").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
                let val = v.get("value").and_then(|p| p.as_f64()).unwrap_or(0.0);
                unsafe if let Some(this) = this_ptr.as_mut() {
                    match kind {
                        "begin" => { if let Some(h) = &this.handler { let _ = h.beginEdit(GAIN_PARAM_ID); } },
                        "perform" => {
                            if pid == GAIN_PARAM_ID {
                                this.gain_norm = val; this.shared.set_gain_normalized(val);
                                if let Some(h) = &this.handler { let _ = h.performEdit(GAIN_PARAM_ID, val); }
                                this.push_gain_to_ui();
                            }
                        },
                        "end" => { if let Some(h) = &this.handler { let _ = h.endEdit(GAIN_PARAM_ID); } },
                        "poll_meters" => {
                            let js = format!(
                                "window.updateMeters({{peakL:{:.2}, peakR:{:.2}, rmsL:{:.2}, rmsR:{:.2}}});",
                                this.shared.peak_l_db(), this.shared.peak_r_db(), this.shared.rms_l_db(), this.shared.rms_r_db()
                            );
                            if let Some(view) = &this.view { view.eval_js(&js); }
                        },
                        _ => {}
                    }
                }
            }
        } });
        let this = self as *const _ as *mut EditController; if let Some(this_mut) = this.as_mut() { this_mut.view = Some(html); }
        let view_obj = VstPlugView { inner: self.view.clone().unwrap() };
        let wrapper = ComWrapper::new(view_obj);
        match wrapper.to_com_ptr::<IPlugView>() { Ok(ptr) => ptr.as_ptr(), Err(_) => core::ptr::null_mut() }
    }
}
