import React from 'react';
import { WorldSmithClient, PlanetParams } from '../hooks/useWorldSmith';
import PlanetViewer from '../components/PlanetViewer';
import Sidebar from '../components/Sidebar';
import InfoPanel from '../components/InfoPanel';
import './../styles/global.css';

const PIPELINE_STEPS = [
  'Initializing...',
  'Core',
  'Mantle',
  'Volcanism',
  'Plate Tectonics',
  'Atmosphere',
  'Hydrology',
  'Climate',
  'Carbon Cycle',
  'Biosphere',
  'Habitability',
];

const Explorer: React.FC = () => {
  const [client] = React.useState(() => new WorldSmithClient());
  const [planet, setPlanet] = React.useState<ReturnType<WorldSmithClient['generatePlanet']> | null>(null);
  const [snapshot, setSnapshot] = React.useState<ReturnType<WorldSmithClient['tick']> | null>(null);
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [tickCount, setTickCount] = React.useState(0);
  const [speed, setSpeed] = React.useState(1);
  const [status, setStatus] = React.useState<'idle' | 'generating' | 'ready'>('idle');
  const [pipeline, setPipeline] = React.useState<string[]>([]);
  const sidebarRef = React.useRef<HTMLElement>(null);

  const runPipeline = React.useCallback(async () => {
    setStatus('generating');
    setPipeline([PIPELINE_STEPS[0]]);
    for (let i = 1; i < PIPELINE_STEPS.length; i++) {
      await new Promise(resolve => setTimeout(resolve, 80));
      setPipeline(prev => [...prev, PIPELINE_STEPS[i]]);
    }
    setStatus('ready');
  }, []);

  const handleGenerate = React.useCallback(async (params: PlanetParams) => {
    setLoading(true);
    setError(null);
    setPlanet(null);
    setSnapshot(null);
    setTickCount(0);
    try {
      runPipeline();
      client.init(params.seed);
      const p = client.generatePlanet(params);
      setPlanet(p);
      setTickCount(0);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setStatus('idle');
      setPipeline([]);
    } finally {
      setLoading(false);
    }
  }, [client, runPipeline]);

  const handleTick = React.useCallback((count: number) => {
    setLoading(true);
    try {
      const s = client.tick(count * speed);
      setSnapshot(s);
      setTickCount(c => c + count * speed);
      if (s.planets.length > 0) {
        setPlanet(s.planets[0]);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [client, speed]);

  const handleExport = React.useCallback(() => {
    try {
      return client.exportJSON();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return '';
    }
  }, [client]);

  const handleImport = React.useCallback((json: string) => {
    setLoading(true);
    try {
      const p = client.importJSON(json);
      setPlanet(p);
      setStatus('ready');
      setPipeline(PIPELINE_STEPS);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [client]);

  const handleEmptyGenerate = React.useCallback(() => {
    sidebarRef.current?.scrollIntoView({ behavior: 'smooth' });
    const seedInput = document.getElementById('seed') as HTMLInputElement | null;
    seedInput?.focus();
  }, []);

  return (
    <div className="explorer">
      <Sidebar
        ref={sidebarRef as any}
        onGenerate={handleGenerate}
        onTick={handleTick}
        onExport={handleExport}
        onImport={handleImport}
        tickCount={tickCount}
        speed={speed}
        setSpeed={setSpeed}
        loading={loading}
      />
      <main className="explorer-main">
        <PlanetViewer planet={planet ?? null} />
        {!planet && (
          <div className="empty-planet-ui">
            <div className="empty-planet-ring" />
            <div className="empty-planet-title">No World Loaded</div>
            <div className="empty-planet-desc">Generate a planet to begin exploring its physical systems.</div>
            <button className="empty-planet-action secondary-btn" onClick={handleEmptyGenerate}>Open Controls</button>
          </div>
        )}
        {error && <div className="error-toast">{error}</div>}
      </main>
      <InfoPanel planet={planet ?? undefined} snapshot={snapshot ?? undefined} status={status} pipeline={pipeline} />
    </div>
  );
};

export default Explorer;
