import { Explorer } from '../lib/wasm/worldsmith_web';

export interface PlanetParams {
  seed: number;
  radius_m: number;
  mass_kg: number;
  stellar_class?: string;
  initial_water_fraction?: number;
}

export interface PlanetState {
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
  planets: PlanetState[];
}

export class WorldSmithClient {
  private explorer: Explorer | null = null;

  init(seed: number): void {
    this.explorer = new Explorer(BigInt(seed));
  }

  generatePlanet(params: PlanetParams): PlanetState {
    if (!this.explorer) throw new Error('Client not initialized');
    return this.explorer.generate_planet(
      BigInt(params.seed),
      params.radius_m,
      params.mass_kg,
      params.stellar_class ?? null,
      params.initial_water_fraction ?? null
    );
  }

  tick(count: number): Snapshot {
    if (!this.explorer) throw new Error('Client not initialized');
    return this.explorer.tick(count);
  }

  snapshot(): Snapshot {
    if (!this.explorer) throw new Error('Client not initialized');
    return this.explorer.snapshot();
  }

  exportJSON(): string {
    if (!this.explorer) throw new Error('Client not initialized');
    return this.explorer.export_json();
  }

  importJSON(json: string): PlanetState {
    if (!this.explorer) throw new Error('Client not initialized');
    return this.explorer.import_json(json);
  }

  getPlanetState(): PlanetState {
    if (!this.explorer) throw new Error('Client not initialized');
    return this.explorer.planet_state();
  }
}
