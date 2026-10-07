//! Audio senza Tauri: cattura, ricampionamento, VAD, segmentazione, decodifica, mixer, writer.

pub mod capture;
pub mod cleaning;
pub mod decode;
pub mod deepfilter;
pub mod forma_onda;
pub mod mixer;
pub mod ogg_opus;
pub mod processing;
pub mod protection;
pub mod resample;
pub mod segmenter;
pub mod vad;
