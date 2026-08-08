import React from 'react';
import { useNavigate } from 'react-router-dom';
import './../styles/global.css';

const Home: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="home">
      <div className="home-content">
        <div className="home-logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" width="32" height="32">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            <path d="M2 12h20" />
          </svg>
        </div>
        <h1 className="home-title">WorldSmith</h1>
        <p className="home-subtitle">
          Realistic planetary simulation and visualization engine
        </p>
        <div className="home-meta">
          <span className="meta-tag">Rust + WASM</span>
          <span className="meta-tag">Three.js</span>
          <span className="meta-tag">Client-side</span>
        </div>
        <button className="launch-btn" onClick={() => navigate('/explorer')}>
          Launch Explorer
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="18" height="18">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
        </button>
      </div>
    </div>
  );
};

export default Home;
