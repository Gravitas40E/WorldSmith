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

const Sidebar = React.forwardRef<HTMLElement, Props>((props, ref) => {
  const {
    onGenerate,
    onTick,
    onExport,
    onImport,
    tickCount,
    speed,
    setSpeed,
    loading,
  } = props;

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
    <aside className="sidebar" ref={ref}>
      <div className="sidebar-header">
        <div className="brand">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" width="16" height="16">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            <path d="M2 12h20" />
          </svg>
          <span>WorldSmith</span>
        </div>
        <span className="sidebar-tag">Explorer</span>
      </div>

      <div className="sidebar-section">
        <h2>Simulation</h2>
        <div className="field">
          <label htmlFor="seed">Seed</label>
          <input id="seed" type="number" value={seed} onChange={e => setSeed(Number(e.target.value))} />
        </div>
        <div className="field">
          <label htmlFor="radius">Radius (m)</label>
          <input id="radius" type="number" value={radius} onChange={e => setRadius(Number(e.target.value))} />
        </div>
        <div className="field">
          <label htmlFor="mass">Mass (kg)</label>
          <input id="mass" type="number" value={mass} onChange={e => setMass(Number(e.target.value))} />
        </div>
        <div className="field">
          <label htmlFor="water">Water fraction</label>
          <input id="water" type="range" min={0} max={1} step={0.01} value={water} onChange={e => setWater(Number(e.target.value))} />
          <span className="field-value">{(water * 100).toFixed(0)}%</span>
        </div>
        <button className="primary-btn" onClick={handleGenerate} disabled={loading}>
          {loading ? 'Running...' : 'Generate Planet'}
        </button>
      </div>

      <div className="sidebar-section">
        <h2>Evolution</h2>
        <div className="field">
          <label htmlFor="speed">Speed</label>
          <select id="speed" value={speed} onChange={e => setSpeed(Number(e.target.value))}>
            <option value={1}>1x</option>
            <option value={10}>10x</option>
            <option value={100}>100x</option>
          </select>
        </div>
        <button className="primary-btn" onClick={() => onTick(1)} disabled={loading}>Advance Tick</button>
        <div className="metric">Tick {tickCount.toLocaleString()}</div>
      </div>

      <div className="sidebar-section">
        <h2>State</h2>
        <button className="secondary-btn" onClick={handleExport}>Export JSON</button>
        <textarea
          className="json-area"
          value={json}
          onChange={e => setJson(e.target.value)}
          placeholder="Simulation JSON will appear here..."
          aria-label="Simulation JSON"
        />
        <button className="secondary-btn" onClick={handleImport}>Import JSON</button>
      </div>
    </aside>
  );
});

Sidebar.displayName = 'Sidebar';

export default Sidebar;
