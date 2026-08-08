import React, { useRef, useEffect } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, PerspectiveCamera } from '@react-three/drei';
import { BufferAttribute } from 'three';

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

function Stars() {
  const points = useRef<any>(null);
  useEffect(() => {
    const positions = new Float32Array(1200 * 3);
    for (let i = 0; i < 1200; i++) {
      const r = 18 + Math.random() * 24;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      positions[i * 3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      positions[i * 3 + 2] = r * Math.cos(phi);
    }
    if (points.current) {
      points.current.geometry.setAttribute('position', new BufferAttribute(positions, 3));
    }
  }, []);
  return (
    <points ref={points}>
      <bufferGeometry />
      <pointsMaterial color="#8a8f98" size={0.018} sizeAttenuation depthWrite={false} />
    </points>
  );
}

function PlanetMaterial({ color, emissive }: { color: string; emissive?: string }) {
  return (
    <meshStandardMaterial
      color={color}
      emissive={emissive ?? '#000000'}
      emissiveIntensity={0.08}
      roughness={0.65}
      metalness={0.05}
    />
  );
}

function BiomeSphere({ color, emissive }: { color: string; emissive?: string }) {
  return (
    <mesh>
      <sphereGeometry args={[1, 64, 64]} />
      <PlanetMaterial color={color} emissive={emissive} />
    </mesh>
  );
}

function Atmosphere({ color }: { color: string }) {
  return (
    <mesh scale={[1.04, 1.04, 1.04]}>
      <sphereGeometry args={[1, 64, 64]} />
      <meshBasicMaterial color={color} transparent opacity={0.08} depthWrite={false} />
    </mesh>
  );
}

function Scene({ color, emissive }: { color: string; emissive?: string }) {
  return (
    <>
      <PerspectiveCamera makeDefault position={[0, 0.4, 3.2]} fov={45} />
      <OrbitControls
        enableDamping
        dampingFactor={0.08}
        minDistance={1.6}
        maxDistance={12}
        autoRotate
        autoRotateSpeed={0.4}
      />
      <ambientLight intensity={0.35} />
      <directionalLight position={[5, 3, 5]} intensity={1.15} color="#eef2ff" />
      <BiomeSphere color={color} emissive={emissive} />
      <Atmosphere color={color} />
      <Stars />
    </>
  );
}

const PlanetViewer: React.FC<Props> = ({ planet }) => {
  let color = '#27272a';
  let emissive: string | undefined;

  if (planet) {
    const temp = planet.temperature_k ?? planet.mean_temperature_k;
    const water = planet.water_fraction ?? 0;
    const ice = planet.ice_fraction ?? 0;
    const classification = (planet.primary_classification ?? '').toLowerCase();

    if (classification.includes('icy') || ice > 0.4 || (temp !== undefined && temp < 130)) {
      color = '#e4e4e7';
      emissive = '#18181b';
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
      color = '#52525b';
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
