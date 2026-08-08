import React from 'react';
import { formatNumber, formatMeters, formatKelvin, formatPressure, formatFraction, formatMass, formatAge, formatTick, formatIndex } from '../utils/format';

interface PlanetState {
  id?: string;
  name?: string;
  class?: string;
  planet_type?: string;
  radius_m?: number;
  mass_kg?: number;
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

interface Props {
  planet?: PlanetState;
  snapshot?: { simulation_id?: string; timestamp_s?: number; tick?: number; planets?: PlanetState[] };
  status?: 'idle' | 'generating' | 'ready';
  pipeline?: string[];
}

const Row = ({ label, children }: { label: string; children: React.ReactNode }) => (
  <div className="info-row">
    <span className="info-label">{label}</span>
    <span className="info-value">{children}</span>
  </div>
);

const fmt = (label: string, value?: React.ReactNode) => <Row label={label}>{value ?? '—'}</Row>;

const Section = ({ title, children }: { title: string; children: React.ReactNode }) => (
  <div className="info-block">
    <h3>{title}</h3>
    {children}
  </div>
);

const StatusIndicator: React.FC<{ status?: 'idle' | 'generating' | 'ready'; tick?: number; seed?: number; planetClass?: string }> = ({ status, tick, seed, planetClass }) => {
  if (status === 'idle' || !status) {
    return (
      <div className="status-block">
        <div className="status-row">
          <span className="status-dot idle" aria-hidden="true" />
          <span className="status-label">Ready</span>
        </div>
        <p className="empty-state">Awaiting simulation.</p>
      </div>
    );
  }
  if (status === 'generating') {
    return (
      <div className="status-block">
        <div className="status-row">
          <span className="status-dot active" aria-hidden="true" />
          <span className="status-label">Generating...</span>
        </div>
      </div>
    );
  }
  return (
    <div className="status-block">
      <div className="status-row">
        <span className="status-dot success" aria-hidden="true" />
        <span className="status-label">Simulation Complete</span>
      </div>
      <div className="status-metrics">
        {typeof tick !== 'undefined' && <Row label="Current Tick">{formatTick(tick)}</Row>}
        {typeof seed !== 'undefined' && <Row label="Seed">{formatNumber(seed, 0)}</Row>}
        {planetClass && <Row label="Planet Class">{planetClass}</Row>}
      </div>
    </div>
  );
};

const ModuleInspector: React.FC<{ planet?: PlanetState }> = ({ planet }) => {
  if (!planet) return null;
  const modules: { title: string; rows: { label: string; value: React.ReactNode }[] }[] = [];

  const coreRows: { label: string; value: React.ReactNode }[] = [];
  if (typeof planet.radius_m !== 'undefined') coreRows.push({ label: 'Radius', value: formatMeters(planet.radius_m) });
  if (typeof planet.mass_kg !== 'undefined') coreRows.push({ label: 'Mass', value: formatMass(planet.mass_kg) });
  if (typeof planet.gravity_m_s2 !== 'undefined') coreRows.push({ label: 'Gravity', value: `${formatNumber(planet.gravity_m_s2, 3)} m/s²` });
  if (typeof planet.age_seconds !== 'undefined') coreRows.push({ label: 'Age', value: formatAge(planet.age_seconds) });
  if (coreRows.length) modules.push({ title: 'Core', rows: coreRows });

  const mantleRows: { label: string; value: React.ReactNode }[] = [];
  if (typeof planet.pressure_pa !== 'undefined') mantleRows.push({ label: 'Surface Pressure', value: formatPressure(planet.pressure_pa) });
  if (typeof planet.temperature_k !== 'undefined') mantleRows.push({ label: 'Surface Temperature', value: formatKelvin(planet.temperature_k) });
  if (typeof planet.mean_temperature_k !== 'undefined') mantleRows.push({ label: 'Mean Temperature', value: formatKelvin(planet.mean_temperature_k) });
  if (typeof planet.equilibrium_temperature_k !== 'undefined') mantleRows.push({ label: 'Equilibrium Temperature', value: formatKelvin(planet.equilibrium_temperature_k) });
  if (typeof planet.planetary_albedo !== 'undefined') mantleRows.push({ label: 'Albedo', value: formatIndex(planet.planetary_albedo, 4) });
  if (mantleRows.length) modules.push({ title: 'Atmosphere / Surface', rows: mantleRows });

  const hydroRows: { label: string; value: React.ReactNode }[] = [];
  if (typeof planet.water_fraction !== 'undefined') hydroRows.push({ label: 'Water Fraction', value: formatFraction(planet.water_fraction, 2) });
  if (typeof planet.ice_fraction !== 'undefined') hydroRows.push({ label: 'Ice Fraction', value: formatFraction(planet.ice_fraction, 2) });
  if (typeof planet.atmospheric_mass_kg !== 'undefined') hydroRows.push({ label: 'Atmospheric Mass', value: formatMass(planet.atmospheric_mass_kg) });
  if (hydroRows.length) modules.push({ title: 'Hydrology', rows: hydroRows });

  const climateRows: { label: string; value: React.ReactNode }[] = [];
  if (typeof planet.equilibrium_temperature_k !== 'undefined') climateRows.push({ label: 'Equilibrium Temp', value: formatKelvin(planet.equilibrium_temperature_k) });
  if (typeof planet.mean_temperature_k !== 'undefined') climateRows.push({ label: 'Mean Temp', value: formatKelvin(planet.mean_temperature_k) });
  if (typeof planet.planetary_albedo !== 'undefined') climateRows.push({ label: 'Albedo', value: formatIndex(planet.planetary_albedo, 4) });
  if (planet.habitability_class && climateRows.length === 0) climateRows.push({ label: 'Climate Class', value: planet.habitability_class });
  if (climateRows.length) modules.push({ title: 'Climate', rows: climateRows });

  const habitabilityRows: { label: string; value: React.ReactNode }[] = [];
  if (typeof planet.habitability_index !== 'undefined') habitabilityRows.push({ label: 'Habitability Index', value: formatIndex(planet.habitability_index, 3) });
  if (planet.habitability_class) habitabilityRows.push({ label: 'Habitability Class', value: planet.habitability_class });
  if (typeof planet.confidence_score !== 'undefined') habitabilityRows.push({ label: 'Confidence', value: formatIndex(planet.confidence_score, 3) });
  if (planet.classification_summary) habitabilityRows.push({ label: 'Summary', value: planet.classification_summary });
  if (habitabilityRows.length) modules.push({ title: 'Classification / Habitability', rows: habitabilityRows });

  if (!modules.length) return null;

  return (
    <div className="info-block">
      <h3>Simulation Systems</h3>
      {modules.map(module => (
        <details key={module.title} className="module-details">
          <summary className="module-summary">{module.title}</summary>
          <div className="module-rows">
            {module.rows.map(row => (
              <Row key={row.label} label={row.label}>{row.value}</Row>
            ))}
          </div>
        </details>
      ))}
    </div>
  );
};

const Pipeline: React.FC<{ steps: string[] }> = ({ steps }) => (
  <div className="pipeline" aria-live="polite">
    {steps.map((step, idx) => (
      <div key={idx} className={`pipeline-step ${step === 'Initializing...' ? 'active' : 'done'}`}>
        <span className="pipeline-marker">{step === 'Initializing...' ? '•' : '✓'}</span>
        <span className="pipeline-text">{step}</span>
      </div>
    ))}
  </div>
);

const InfoPanel: React.FC<Props> = ({ planet, snapshot, status = 'idle', pipeline = [] }) => {
  return (
    <aside className="info-panel">
      <h2>Explorer</h2>
      <StatusIndicator
        status={status}
        tick={planet?.tick}
        seed={planet ? Number((planet.id ?? '').replace(/\D/g, '')) || undefined : undefined}
        planetClass={planet?.primary_classification}
      />
      {status === 'generating' && pipeline.length > 0 && <Pipeline steps={pipeline} />}

      {planet && (
        <>
          <Section title="World">
            {planet.name && <Row label="Name">{planet.name}</Row>}
            {typeof planet.tick !== 'undefined' && <Row label="Current Tick">{formatTick(planet.tick)}</Row>}
            <Row label="Planet Classification">{planet.class}</Row>
            <Row label="Planet Type">{planet.planet_type}</Row>
            {planet.stellar_class && <Row label="Stellar Class">{planet.stellar_class}</Row>}
            {typeof planet.habitability_index !== 'undefined' && <Row label="Habitability">{formatIndex(planet.habitability_index, 3)}</Row>}
            {planet.habitability_class && <Row label="Habitability Class">{planet.habitability_class}</Row>}
          </Section>

          <Section title="Physical">
            {typeof planet.radius_m !== 'undefined' && <Row label="Radius">{formatMeters(planet.radius_m)}</Row>}
            {typeof planet.mass_kg !== 'undefined' && <Row label="Mass">{formatMass(planet.mass_kg)}</Row>}
            {typeof planet.gravity_m_s2 !== 'undefined' && <Row label="Gravity">{formatNumber(planet.gravity_m_s2, 3)} m/s²</Row>}
            {typeof planet.temperature_k !== 'undefined' && <Row label="Temperature">{formatKelvin(planet.temperature_k)}</Row>}
          </Section>

          <Section title="Atmosphere">
            {typeof planet.pressure_pa !== 'undefined' && <Row label="Surface Pressure">{formatPressure(planet.pressure_pa)}</Row>}
            {typeof planet.mean_temperature_k !== 'undefined' && <Row label="Mean Temperature">{formatKelvin(planet.mean_temperature_k)}</Row>}
            {typeof planet.equilibrium_temperature_k !== 'undefined' && <Row label="Equilibrium Temp">{formatKelvin(planet.equilibrium_temperature_k)}</Row>}
            {typeof planet.atmospheric_mass_kg !== 'undefined' && <Row label="Atmospheric Mass">{formatMass(planet.atmospheric_mass_kg)}</Row>}
            {typeof planet.planetary_albedo !== 'undefined' && <Row label="Albedo">{formatIndex(planet.planetary_albedo, 4)}</Row>}
          </Section>

          <Section title="Hydrology">
            {typeof planet.water_fraction !== 'undefined' && <Row label="Water Fraction">{formatFraction(planet.water_fraction, 2)}</Row>}
            {typeof planet.ice_fraction !== 'undefined' && <Row label="Ice Fraction">{formatFraction(planet.ice_fraction, 2)}</Row>}
          </Section>

          <Section title="Climate">
            {typeof planet.mean_temperature_k !== 'undefined' && <Row label="Mean Temperature">{formatKelvin(planet.mean_temperature_k)}</Row>}
            {typeof planet.equilibrium_temperature_k !== 'undefined' && <Row label="Equilibrium Temperature">{formatKelvin(planet.equilibrium_temperature_k)}</Row>}
            {typeof planet.planetary_albedo !== 'undefined' && <Row label="Albedo">{formatIndex(planet.planetary_albedo, 4)}</Row>}
          </Section>

          <ModuleInspector planet={planet} />

          {snapshot && (
            <Section title="Snapshot">
              {fmt('Simulation', snapshot.simulation_id)}
              {fmt('Timestamp', typeof snapshot.timestamp_s === 'number' ? `${formatNumber(snapshot.timestamp_s, 0)} s` : undefined)}
              {fmt('Tick', snapshot.tick)}
            </Section>
          )}
        </>
      )}
    </aside>
  );
};

export default InfoPanel;
