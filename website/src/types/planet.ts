export interface Planet {
  id: string;
  name: string;
  class: string;
  planet_type: string;
  radius_m: number;
  mass_kg: number;
  gravity_m_s2?: number;
  stellar_class?: string;
  temperature_k?: number;
  pressure_pa?: number;
  water_fraction?: number;
  ice_fraction?: number;
  atmospheric_mass_kg?: number;
  mean_temperature_k?: number;
  equilibrium_temperature_k?: number;
  planetary_albedo?: number;
  habitability_index?: number;
  habitability_class?: string;
  primary_classification?: string;
  secondary_classification?: string;
  confidence_score?: number;
  classification_summary?: string;
  age_seconds?: number;
  tick?: number;
}

export interface Snapshot {
  simulation_id: string;
  timestamp_s: number;
  tick: number;
  planets: Planet[];
}
