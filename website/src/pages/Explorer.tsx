import React from 'react';
import { WorldSmithClient, PlanetParams, PlanetState, Snapshot } from '../hooks/useWorldSmith';
import PlanetViewer from '../components/PlanetViewer';
import Sidebar from '../components/Sidebar';
import InfoPanel from '../components/InfoPanel';
import '../styles/global.css';

const Explorer: React.FC = () => {
  const [client] = React.useState(() => new WorldSmithClient());
  const [planet, setPlanet] = React.useState<PlanetState | null>(null);
  const [snapshot, setSnapshot] = React.useState<Snapshot | null>(null);
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [tickCount, setTickCount] = React.useState(0);
  const [speed, setSpeed] = React.useState(1);

  const handleGenerate = React.useCallback((params: PlanetParams) => {
    setLoading(true);
    setError(null);
    try {
      client.init(params.seed);
      const p = client.generatePlanet(params);
      setPlanet(p);
      setTickCount(0);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [client]);

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
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [client]);

  return (
    <div className="explorer">
      <Sidebar
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
        <PlanetViewer planet={planet} />
        {error && <div className="error-toast">{error}</div>}
      </main>
      <InfoPanel planet={planet ?? undefined} snapshot={snapshot ?? undefined} />
    </div>
  );
};

export default Explorer;
