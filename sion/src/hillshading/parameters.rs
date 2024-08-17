pub struct HillshadingParameters {
    pub sun_azimuth: f32,
    pub intensity: f32,
}

impl Default for HillshadingParameters {
    fn default() -> Self {
        Self {
            sun_azimuth: 315.0,
            intensity: 1.0,
        }
    }
}

impl HillshadingParameters {
    #[allow(dead_code)]
    fn new(sun_azimuth: f32, intensity: f32) -> Self {
        Self {
            sun_azimuth,
            intensity,
        }
    }
}
