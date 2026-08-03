import React from 'react';

interface Props {
  onGenerate: (params: { seed: number; radius_m: number; mass_kg: number; stellar_class?: string; initial_water_fraction?: number }) => void;
  onTick: (count: number) => void;
  onExport: () => string;
  onImport: (json: string) => void;
  tickCount: number;
  speed: number;
  setSpeed: (v: number) => void;
  loading: boolean;
}

const Sidebar: React.FC<Props> = ({ onGenerate, onTick, onExport, onImport, tickCount, speed, setSpeed, loading }) => {
  const [seed, setSeed] = React.useState(42);
  const [radius, setRadius] = React.useState(6_371_000);
  const [mass, setMass] = React.useState(5.972e24);
  const [water, setWater] = React.useState(0.7);
  const [json, setJson] = React.useState('');

  const handleGenerate = () => {
    onGenerate({ seed, radius_m: radius, mass_kg: mass, initial_water_fraction: water });
  };

  const handleExport = () => {
    const data = onExport();
    setJson(data);
  };

  const handleImport = () => {
    if (!json.trim()) return;
    onImport(json);
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-section">
        <h2>Simulation</h2>
        <div className="field">
          <label>Seed</label>
          <input type="number" value={seed} onChange={e => setSeed(Number(e.target.value))} />
        </div>
        <div className="field">
          <label>Radius (m)</label>
          <input type="number" value={radius} onChange={e => setRadius(Number(e.target.value))} />
        </div>
        <div className="field">
          <label>Mass (kg)</label>
          <input type="number" value={mass} onChange={e => setMass(Number(e.target.value))} />
        </div>
        <div className="field">
          <label>Water fraction</label>
          <input type="range" min={0} max={1} step={0.01} value={water} onChange={e => setWater(Number(e.target.value))} />
          <span className="field-value">{water.toFixed(2)}</span>
        </div>
        <button className="primary-btn" onClick={handleGenerate} disabled={loading}>
          {loading ? 'Running...' : 'Generate Planet'}
        </button>
      </div>

      <div className="sidebar-section">
        <h2>Evolution</h2>
        <div className="field">
          <label>Speed</label>
          <select value={speed} onChange={e => setSpeed(Number(e.target.value))}>
            <option value={1}>1x</option>
            <option value={10}>10x</option>
            <option value={100}>100x</option>
          </select>
        </div>
        <button className="primary-btn" onClick={() => onTick(1)} disabled={loading}>Tick</button>
        <div className="metric">Tick count: {tickCount}</div>
      </div>

      <div className="sidebar-section">
        <h2>State</h2>
        <button className="secondary-btn" onClick={handleExport}>Export JSON</button>
        <textarea
          className="json-area"
          value={json}
          onChange={e => setJson(e.target.value)}
          placeholder="Simulation JSON will appear here..."
        />
        <button className="secondary-btn" onClick={handleImport}>Import JSON</button>
      </div>
    </aside>
  );
};

export default Sidebar;
