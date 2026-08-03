mod engine;
mod model;

use engine::WorldSmithEngine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct Explorer {
    engine: WorldSmithEngine,
}

#[wasm_bindgen]
impl Explorer {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> Result<Explorer, JsValue> {
        Ok(Self {
            engine: WorldSmithEngine::new(seed).map_err(|e| JsValue::from(format!("{e:?}")))?,
        })
    }

    pub fn generate_planet(
        &mut self,
        seed: u64,
        radius_m: f64,
        mass_kg: f64,
        stellar_class: Option<String>,
        initial_water_fraction: Option<f64>,
    ) -> Result<JsValue, JsValue> {
        let planet = self
            .engine
            .generate_planet(
                seed,
                radius_m,
                mass_kg,
                stellar_class,
                initial_water_fraction,
            )
            .map_err(JsValue::from)?;
        serde_wasm_bindgen::to_value(&planet).map_err(|e| JsValue::from(format!("{e:?}")))
    }

    pub fn tick(&mut self, ticks: u32) -> Result<JsValue, JsValue> {
        let snapshot = self.engine.tick(ticks).map_err(JsValue::from)?;
        serde_wasm_bindgen::to_value(&snapshot).map_err(|e| JsValue::from(format!("{e:?}")))
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        let snapshot = self.engine.snapshot().map_err(JsValue::from)?;
        serde_wasm_bindgen::to_value(&snapshot).map_err(|e| JsValue::from(format!("{e:?}")))
    }

    pub fn export_json(&self) -> Result<String, JsValue> {
        self.engine.export_json().map_err(JsValue::from)
    }

    pub fn import_json(&mut self, json: &str) -> Result<JsValue, JsValue> {
        let planet = self.engine.import_json(json).map_err(JsValue::from)?;
        serde_wasm_bindgen::to_value(&planet).map_err(|e| JsValue::from(format!("{e:?}")))
    }

    pub fn planet_state(&self) -> Result<JsValue, JsValue> {
        let planet = self.engine.planet_state().map_err(JsValue::from)?;
        serde_wasm_bindgen::to_value(&planet).map_err(|e| JsValue::from(format!("{e:?}")))
    }
}
