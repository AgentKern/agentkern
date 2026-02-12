import { useState, useEffect } from 'react'

function App() {
  const [activity, setActivity] = useState([])
  const [stats, setStats] = useState({
    activeAgents: 0,
    blockedThreats: 0,
    budgetUsed: 0,
    avgRisk: 0
  })
  const [connectionStatus, setConnectionStatus] = useState('connecting')

  useEffect(() => {
    // Connect to AgentKern Unified Server WebSocket
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.hostname}:3000/api/v1/gate/ws/activity`;
    
    console.log(`📡 Connecting to Live Activity Feed: ${wsUrl}`);
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      setConnectionStatus('operational');
      console.log('✅ Connected to AgentKern Live Feed');
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        
        if (data.type === 'verification') {
          const v = data.verification;
          
          // Add to activity feed (keep last 10)
          setActivity(prev => [
            {
              id: Date.now(),
              time: new Date(v.timestamp).toLocaleTimeString(),
              agent: v.agent_id,
              action: v.action,
              status: v.result.allowed ? 'Allowed' : 'Blocked',
              risk: v.result.final_risk_score
            },
            ...prev
          ].slice(0, 10));

          // Update stats
          setStats(prev => ({
            ...prev,
            blockedThreats: v.result.allowed ? prev.blockedThreats : prev.blockedThreats + 1,
            avgRisk: Math.round((prev.avgRisk + v.result.final_risk_score) / 2)
          }));
        } else if (data.type === 'system_status') {
          setStats(prev => ({
            ...prev,
            activeAgents: data.active_agents
          }));
        }
      } catch (e) {
        console.error('Failed to parse dashboard event', e);
      }
    };

    ws.onclose = () => {
      setConnectionStatus('disconnected');
      console.warn('❌ Disconnected from AgentKern Live Feed');
    };

    return () => ws.close();
  }, [])

  return (
    <div className="app-container">
      <header className="glass">
        <div className="logo">AgentKern // Dashboard</div>
        <div className="status">
          System Status: <span style={{color: connectionStatus === 'operational' ? 'var(--accent)' : 'var(--danger)'}}>
            {connectionStatus.charAt(0).toUpperCase() + connectionStatus.slice(1)}
          </span>
        </div>
      </header>
      
      <main>
        <section className="stats-grid" style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
          gap: '1.5rem',
          marginBottom: '2rem'
        }}>
          <StatCard title="Active Agents" value={stats.activeAgents} color="var(--primary)" />
          <StatCard title="Blocked Threats" value={stats.blockedThreats} color="var(--danger)" />
          <StatCard title="Live Risk Average" value={stats.avgRisk} color="var(--accent)" />
          <StatCard title="Uptime" value="Live" color="var(--warning)" />
        </section>

        <section className="activity-feed glass" style={{
          padding: '1.5rem',
          borderRadius: '1rem'
        }}>
          <h2 style={{marginBottom: '1rem'}}>Live Activity Feed</h2>
          {activity.length === 0 ? (
            <div style={{color: 'var(--text-muted)', textAlign: 'center', padding: '2rem'}}>
              Waiting for agent activity...
            </div>
          ) : (
            <table style={{width: '100%', borderCollapse: 'collapse'}}>
              <thead>
                <tr style={{textAlign: 'left', borderBottom: '1px solid rgba(255, 255, 255, 0.1)'}}>
                  <th style={{padding: '1rem 0'}}>Time</th>
                  <th style={{padding: '1rem 0'}}>Agent</th>
                  <th style={{padding: '1rem 0'}}>Action</th>
                  <th style={{padding: '1rem 0'}}>Status</th>
                  <th style={{padding: '1rem 0'}}>Risk</th>
                </tr>
              </thead>
              <tbody>
                {activity.map(item => (
                  <tr key={item.id} style={{borderBottom: '1px solid rgba(255, 255, 255, 0.05)'}}>
                    <td style={{padding: '1rem 0', color: 'var(--text-muted)'}}>{item.time}</td>
                    <td style={{padding: '1rem 0', fontWeight: 600}}>{item.agent}</td>
                    <td style={{padding: '1rem 0'}}>{item.action}</td>
                    <td style={{padding: '1rem 0'}}>
                      <span style={{
                        padding: '0.25rem 0.5rem',
                        borderRadius: '0.25rem',
                        fontSize: '0.8rem',
                        backgroundColor: item.status === 'Blocked' ? 'rgba(239, 68, 68, 0.1)' : 'rgba(16, 185, 129, 0.1)',
                        color: item.status === 'Blocked' ? 'var(--danger)' : 'var(--accent)'
                      }}>{item.status}</span>
                    </td>
                    <td style={{padding: '1rem 0', fontWeight: 700}}>{item.risk}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </section>
      </main>
    </div>
  )
}

function StatCard({ title, value, color }) {
  return (
    <div className="stat-card glass" style={{
      padding: '1.5rem',
      borderRadius: '1rem',
      borderLeft: `4px solid ${color}`
    }}>
      <div style={{color: 'var(--text-muted)', fontSize: '0.9rem', marginBottom: '0.5rem'}}>{title}</div>
      <div style={{fontSize: '2rem', fontWeight: 700}}>{value}</div>
    </div>
  )
}

export default App
