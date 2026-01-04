
use vst3::Steinberg::*;
use std::sync::Arc;
use crate::shared::SharedParams;
use crate::lib::CONTROLLER_CLASS_ID as CONTROLLER_CID;

pub struct AudioComponent { shared: Arc<SharedParams> }
impl AudioComponent { pub fn new(shared: Arc<SharedParams>) -> Self { Self { shared } } }

impl IPluginBaseTrait for AudioComponent {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult { kResultOk }
    unsafe fn terminate(&self) -> tresult { kResultOk }
}

impl IComponentTrait for AudioComponent {
    unsafe fn getControllerClassId(&self, class_id: *mut TUID) -> tresult { if let Some(cid) = class_id.as_mut() { *cid = CONTROLLER_CID; } kResultOk }
    unsafe fn setIoMode(&self, _mode: IoMode) -> tresult { kResultOk }
    unsafe fn getBusCount(&self, _direction: MediaTypes, _type: BusTypes) -> int32 { 1 }
    unsafe fn getBusInfo(&self, direction: MediaTypes, _type: BusTypes, index: int32, info: *mut BusInfo) -> tresult {
        if index != 0 { return kInvalidArgument; }
        if let Some(out) = info.as_mut() {
            out.mediaType = direction;
            out.direction = if direction == MediaTypes_kAudio { BusDirections_kInput } else { BusDirections_kOutput };
            out.channelCount = 2; out.busType = BusTypes_kMain;
            out.name = *b"Main\0" as *const u8 as *const i8; out.flags = BusInfo_kDefaultActive as int32;
        }
        kResultOk
    }
    unsafe fn initializeFUnknown(&self, _context: *mut FUnknown) -> tresult { kResultOk }
}

impl IAudioProcessorTrait for AudioComponent {
    unsafe fn setBusArrangements(&self, _inputs: *mut SpeakerArrangement, _numIns: int32, _outputs: *mut SpeakerArrangement, _numOuts: int32) -> tresult { kResultOk }
    unsafe fn setupProcessing(&self, _setup: *const ProcessSetup) -> tresult { kResultOk }
    unsafe fn setProcessing(&self, _state: TBool) -> tresult { kResultOk }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        if data.is_null() { return kInvalidArgument; }
        let pd = &mut *data;
        // normalized -> dB -> linear amplitude
        let norm = self.shared.gain_normalized();
        let min_db = -60.0f64; let max_db = 12.0f64;
        let db = min_db + norm * (max_db - min_db);
        let gain = 10.0f32.powf((db as f32) / 20.0);

        if pd.numInputs >= 1 && pd.numOutputs >= 1 {
            let input  = &pd.inputs[0];
            let output = &mut pd.outputs[0];
            if !input.channelBuffers32.is_null() && !output.channelBuffers32.is_null() {
                let in_bufs  = std::slice::from_raw_parts(input.channelBuffers32,  input.numChannels as usize);
                let out_bufs = std::slice::from_raw_parts_mut(output.channelBuffers32, output.numChannels as usize);
                let nframes  = pd.numSamples as usize;

                let mut peak_l: f32 = 0.0; let mut peak_r: f32 = 0.0;
                let mut sumsq_l: f64 = 0.0; let mut sumsq_r: f64 = 0.0;

                for ch in 0..std::cmp::min(in_bufs.len(), out_bufs.len()) {
                    let in_ch  = std::slice::from_raw_parts(in_bufs[ch],  nframes);
                    let out_ch = std::slice::from_raw_parts_mut(out_bufs[ch], nframes);
                    for i in 0..nframes {
                        let y = in_ch[i] * gain;
                        out_ch[i] = y;
                        let ay = y.abs();
                        if ch == 0 { if ay > peak_l { peak_l = ay; } sumsq_l += (y as f64) * (y as f64); }
                        else if ch == 1 { if ay > peak_r { peak_r = ay; } sumsq_r += (y as f64) * (y as f64); }
                    }
                }
                // Compute RMS and Peak in dBFS (avoid -inf with EPS)
                let eps = 1e-12f64;
                let rms_l = (sumsq_l / (nframes.max(1) as f64)).sqrt();
                let rms_r = (sumsq_r / (nframes.max(1) as f64)).sqrt();
                let peak_l_db = 20.0f64 * (peak_l.max(eps) as f64).log10();
                let peak_r_db = 20.0f64 * (peak_r.max(eps) as f64).log10();
                let rms_l_db  = 20.0f64 * rms_l.max(eps).log10();
                let rms_r_db  = 20.0f64 * rms_r.max(eps).log10();
                self.shared.set_peak_l_db(peak_l_db);
                self.shared.set_peak_r_db(peak_r_db);
                self.shared.set_rms_l_db(rms_l_db);
                self.shared.set_rms_r_db(rms_r_db);
            }
        }
        kResultOk
    }
    unsafe fn canProcessSampleSize(&self, _symbolic: SymbolicSampleSizes) -> tresult { kResultOk }
    unsafe fn getLatencySamples(&self) -> int32 { 0 }
}
