
pub mod processor;
pub mod controller;
pub mod view;
pub mod shared;
pub mod util;

use core::ffi::c_void;
use vst3::Steinberg::*;
use std::sync::{Arc, OnceLock};
use crate::shared::SharedParams;

pub const PLUGIN_NAME: &str = "Vst3Skeleton";

static SHARED_PARAMS: OnceLock<Arc<SharedParams>> = OnceLock::new();
fn get_shared() -> Arc<SharedParams> {
    SHARED_PARAMS.get_or_init(|| Arc::new(SharedParams::new())).clone()
}

const PROCESSOR_CID: TUID = vst3::uid(0x13A5_7D21, 0x7B9C_4A10, 0x2C31_90EF, 0xA1B2_C3D4);
const CONTROLLER_CID: TUID = vst3::uid(0x89BC_12EF, 0x4D3A_77C2, 0x55AA_66BB, 0x99EE_FF00);

pub use PROCESSOR_CID as PROCESSOR_CLASS_ID;
pub use CONTROLLER_CID as CONTROLLER_CLASS_ID;

static FACTORY_INFO: PFactoryInfo = PFactoryInfo {
    vendor: *b"ExampleVendor\0" as *const u8 as *const i8,
    url:    *b"https://example.com\0" as *const u8 as *const i8,
    email:  *b"dev@example.com\0" as *const u8 as *const i8,
    flags: PFactoryInfo_kNoFlags as i32,
};

static CLASS_INFOS: &[PClassInfo2] = &[
    PClassInfo2 {
        cid: PROCESSOR_CID,
        cardinality: 1,
        category: *b"Audio Module Class\0" as *const u8 as *const i8,
        name: *b"Vst3Skeleton\0" as *const u8 as *const i8,
        classFlags: 0,
        subCategories: *b"Fx\0" as *const u8 as *const i8,
        vendor: *b"ExampleVendor\0" as *const u8 as *const i8,
        version: *b"0.1.0\0" as *const u8 as *const i8,
        sdkVersion: *b"3.8.0\0" as *const u8 as *const i8,
    },
    PClassInfo2 {
        cid: CONTROLLER_CID,
        cardinality: 1,
        category: *b"Component Controller Class\0" as *const u8 as *const i8,
        name: *b"Vst3SkeletonController\0" as *const u8 as *const i8,
        classFlags: 0,
        subCategories: *b"Fx\0" as *const u8 as *const i8,
        vendor: *b"ExampleVendor\0" as *const u8 as *const i8,
        version: *b"0.1.0\0" as *const u8 as *const i8,
        sdkVersion: *b"3.8.0\0" as *const u8 as *const i8,
    },
];

struct Factory;

impl IPluginFactoryTrait for Factory {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        if let Some(out) = info.as_mut() { *out = FACTORY_INFO; }
        kResultOk
    }
    unsafe fn countClasses(&self) -> int32 { CLASS_INFOS.len() as int32 }
    unsafe fn getClassInfo(&self, _index: int32, _info: *mut PClassInfo) -> tresult { kResultFalse }

    unsafe fn createInstance(&self, cid: *const TUID, _iid: *const TUID, obj: *mut *mut c_void) -> tresult {
        if obj.is_null() || cid.is_null() { return kInvalidArgument; }
        if *cid == PROCESSOR_CID {
            let instance = ComWrapper::new(crate::processor::AudioComponent::new(get_shared()));
            if let Ok(ptr) = instance.to_com_ptr::<IComponent>() { *obj = ptr.as_ptr() as *mut c_void; return kResultOk; }
        }
        if *cid == CONTROLLER_CID {
            let instance = ComWrapper::new(crate::controller::EditController::new(get_shared()));
            if let Ok(ptr) = instance.to_com_ptr::<IEditController>() { *obj = ptr.as_ptr() as *mut c_void; return kResultOk; }
        }
        kNoInterface
    }
}

impl IPluginFactory2Trait for Factory {
    unsafe fn getClassInfo2(&self, index: int32, info: *mut PClassInfo2) -> tresult {
        let idx = index as usize;
        if idx >= CLASS_INFOS.len() { return kInvalidArgument; }
        if let Some(out) = info.as_mut() { *out = CLASS_INFOS[idx]; }
        kResultOk
    }
}

#[no_mangle]
pub extern "C" fn GetPluginFactory() -> *mut c_void {
    let wrapper = ComWrapper::new(Factory);
    match wrapper.to_com_ptr::<IPluginFactory>() { Ok(ptr) => ptr.as_ptr() as *mut c_void, Err(_) => core::ptr::null_mut() }
}
