import React from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, PerspectiveCamera, Grid, Environment } from '@react-three/drei';

interface Props {
  planet?: {
    temperature_k?: number;
    water_fraction?: number;
    ice_fraction?: number;
    primary_classification?: string;
  } | null;
}

function BiomeSphere({ color }: { color: string }) {
  return (
    <mesh>
      <sphereGeometry args={[1, 64, 64]} />
      <meshStandardMaterial color={color} roughness={0.6} metalness={0.1} />
    </mesh>
  );
}

function Scene({ color }: { color: string }) {
  return (
    <>
      <PerspectiveCamera makeDefault position={[0, 0, 3]} fov={50} />
      <OrbitControls enableDamping dampingFactor={0.05} />
      <ambientLight intensity={0.4} />
      <directionalLight position={[5, 3, 5]} intensity={1.2} />
      <BiomeSphere color={color} />
      <Grid args={[20, 20]} cellSize={0.5} cellThickness={0.5} cellColor="#1a1e2e" sectionSize={2} sectionThickness={1} sectionColor="#2a3045" fadeDistance={30} />
      <Environment preset="city" />
    </>
  );
}

const PlanetViewer: React.FC<Props> = ({ planet }) => {
  let color = '#3b82f6';
  if (planet) {
    const temp = planet.temperature_k;
    const water = planet.water_fraction ?? 0;
    const ice = planet.ice_fraction ?? 0;
    if (ice > 0.3) color = '#e2e8f0';
    else if (water > 0.3 && temp && temp > 273 && temp < 373) color = '#22c55e';
    else if (temp && temp > 373) color = '#ef4444';
    else if (temp && temp < 150) color = '#94a3b8';
    else color = '#a16207';
  }

  return (
    <div className="planet-viewer">
      <Canvas dpr={[1, 2]}>
        <Scene color={color} />
      </Canvas>
      <div className="planet-viewer-overlay">
        <div className="planet-chip" style={{ borderColor: color }}>
          <span className="planet-chip-dot" style={{ backgroundColor: color }} />
          {planet?.primary_classification ?? 'Awaiting simulation'}
        </div>
      </div>
    </div>
  );
};

export default PlanetViewer;
