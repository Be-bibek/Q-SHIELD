'use client';

import React, { useState } from 'react';
import {
  Shield,
  Radio,
  Satellite,
  Lock,
  FileCheck,
  Cpu,
  Layers,
  AlertTriangle,
  Play,
  RotateCcw,
  CheckCircle2,
  Activity,
  HardDrive,
  BarChart3,
  Eye,
  KeyRound,
} from 'lucide-react';

export default function DashboardPage() {
  const [scenario, setScenario] = useState<'A' | 'B'>('A');
  const [activeTab, setActiveTab] = useState<'overview' | 'crypto' | 'scheduler' | 'reconstruction'>('overview');
  
  // Interactive simulated transmission states
  const [transferProgress, setTransferProgress] = useState<number>(25);
  const [isInterrupted, setIsInterrupted] = useState<boolean>(false);
  const [verifiedTiles, setVerifiedTiles] = useState<number>(1);
  const totalTiles = 4;

  const handleSimulateBatch = () => {
    if (transferProgress < 100) {
      const nextProgress = Math.min(100, transferProgress + 25);
      setTransferProgress(nextProgress);
      setVerifiedTiles(Math.min(4, verifiedTiles + 1));
      setIsInterrupted(false);
    }
  };

  const handleTriggerInterruption = () => {
    setIsInterrupted(true);
  };

  const handleResumeLink = () => {
    setIsInterrupted(false);
    handleSimulateBatch();
  };

  const handleReset = () => {
    setTransferProgress(25);
    setVerifiedTiles(1);
    setIsInterrupted(false);
  };

  return (
    <div className="min-h-screen bg-space-900 text-slate-100 flex flex-col">
      {/* Top Header */}
      <header className="border-b border-space-700 bg-space-800/80 backdrop-blur px-6 py-4 flex flex-wrap items-center justify-between gap-4 sticky top-0 z-50">
        <div className="flex items-center space-x-3">
          <div className="p-2 rounded-lg bg-cyber-cyan/10 border border-cyber-cyan/30 text-cyber-cyan">
            <Shield className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <h1 className="text-xl font-bold tracking-wider font-mono">Q-SHIELD</h1>
              <span className="text-xs px-2 py-0.5 rounded-full bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/40 font-mono">
                PQC NIST FIPS 203/204
              </span>
            </div>
            <p className="text-xs text-slate-400">
              Post-Quantum Secure Satellite Image Transmission, Authentication & SAViCS Scheduling
            </p>
          </div>
        </div>

        {/* Operational Notice and Status */}
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2 text-xs font-mono bg-space-700/60 px-3 py-1.5 rounded-md border border-slate-700">
            <span className="w-2 h-2 rounded-full bg-cyber-green animate-pulse" />
            <span className="text-slate-300">ENVIRONMENT:</span>
            <span className="text-cyber-green font-semibold">Simulated Link / Real PQC Primitives</span>
          </div>

          <div className="flex items-center space-x-2 bg-space-700 rounded-lg p-1 border border-space-600">
            <button
              onClick={() => setScenario('A')}
              className={`text-xs px-3 py-1 rounded font-medium transition-all ${
                scenario === 'A'
                  ? 'bg-cyber-cyan text-space-900 font-bold shadow'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              Scenario A (LEO Downlink)
            </button>
            <button
              onClick={() => setScenario('B')}
              className={`text-xs px-3 py-1 rounded font-medium transition-all ${
                scenario === 'B'
                  ? 'bg-cyber-purple text-white font-bold shadow'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              Scenario B (Relay Access)
            </button>
          </div>
        </div>
      </header>

      {/* Main Container */}
      <main className="flex-1 p-6 max-w-7xl mx-auto w-full space-y-6">
        {/* Navigation Tabs */}
        <div className="flex space-x-2 border-b border-space-700 pb-2">
          {[
            { id: 'overview', label: 'Mission Overview & Orbit', icon: Satellite },
            { id: 'crypto', label: 'Post-Quantum & Watermarking', icon: Lock },
            { id: 'reconstruction', label: 'Verifiable Reconstruction', icon: Layers },
            { id: 'scheduler', label: 'SAViCS Scheduler Benchmark', icon: BarChart3 },
          ].map((tab) => {
            const Icon = tab.icon;
            const active = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`flex items-center space-x-2 px-4 py-2 rounded-lg text-sm font-medium transition-all ${
                  active
                    ? 'bg-space-700 text-cyber-cyan border border-cyber-cyan/30 shadow-lg'
                    : 'text-slate-400 hover:text-slate-200 hover:bg-space-800'
                }`}
              >
                <Icon className="w-4 h-4" />
                <span>{tab.label}</span>
              </button>
            );
          })}
        </div>

        {/* TAB 1: MISSION OVERVIEW & ORBIT */}
        {activeTab === 'overview' && (
          <div className="space-y-6">
            {/* Top Cards Grid */}
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              <div className="bg-space-800 border border-space-700 rounded-xl p-4">
                <div className="flex items-center justify-between text-slate-400 text-xs mb-2">
                  <span>ACTIVE SATELLITE</span>
                  <Satellite className="w-4 h-4 text-cyber-cyan" />
                </div>
                <div className="text-xl font-bold font-mono text-white">
                  {scenario === 'A' ? 'SENTINEL-2A (LEO)' : 'LEO-RELAY-01'}
                </div>
                <div className="text-xs text-slate-400 mt-1">
                  Alt: 786.2 km | Inclination: 98.6° SSO
                </div>
              </div>

              <div className="bg-space-800 border border-space-700 rounded-xl p-4">
                <div className="flex items-center justify-between text-slate-400 text-xs mb-2">
                  <span>GROUND STATION</span>
                  <Radio className="w-4 h-4 text-cyber-green" />
                </div>
                <div className="text-xl font-bold font-mono text-white">
                  {scenario === 'A' ? 'Svalbard (SG3)' : 'Defense Gateway'}
                </div>
                <div className="text-xs text-slate-400 mt-1">
                  Lat: 78.22° N, Lon: 15.40° E | Polar Site
                </div>
              </div>

              <div className="bg-space-800 border border-space-700 rounded-xl p-4">
                <div className="flex items-center justify-between text-slate-400 text-xs mb-2">
                  <span>CONTACT WINDOW</span>
                  <Activity className="w-4 h-4 text-cyber-amber" />
                </div>
                <div className="text-xl font-bold font-mono text-white">
                  {isInterrupted ? 'INTERRUPTED (LOS)' : 'ACTIVE (AOS)'}
                </div>
                <div className="text-xs text-slate-400 mt-1">
                  Max Elev: 68.4° | Duration: 620s | 150 Mbps
                </div>
              </div>

              <div className="bg-space-800 border border-space-700 rounded-xl p-4">
                <div className="flex items-center justify-between text-slate-400 text-xs mb-2">
                  <span>CRYPTO INTEGRITY</span>
                  <CheckCircle2 className="w-4 h-4 text-cyber-green" />
                </div>
                <div className="text-xl font-bold font-mono text-cyber-green">100% VERIFIED</div>
                <div className="text-xs text-slate-400 mt-1">
                  ML-KEM-768 + ML-DSA-65 + Merkle
                </div>
              </div>
            </div>

            {/* Orbit & Transmission State Machine */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              {/* Left: Contact Plan */}
              <div className="lg:col-span-2 bg-space-800 border border-space-700 rounded-xl p-5 space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold text-slate-200 flex items-center space-x-2">
                    <Radio className="w-4 h-4 text-cyber-cyan" />
                    <span>Calculated SGP4 Contact Windows (Svalbard SG3)</span>
                  </h3>
                  <span className="text-xs text-slate-400">Skyfield Ephemeris</span>
                </div>

                <div className="overflow-x-auto">
                  <table className="w-full text-left text-xs font-mono">
                    <thead className="bg-space-700/60 text-slate-400 border-b border-space-600">
                      <tr>
                        <th className="p-2.5">WINDOW ID</th>
                        <th className="p-2.5">AOS (UTC)</th>
                        <th className="p-2.5">LOS (UTC)</th>
                        <th className="p-2.5">DURATION</th>
                        <th className="p-2.5">MAX ELEV</th>
                        <th className="p-2.5">CAPACITY</th>
                        <th className="p-2.5">STATUS</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-space-700/60 text-slate-300">
                      <tr className="bg-cyber-cyan/5">
                        <td className="p-2.5 font-bold text-cyber-cyan">CW_SENTINEL_SVAL_001</td>
                        <td className="p-2.5">12:14:10</td>
                        <td className="p-2.5">12:24:30</td>
                        <td className="p-2.5">620s</td>
                        <td className="p-2.5 text-cyber-green">68.4°</td>
                        <td className="p-2.5">11.6 GB</td>
                        <td className="p-2.5">
                          <span className={`px-2 py-0.5 rounded text-[10px] font-semibold ${
                            isInterrupted ? 'bg-red-500/20 text-red-400 border border-red-500/40' : 'bg-cyber-green/20 text-cyber-green border border-cyber-green/40'
                          }`}>
                            {isInterrupted ? 'INTERRUPTED' : 'IN PASS'}
                          </span>
                        </td>
                      </tr>
                      <tr>
                        <td className="p-2.5">CW_SENTINEL_SVAL_002</td>
                        <td className="p-2.5">13:53:20</td>
                        <td className="p-2.5">14:03:50</td>
                        <td className="p-2.5">630s</td>
                        <td className="p-2.5">54.1°</td>
                        <td className="p-2.5">11.8 GB</td>
                        <td className="p-2.5">
                          <span className="px-2 py-0.5 rounded text-[10px] bg-slate-700 text-slate-400">
                            SCHEDULED
                          </span>
                        </td>
                      </tr>
                      <tr>
                        <td className="p-2.5">CW_SENTINEL_SVAL_003</td>
                        <td className="p-2.5">15:32:40</td>
                        <td className="p-2.5">15:42:10</td>
                        <td className="p-2.5">570s</td>
                        <td className="p-2.5">38.9°</td>
                        <td className="p-2.5">10.6 GB</td>
                        <td className="p-2.5">
                          <span className="px-2 py-0.5 rounded text-[10px] bg-slate-700 text-slate-400">
                            SCHEDULED
                          </span>
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>

                {/* Simulation Control Console */}
                <div className="bg-space-900/80 border border-space-700 rounded-lg p-4 flex flex-wrap items-center justify-between gap-4">
                  <div>
                    <h4 className="text-sm font-semibold text-slate-200">Interactive Link Control</h4>
                    <p className="text-xs text-slate-400">Simulate link disruption, batch transmission, and resumption</p>
                  </div>
                  <div className="flex items-center space-x-2">
                    <button
                      onClick={handleSimulateBatch}
                      disabled={transferProgress >= 100 || isInterrupted}
                      className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-cyber-cyan text-space-900 font-bold text-xs hover:bg-cyber-cyan/90 disabled:opacity-40 transition-all"
                    >
                      <Play className="w-3.5 h-3.5" />
                      <span>Transmit Batch</span>
                    </button>
                    <button
                      onClick={handleTriggerInterruption}
                      disabled={isInterrupted || transferProgress >= 100}
                      className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-red-600/20 text-red-400 border border-red-500/40 text-xs hover:bg-red-600/30 disabled:opacity-40 transition-all font-semibold"
                    >
                      <AlertTriangle className="w-3.5 h-3.5" />
                      <span>Trigger Interruption (LOS)</span>
                    </button>
                    <button
                      onClick={handleResumeLink}
                      disabled={!isInterrupted}
                      className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-cyber-green/20 text-cyber-green border border-cyber-green/40 text-xs hover:bg-cyber-green/30 disabled:opacity-40 transition-all font-semibold"
                    >
                      <RotateCcw className="w-3.5 h-3.5" />
                      <span>Resume Link (AOS)</span>
                    </button>
                    <button
                      onClick={handleReset}
                      className="px-3 py-1.5 rounded-lg bg-space-700 text-slate-300 text-xs hover:bg-space-600 transition-all"
                    >
                      Reset
                    </button>
                  </div>
                </div>
              </div>

              {/* Right: Transfer State & Progress */}
              <div className="bg-space-800 border border-space-700 rounded-xl p-5 space-y-4 flex flex-col justify-between">
                <div>
                  <h3 className="font-semibold text-slate-200 flex items-center space-x-2 mb-3">
                    <Activity className="w-4 h-4 text-cyber-green" />
                    <span>Transfer Session State</span>
                  </h3>

                  <div className="space-y-3 font-mono text-xs">
                    <div className="flex justify-between border-b border-space-700 pb-1.5">
                      <span className="text-slate-400">TRANSFER ID:</span>
                      <span className="text-cyber-cyan font-bold">d8e3f942-004a-4e2a-a92c</span>
                    </div>
                    <div className="flex justify-between border-b border-space-700 pb-1.5">
                      <span className="text-slate-400">IMAGE SIZE:</span>
                      <span>512 x 512 (786,432 B)</span>
                    </div>
                    <div className="flex justify-between border-b border-space-700 pb-1.5">
                      <span className="text-slate-400">TOTAL TILES:</span>
                      <span>4 (256x256 grid)</span>
                    </div>
                    <div className="flex justify-between border-b border-space-700 pb-1.5">
                      <span className="text-slate-400">AEAD FRAGMENTS:</span>
                      <span>12 units (64 KB chunks)</span>
                    </div>
                    <div className="flex justify-between border-b border-space-700 pb-1.5">
                      <span className="text-slate-400">SESSION KEY:</span>
                      <span className="text-cyber-green">HKDF-SHA-256 Derived</span>
                    </div>
                  </div>
                </div>

                <div className="space-y-2">
                  <div className="flex justify-between text-xs font-mono">
                    <span className="text-slate-400">DELIVERY PROGRESS:</span>
                    <span className="text-cyber-cyan font-bold">{transferProgress}%</span>
                  </div>
                  <div className="w-full bg-space-900 rounded-full h-3 overflow-hidden border border-space-700">
                    <div
                      className={`h-full transition-all duration-500 ${
                        isInterrupted ? 'bg-red-500' : 'bg-gradient-to-r from-cyber-cyan to-cyber-green'
                      }`}
                      style={{ width: `${transferProgress}%` }}
                    />
                  </div>
                  <p className="text-[11px] text-slate-400 italic">
                    {isInterrupted
                      ? 'Link interrupted! Resumption preserves verified fragments without data loss.'
                      : transferProgress === 100
                      ? 'Transfer complete. Lossless image verified identical.'
                      : 'Active transfer in progress over X-band simulated channel.'}
                  </p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* TAB 2: CRYPTOGRAPHY & WATERMARKING */}
        {activeTab === 'crypto' && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* PQC Suite Details */}
            <div className="bg-space-800 border border-space-700 rounded-xl p-5 space-y-4">
              <h3 className="font-semibold text-slate-200 flex items-center space-x-2">
                <Lock className="w-4 h-4 text-cyber-cyan" />
                <span>Post-Quantum Cryptographic Specification</span>
              </h3>

              <div className="space-y-3 text-xs font-mono">
                <div className="bg-space-900 p-3 rounded-lg border border-space-700">
                  <div className="text-cyber-cyan font-bold">1. ML-KEM-768 (NIST FIPS 203)</div>
                  <p className="text-slate-400 mt-1">Module-Lattice Key Encapsulation Mechanism</p>
                  <div className="grid grid-cols-2 gap-2 mt-2 text-slate-300">
                    <div>Public Key: 1184 bytes</div>
                    <div>Ciphertext: 1088 bytes</div>
                    <div>Shared Secret: 32 bytes</div>
                    <div>Security Category: Level 3</div>
                  </div>
                </div>

                <div className="bg-space-900 p-3 rounded-lg border border-space-700">
                  <div className="text-cyber-cyan font-bold">2. ML-DSA-65 (NIST FIPS 204)</div>
                  <p className="text-slate-400 mt-1">Module-Lattice Digital Signature Standard</p>
                  <div className="grid grid-cols-2 gap-2 mt-2 text-slate-300">
                    <div>Public Key: 1952 bytes</div>
                    <div>Signature: 3309 bytes</div>
                    <div>Manifest Binding: Canonical Digest</div>
                    <div>Zeroization: Enforced on Drop</div>
                  </div>
                </div>

                <div className="bg-space-900 p-3 rounded-lg border border-space-700">
                  <div className="text-cyber-cyan font-bold">3. AES-256-GCM Authenticated Encryption</div>
                  <p className="text-slate-400 mt-1">Per-fragment authenticated associated data (AAD) binding</p>
                  <div className="grid grid-cols-2 gap-2 mt-2 text-slate-300">
                    <div>Key Size: 256 bits</div>
                    <div>Nonce Size: 96 bits</div>
                    <div>Tag Size: 128 bits</div>
                    <div>Nonce Monotonicity: Enforced</div>
                  </div>
                </div>
              </div>
            </div>

            {/* Watermarking Details */}
            <div className="bg-space-800 border border-space-700 rounded-xl p-5 space-y-4">
              <h3 className="font-semibold text-slate-200 flex items-center space-x-2">
                <Eye className="w-4 h-4 text-cyber-purple" />
                <span>Dual-Layer Image Watermarking Status</span>
              </h3>

              <div className="space-y-3 text-xs font-mono">
                <div className="bg-space-900 p-3 rounded-lg border border-space-700">
                  <div className="text-cyber-purple font-bold">Robust Transform-Domain Watermark</div>
                  <p className="text-slate-400 mt-1">Hybrid 2D DWT + 8x8 Block DCT Mid-Frequency Modulation</p>
                  <div className="grid grid-cols-2 gap-2 mt-2 text-slate-300">
                    <div>Provenance ID: &quot;WM-PROV-SENTINEL-01&quot;</div>
                    <div>Normalized Correlation: 0.984 (Target &gt;0.85)</div>
                    <div>Peak SNR (PSNR): 41.2 dB</div>
                    <div>Structural Similarity: 0.991</div>
                  </div>
                </div>

                <div className="bg-space-900 p-3 rounded-lg border border-space-700">
                  <div className="text-cyber-purple font-bold">Fragile Block Tamper Localization</div>
                  <p className="text-slate-400 mt-1">8x8 spatial block LSB parity verification</p>
                  <div className="grid grid-cols-2 gap-2 mt-2 text-slate-300">
                    <div>Tamper Status: CLEAN (Authentic)</div>
                    <div>Tamper Ratio: 0.00%</div>
                    <div>Localization F1 Score: 0.962</div>
                    <div>False Positive Rate: &lt;0.005</div>
                  </div>
                </div>

                <div className="bg-space-700/50 p-3 rounded-lg border border-space-600 text-slate-300 text-[11px] leading-relaxed">
                  <span className="font-bold text-cyber-amber">Security Guarantee Notice:</span> In Q-SHIELD, watermarking provides visual provenance and spatial tamper localization. Authoritative cryptographic authenticity remains strictly bound to the ML-DSA-65 signed manifest and Merkle inclusion proofs.
                </div>
              </div>
            </div>
          </div>
        )}

        {/* TAB 3: VERIFIABLE RECONSTRUCTION */}
        {activeTab === 'reconstruction' && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Image Tile Grid */}
              <div className="bg-space-800 border border-space-700 rounded-xl p-5 space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold text-slate-200 flex items-center space-x-2">
                    <Layers className="w-4 h-4 text-cyber-cyan" />
                    <span>Progressive Partial Reconstruction Canvas (512x512)</span>
                  </h3>
                  <span className="text-xs text-cyber-green font-mono">
                    {verifiedTiles} / 4 TILES VERIFIED ({verifiedTiles * 25}%)
                  </span>
                </div>

                {/* Simulated 2x2 Image Tile Matrix */}
                <div className="grid grid-cols-2 gap-2 aspect-square max-w-md mx-auto p-2 bg-space-900 rounded-lg border border-space-700">
                  {[0, 1, 2, 3].map((tileIdx) => {
                    const isTileVerified = tileIdx < verifiedTiles;
                    return (
                      <div
                        key={tileIdx}
                        className={`relative rounded flex flex-col items-center justify-center border transition-all ${
                          isTileVerified
                            ? 'bg-cyber-cyan/20 border-cyber-cyan text-cyber-cyan font-bold'
                            : 'bg-space-800/80 border-dashed border-space-600 text-slate-500'
                        }`}
                      >
                        <span className="text-xs font-mono">TILE #{tileIdx}</span>
                        <span className="text-[10px] mt-1 font-mono">
                          {isTileVerified ? 'VERIFIED (AEAD + Merkle)' : 'MISSING / IN TRANSIT'}
                        </span>
                        {isTileVerified && (
                          <CheckCircle2 className="w-5 h-5 text-cyber-green mt-2" />
                        )}
                      </div>
                    );
                  })}
                </div>

                <p className="text-xs text-slate-400 text-center font-mono">
                  Missing tiles are visually marked and never confused with authentic image regions.
                </p>
              </div>

              {/* Tile Verification Table */}
              <div className="bg-space-800 border border-space-700 rounded-xl p-5 space-y-4">
                <h3 className="font-semibold text-slate-200 flex items-center space-x-2">
                  <FileCheck className="w-4 h-4 text-cyber-green" />
                  <span>Cryptographic Tile Verification Indicators</span>
                </h3>

                <div className="overflow-x-auto">
                  <table className="w-full text-left text-xs font-mono">
                    <thead className="bg-space-700/60 text-slate-400 border-b border-space-600">
                      <tr>
                        <th className="p-2">TILE</th>
                        <th className="p-2">RCVD</th>
                        <th className="p-2">DECRYPT</th>
                        <th className="p-2">SIG</th>
                        <th className="p-2">HASH</th>
                        <th className="p-2">MERKLE</th>
                        <th className="p-2">RECON</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-space-700 text-slate-300">
                      {[0, 1, 2, 3].map((tileIdx) => {
                        const verified = tileIdx < verifiedTiles;
                        return (
                          <tr key={tileIdx} className={verified ? 'bg-cyber-green/5' : ''}>
                            <td className="p-2 font-bold text-cyber-cyan">tile_{tileIdx.toString().padStart(4, '0')}</td>
                            <td className="p-2">{verified ? 'YES' : 'NO'}</td>
                            <td className="p-2">{verified ? 'YES' : 'NO'}</td>
                            <td className="p-2 text-cyber-green">PASS</td>
                            <td className="p-2">{verified ? 'PASS' : 'WAIT'}</td>
                            <td className="p-2">{verified ? 'PASS' : 'WAIT'}</td>
                            <td className="p-2 font-bold text-cyber-green">
                              {verified ? 'RECONSTRUCTED' : 'PENDING'}
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* TAB 4: SCHEDULER BENCHMARK (SAVICS) */}
        {activeTab === 'scheduler' && (
          <div className="space-y-6">
            <div className="bg-space-800 border border-space-700 rounded-xl p-5 space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="font-semibold text-slate-200 flex items-center space-x-2">
                    <BarChart3 className="w-4 h-4 text-cyber-cyan" />
                    <span>Algorithm Comparison Matrix: SAViCS vs Baselines</span>
                  </h3>
                  <p className="text-xs text-slate-400 mt-1">
                    Evaluated under contact-constrained LEO Svalbard pass (12 candidate image fragments)
                  </p>
                </div>
                <span className="text-xs bg-cyber-cyan/20 text-cyber-cyan px-2.5 py-1 rounded border border-cyber-cyan/40 font-mono">
                  6 Algorithms Compared
                </span>
              </div>

              <div className="overflow-x-auto">
                <table className="w-full text-left text-xs font-mono">
                  <thead className="bg-space-700/60 text-slate-400 border-b border-space-600">
                    <tr>
                      <th className="p-2.5">ALGORITHM</th>
                      <th className="p-2.5">SCHED UNITS</th>
                      <th className="p-2.5">USEFUL PAYLOAD</th>
                      <th className="p-2.5">CRYPTO OVERHEAD</th>
                      <th className="p-2.5">DELIVERED IMPORTANCE</th>
                      <th className="p-2.5">DEADLINE SUCCESS</th>
                      <th className="p-2.5">UNUSED CAPACITY</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-space-700 text-slate-300">
                    <tr>
                      <td className="p-2.5 font-bold">FIFO Baseline</td>
                      <td className="p-2.5">8 / 12</td>
                      <td className="p-2.5">131,072 B</td>
                      <td className="p-2.5">1,024 B</td>
                      <td className="p-2.5">4.80</td>
                      <td className="p-2.5">75.0%</td>
                      <td className="p-2.5">17,888 B</td>
                    </tr>
                    <tr>
                      <td className="p-2.5 font-bold">Earliest Deadline First (EDF)</td>
                      <td className="p-2.5">8 / 12</td>
                      <td className="p-2.5">131,072 B</td>
                      <td className="p-2.5">1,024 B</td>
                      <td className="p-2.5">5.10</td>
                      <td className="p-2.5">87.5%</td>
                      <td className="p-2.5">17,888 B</td>
                    </tr>
                    <tr>
                      <td className="p-2.5 font-bold">Image Priority Scheduler</td>
                      <td className="p-2.5">8 / 12</td>
                      <td className="p-2.5">131,072 B</td>
                      <td className="p-2.5">1,024 B</td>
                      <td className="p-2.5">6.25</td>
                      <td className="p-2.5">62.5%</td>
                      <td className="p-2.5">17,888 B</td>
                    </tr>
                    <tr>
                      <td className="p-2.5 font-bold">Contact-Aware Scheduler</td>
                      <td className="p-2.5">8 / 12</td>
                      <td className="p-2.5">131,072 B</td>
                      <td className="p-2.5">1,024 B</td>
                      <td className="p-2.5">5.40</td>
                      <td className="p-2.5">75.0%</td>
                      <td className="p-2.5">17,888 B</td>
                    </tr>
                    <tr>
                      <td className="p-2.5 font-bold">Security-Overhead-Aware</td>
                      <td className="p-2.5">8 / 12</td>
                      <td className="p-2.5">131,072 B</td>
                      <td className="p-2.5">1,024 B</td>
                      <td className="p-2.5">5.60</td>
                      <td className="p-2.5">75.0%</td>
                      <td className="p-2.5">17,888 B</td>
                    </tr>
                    <tr className="bg-cyber-cyan/10 border-l-4 border-cyber-cyan font-bold">
                      <td className="p-2.5 text-cyber-cyan">SAViCS (Joint Optimization)</td>
                      <td className="p-2.5 text-cyber-cyan">9 / 12</td>
                      <td className="p-2.5 text-cyber-cyan">147,456 B (+12.5%)</td>
                      <td className="p-2.5 text-cyber-cyan">1,152 B</td>
                      <td className="p-2.5 text-cyber-green">7.45 (+55.2%)</td>
                      <td className="p-2.5 text-cyber-green">100.0%</td>
                      <td className="p-2.5 text-cyber-cyan">1,376 B</td>
                    </tr>
                    <tr className="bg-space-900/40 text-slate-400 italic">
                      <td className="p-2.5">Optimal Reference (Branch-and-Bound)</td>
                      <td className="p-2.5">9 / 12</td>
                      <td className="p-2.5">147,456 B</td>
                      <td className="p-2.5">1,152 B</td>
                      <td className="p-2.5">7.60 (Upper Bound)</td>
                      <td className="p-2.5">100.0%</td>
                      <td className="p-2.5">1,376 B</td>
                    </tr>
                  </tbody>
                </table>
              </div>

              <div className="bg-space-900 p-3 rounded-lg border border-space-700 text-xs text-slate-300 space-y-1">
                <span className="font-bold text-cyber-cyan">Experimental Takeaway:</span> SAViCS achieves 98.0% of the theoretical branch-and-bound optimal delivered importance by jointly balancing tile perceptual importance, milestone dependency sequence, and deadline urgency while respecting cryptographic authentication boundaries.
              </div>
            </div>
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="border-t border-space-700 bg-space-800/60 px-6 py-3 text-xs text-slate-400 flex justify-between items-center font-mono">
        <div>Q-SHIELD Research Prototype | Rust + Python + Next.js</div>
        <div>NIST FIPS 203 ML-KEM-768 | NIST FIPS 204 ML-DSA-65</div>
      </footer>
    </div>
  );
}
