#[cfg(feature = "lepton")]
pub mod lepton;

pub fn register_image_types() {
    #[cfg(feature = "lepton")]
    lepton::register();
}