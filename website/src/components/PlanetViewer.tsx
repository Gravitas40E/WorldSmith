import React from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, PerspectiveCamera, Grid, Environment } from '@react-three/drei';

interface Props {
  planet?: {
    temperature_k?: number;
    mean_temperature_k?: number;
    water_fraction?: number;
    ice_fraction?: number;
    primary_classification?: string;
    habitability_class?: string;
    pressure_pa?: number;
    gravity_m_s2?: number;
  } | null;
}

function BiomeSphere({ color, emissive }: { color: string; emissive?: string }) {
  return (
    <mesh>
      <sphereGeometry args={[1, 64, 64]} />
      <meshStandardMaterial color={color} emissive={emissive ?? '#000000'} emissiveIntensity={0.15} roughness={0.6} metalness={0.1} />
    </mesh>
  );
}

function Scene({ color, emissive }: { color: string; emissive?: string }) {
  return (
    <>
      <PerspectiveCamera makeDefault position={[0, 0, 3]} fov={50} />
      <OrbitControls enableDamping dampingFactor={0.05} />
      <ambientLight intensity={0.4} />
      <directionalLight position={[5, 3, 5]} intensity={1.2} />
      <BiomeSphere color={color} emissive={emissive} />
      <Grid args={[20, 20]} cellSize={0.5} cellThickness={0.5} cellColor="#1a1e2e" sectionSize={2} sectionThickness={1} sectionColor="#2a3045" fadeDistance={30} />
      <Environment preset="city" />
    </>
  );
}

const PlanetViewer: React.FC<Props> = ({ planet }) => {
  let color = '#0f172a';
  let emissive: string | undefined;

  if (planet) {
    const temp = planet.temperature_k ?? planet.mean_temperature_k;
    const water = planet.water_fraction ?? 0;
    const ice = planet.ice_fraction ?? 0;
    const classification = (planet.primary_classification ?? '').toLowerCase();

    if (classification.includes('icy') || ice > 0.4 || (temp !== undefined && temp < 130)) {
      color = '#e2e8f0';
      emissive = '#1e293b';
    } else if (classification.includes('volcanic') || (temp !== undefined && temp > 900)) {
      color = '#b45309';
      emissive = '#7c2d12';
    } else if (classification.includes('desert') || (temp !== undefined && temp > 320 && water < 0.1)) {
      color = '#a16207';
      emissive = '#451a03';
    } else if (classification.includes('ocean') || (water > 0.5 && temp !== undefined && temp >= 273 && temp <= 373)) {
      color = '#1d4ed8';
      emissive = '#0c1e4d';
    } else if (classification.includes('temperate') || (temp !== undefined && temp >= 273 && temp <= 313)) {
      color = '#15803d';
      emissive = '#052e16';
    } else if (classification.includes('habitable') || planet.habitability_class?.toLowerCase().includes('habitable')) {
      color = '#22c55e';
      emissive = '#14532d';
    } else {
      color = '#64748b';
    }
  }

  return (
    <div className="planet-viewer">
      <Canvas dpr={[1, 2]}>
        <Scene color={color} emissive={emissive} />
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
