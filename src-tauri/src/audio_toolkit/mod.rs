//! Audio senza Tauri: cattura, ricampionamento, VAD, segmentazione, decodifica, mixer, writer.

pub mod capture;
pub mod decode;
pub mod forma_onda;
pub mod mixer;
pub mod ogg_opus;
pub mod resample;
pub mod segmenter;
pub mod vad;
