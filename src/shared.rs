
use std::sync::atomic::{AtomicU64, Ordering};
#[derive(Default)]
pub struct SharedParams {
    gain_bits: AtomicU64,       // normalized [0..1]
    peak_l_db_bits: AtomicU64,  // dBFS
    peak_r_db_bits: AtomicU64,
    rms_l_db_bits: AtomicU64,
    rms_r_db_bits: AtomicU64,
}
impl SharedParams {
    pub fn new() -> Self {
        Self {
            gain_bits: AtomicU64::new((1.0f64).to_bits()),
            peak_l_db_bits: AtomicU64::new((-120.0f64).to_bits()),
            peak_r_db_bits: AtomicU64::new((-120.0f64).to_bits()),
            rms_l_db_bits: AtomicU64::new((-120.0f64).to_bits()),
            rms_r_db_bits: AtomicU64::new((-120.0f64).to_bits()),
        }
    }
    pub fn set_gain_normalized(&self, norm: f64) { self.gain_bits.store(norm.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed); }
    pub fn gain_normalized(&self) -> f64 { f64::from_bits(self.gain_bits.load(Ordering::Relaxed)) }
    pub fn set_peak_l_db(&self, db: f64) { self.peak_l_db_bits.store(db.to_bits(), Ordering::Relaxed); }
    pub fn set_peak_r_db(&self, db: f64) { self.peak_r_db_bits.store(db.to_bits(), Ordering::Relaxed); }
    pub fn set_rms_l_db(&self, db: f64) { self.rms_l_db_bits.store(db.to_bits(), Ordering::Relaxed); }
    pub fn set_rms_r_db(&self, db: f64) { self.rms_r_db_bits.store(db.to_bits(), Ordering::Relaxed); }
    pub fn peak_l_db(&self) -> f64 { f64::from_bits(self.peak_l_db_bits.load(Ordering::Relaxed)) }
    pub fn peak_r_db(&self) -> f64 { f64::from_bits(self.peak_r_db_bits.load(Ordering::Relaxed)) }
    pub fn rms_l_db(&self) -> f64 { f64::from_bits(self.rms_l_db_bits.load(Ordering::Relaxed)) }
    pub fn rms_r_db(&self) -> f64 { f64::from_bits(self.rms_r_db_bits.load(Ordering::Relaxed)) }
}
