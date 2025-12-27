import { useState } from 'react'
import './App.css'

// Types
interface Agent {
  id: string;
  name: string;
  capabilities: string[];
  trustScore: number;
}

interface VerificationResult {
  allowed: boolean;
  riskScore: number;
  evaluatedPolicies: string[];
  reasoning: string;
}

interface IntentPath {
  intent: string;
  currentStep: number;
  expectedSteps: number;
  driftScore: number;
}

// Simulated AgentKern Client
const simulateRegister = async (name: string): Promise<Agent> => {
  await new Promise(r => setTimeout(r, 500));
  return {
    id: `agent-${Date.now().toString(36)}`,
    name,
    capabilities: ['read', 'write'],
    trustScore: 100,
  };
};

const simulateVerify = async (action: string, context: Record<string, unknown>): Promise<VerificationResult> => {
  await new Promise(r => setTimeout(r, 300));
  const amount = (context.amount as number) || 0;
  const riskScore = Math.min(100, Math.floor(amount / 100));
  return {
    allowed: riskScore < 80,
    riskScore,
    evaluatedPolicies: ['spending-limits', 'data-access'],
    reasoning: riskScore >= 80 ? 'Transaction amount too high' : 'All policies passed',
  };
};

const simulateStartIntent = async (intent: string, steps: number): Promise<IntentPath> => {
  await new Promise(r => setTimeout(r, 200));
  return {
    intent,
    currentStep: 0,
    expectedSteps: steps,
    driftScore: 0,
  };
};

export default function App() {
  const [activeTab, setActiveTab] = useState<'identity' | 'gate' | 'synapse' | 'arbiter'>('identity');
  const [agent, setAgent] = useState<Agent | null>(null);
  const [agentName, setAgentName] = useState('my-agent');
  const [loading, setLoading] = useState(false);

  // Gate state
  const [action, setAction] = useState('transfer_funds');
  const [amount, setAmount] = useState('5000');
  const [verification, setVerification] = useState<VerificationResult | null>(null);

  // Synapse state
  const [intent, setIntent] = useState('Process customer order');
  const [intentPath, setIntentPath] = useState<IntentPath | null>(null);

  const handleRegister = async () => {
    setLoading(true);
    const newAgent = await simulateRegister(agentName);
    setAgent(newAgent);
    setLoading(false);
  };

  const handleVerify = async () => {
    setLoading(true);
    const result = await simulateVerify(action, { amount: parseInt(amount) });
    setVerification(result);
    setLoading(false);
  };

  const handleStartIntent = async () => {
    setLoading(true);
    const path = await simulateStartIntent(intent, 4);
    setIntentPath(path);
    setLoading(false);
  };

  const handleRecordStep = async () => {
    if (!intentPath) return;
    setIntentPath({
      ...intentPath,
      currentStep: intentPath.currentStep + 1,
      driftScore: Math.min(100, intentPath.driftScore + Math.random() * 10),
    });
  };

  return (
    <div className="app">
      <header className="header">
        <div className="logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 2L2 7l10 5 10-5-10-5z"/>
            <path d="M2 17l10 5 10-5"/>
            <path d="M2 12l10 5 10-5"/>
          </svg>
          <span>AgentKern Playground</span>
        </div>
        <nav className="nav">
          <a href="https://github.com/daretechie/agentkern" target="_blank" rel="noopener noreferrer">
            GitHub
          </a>
          <a href="../docs/" target="_blank" rel="noopener noreferrer">
            Docs
          </a>
        </nav>
      </header>

      <main className="main">
        <aside className="sidebar">
          <div className="sidebar-section">
            <h3>The Four Pillars</h3>
            <button
              className={`sidebar-item ${activeTab === 'identity' ? 'active' : ''}`}
              onClick={() => setActiveTab('identity')}
            >
              🪪 Identity
            </button>
            <button
              className={`sidebar-item ${activeTab === 'gate' ? 'active' : ''}`}
              onClick={() => setActiveTab('gate')}
            >
              🛡️ Gate
            </button>
            <button
              className={`sidebar-item ${activeTab === 'synapse' ? 'active' : ''}`}
              onClick={() => setActiveTab('synapse')}
            >
              🧠 Synapse
            </button>
            <button
              className={`sidebar-item ${activeTab === 'arbiter' ? 'active' : ''}`}
              onClick={() => setActiveTab('arbiter')}
            >
              ⚖️ Arbiter
            </button>
          </div>

          {agent && (
            <div className="sidebar-section agent-card">
              <h4>Active Agent</h4>
              <p className="agent-id">{agent.id}</p>
              <p className="agent-name">{agent.name}</p>
              <div className="trust-score">
                Trust Score: <span className="score">{agent.trustScore}</span>
              </div>
            </div>
          )}
        </aside>

        <section className="content">
          {activeTab === 'identity' && (
            <div className="panel">
              <h2>🪪 Identity</h2>
              <p className="description">Register and manage agent identities.</p>

              <div className="form-group">
                <label>Agent Name</label>
                <input
                  type="text"
                  value={agentName}
                  onChange={(e) => setAgentName(e.target.value)}
                  placeholder="Enter agent name"
                />
              </div>

              <button className="primary" onClick={handleRegister} disabled={loading}>
                {loading ? 'Registering...' : 'Register Agent'}
              </button>

              {agent && (
                <div className="result success-result">
                  <h4>✅ Agent Registered</h4>
                  <pre><code>{JSON.stringify(agent, null, 2)}</code></pre>
                </div>
              )}
            </div>
          )}

          {activeTab === 'gate' && (
            <div className="panel">
              <h2>🛡️ Gate</h2>
              <p className="description">Verify actions against policies before execution.</p>

              <div className="form-group">
                <label>Action</label>
                <select value={action} onChange={(e) => setAction(e.target.value)}>
                  <option value="transfer_funds">transfer_funds</option>
                  <option value="read_data">read_data</option>
                  <option value="delete_record">delete_record</option>
                  <option value="send_email">send_email</option>
                </select>
              </div>

              <div className="form-group">
                <label>Amount (for transfers)</label>
                <input
                  type="number"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="Enter amount"
                />
              </div>

              <button className="primary" onClick={handleVerify} disabled={loading || !agent}>
                {loading ? 'Verifying...' : 'Verify Action'}
              </button>

              {!agent && (
                <p className="hint">⚠️ Register an agent first in the Identity tab.</p>
              )}

              {verification && (
                <div className={`result ${verification.allowed ? 'success-result' : 'error-result'}`}>
                  <h4>{verification.allowed ? '✅ Allowed' : '❌ Blocked'}</h4>
                  <div className="risk-meter">
                    <label>Risk Score</label>
                    <div className="meter">
                      <div 
                        className="meter-fill" 
                        style={{ 
                          width: `${verification.riskScore}%`,
                          background: verification.riskScore > 60 
                            ? 'var(--color-error)' 
                            : verification.riskScore > 30 
                              ? 'var(--color-warning)' 
                              : 'var(--color-success)'
                        }}
                      />
                    </div>
                    <span>{verification.riskScore}/100</span>
                  </div>
                  <p><strong>Reasoning:</strong> {verification.reasoning}</p>
                  <p><strong>Policies:</strong> {verification.evaluatedPolicies.join(', ')}</p>
                </div>
              )}
            </div>
          )}

          {activeTab === 'synapse' && (
            <div className="panel">
              <h2>🧠 Synapse</h2>
              <p className="description">Track agent intent and detect drift.</p>

              <div className="form-group">
                <label>Intent Description</label>
                <input
                  type="text"
                  value={intent}
                  onChange={(e) => setIntent(e.target.value)}
                  placeholder="What is the agent trying to do?"
                />
              </div>

              <div className="button-group">
                <button className="primary" onClick={handleStartIntent} disabled={loading || !agent}>
                  Start Intent Path
                </button>
                <button 
                  className="secondary" 
                  onClick={handleRecordStep} 
                  disabled={!intentPath}
                >
                  Record Step
                </button>
              </div>

              {!agent && (
                <p className="hint">⚠️ Register an agent first in the Identity tab.</p>
              )}

              {intentPath && (
                <div className="result">
                  <h4>📍 Intent Path</h4>
                  <p><strong>Goal:</strong> {intentPath.intent}</p>
                  <div className="progress-section">
                    <label>Progress: {intentPath.currentStep}/{intentPath.expectedSteps}</label>
                    <div className="meter">
                      <div 
                        className="meter-fill" 
                        style={{ 
                          width: `${(intentPath.currentStep / intentPath.expectedSteps) * 100}%`,
                          background: 'var(--color-primary)'
                        }}
                      />
                    </div>
                  </div>
                  <div className="drift-section">
                    <label>Drift Score</label>
                    <div className="meter">
                      <div 
                        className="meter-fill" 
                        style={{ 
                          width: `${intentPath.driftScore}%`,
                          background: intentPath.driftScore > 50 
                            ? 'var(--color-error)' 
                            : 'var(--color-success)'
                        }}
                      />
                    </div>
                    <span className={intentPath.driftScore > 50 ? 'error' : 'success'}>
                      {intentPath.driftScore.toFixed(1)}%
                    </span>
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === 'arbiter' && (
            <div className="panel">
              <h2>⚖️ Arbiter</h2>
              <p className="description">Coordinate access to shared resources.</p>

              <div className="form-group">
                <label>Resource</label>
                <input
                  type="text"
                  placeholder="e.g., database:accounts"
                  defaultValue="database:accounts"
                />
              </div>

              <div className="form-group">
                <label>Priority</label>
                <input type="number" defaultValue="5" min="1" max="10" />
              </div>

              <div className="button-group">
                <button className="primary" disabled={!agent}>
                  Request Lock
                </button>
                <button className="secondary" disabled={!agent}>
                  Release Lock
                </button>
              </div>

              {!agent && (
                <p className="hint">⚠️ Register an agent first in the Identity tab.</p>
              )}

              <div className="info-box">
                <h4>ℹ️ How Arbiter Works</h4>
                <ul>
                  <li>Agents request locks on resources they need</li>
                  <li>Higher priority agents can preempt lower priority ones</li>
                  <li>Locks automatically expire to prevent deadlocks</li>
                  <li>Queued agents are notified when locks become available</li>
                </ul>
              </div>
            </div>
          )}
        </section>
      </main>
    </div>
  )
}
