#[derive(Debug, Clone, Copy)]
pub struct MotorModelSpec {
    pub vendor: &'static str,
    pub model: &'static str,
    pub pmax: f32,
    pub vmax: f32,
    pub tmax: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct PvTLimits {
    pub p_min: f32,
    pub p_max: f32,
    pub v_min: f32,
    pub v_max: f32,
    pub t_min: f32,
    pub t_max: f32,
}

impl PvTLimits {
    pub fn from_spec(spec: &MotorModelSpec) -> Self {
        Self {
            p_min: -spec.pmax,
            p_max: spec.pmax,
            v_min: -spec.vmax,
            v_max: spec.vmax,
            t_min: -spec.tmax,
            t_max: spec.tmax,
        }
    }
}

pub trait ModelCatalog: Send + Sync {
    fn vendor(&self) -> &'static str;
    fn get(&self, model: &str) -> Option<&'static MotorModelSpec>;
}

pub struct StaticModelCatalog {
    pub vendor_name: &'static str,
    pub models: &'static [MotorModelSpec],
}

impl ModelCatalog for StaticModelCatalog {
    fn vendor(&self) -> &'static str {
        self.vendor_name
    }

    fn get(&self, model: &str) -> Option<&'static MotorModelSpec> {
        self.models.iter().find(|spec| spec.model == model)
    }
}
