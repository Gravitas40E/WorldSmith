import React from 'react';

interface PlanetState {
  id?: string;
  name?: string;
  class?: string;
  planet_type?: string;
  radius_m?: number;
  mass_kg?: number;
  gravity_m_s2?: number;
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

interface Props {
  planet?: PlanetState;
  snapshot?: { simulation_id?: string; timestamp_s?: number; tick?: number; planets?: PlanetState[] };
}

const fmt = (label: string, value?: number | string | null) => (
  <div className="info-row">
    <span className="info-label">{label}</span>
    <span className="info-value">{value ?? '—'}</span>
  </div>
);

const InfoPanel: React.FC<Props> = ({ planet, snapshot }) => {
  if (!planet) {
    return (
      <aside className="info-panel">
        <h2>Planet Data</h2>
        <p className="empty-state">Generate a planet to view telemetry.</p>
      </aside>
    );
  }

  return (
    <aside className="info-panel">
      <h2>Planet Data</h2>
      {snapshot && (
        <div className="info-block">
          <h3>Snapshot</h3>
          {fmt('Simulation', snapshot.simulation_id)}
          {fmt('Timestamp', snapshot.timestamp_s)}
          {fmt('Tick', snapshot.tick)}
        </div>
      )}
      <div className="info-block">
        <h3>Identity</h3>
        {fmt('Name', planet.name)}
        {fmt('Class', planet.class)}
        {fmt('Type', planet.planet_type)}
        {fmt('Primary', planet.primary_classification)}
        {fmt('Secondary', planet.secondary_classification)}
      </div>
      <div className="info-block">
        <h3>Physical</h3>
        {fmt('Radius (m)', planet.radius_m)}
        {fmt('Mass (kg)', planet.mass_kg)}
        {fmt('Gravity (m/s²)', planet.gravity_m_s2)}
        {fmt('Age (s)', planet.age_seconds)}
      </div>
      <div className="info-block">
        <h3>Climate</h3>
        {fmt('Temp (K)', planet.temperature_k)}
        {fmt('Eq. Temp (K)', planet.equilibrium_temperature_k)}
        {fmt('Mean Temp (K)', planet.mean_temperature_k)}
        {fmt('Pressure (Pa)', planet.pressure_pa)}
        {fmt('Albedo', planet.planetary_albedo)}
      </div>
      <div className="info-block">
        <h3>Hydrosphere</h3>
        {fmt('Water', planet.water_fraction)}
        {fmt('Ice', planet.ice_fraction)}
      </div>
      <div className="info-block">
        <h3>Habitability</h3>
        {fmt('Index', planet.habitability_index)}
        {fmt('Class', planet.habitability_class)}
        {fmt('Summary', planet.classification_summary)}
      </div>
    </aside>
  );
};

export default InfoPanel;
