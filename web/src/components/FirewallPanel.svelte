<script lang="ts">
  const dispatch = new CustomEvent('refresh')

  export let token: string

  let rules = ''
  let loading = false
  let error = ''
  let action = 'allow'
  let protocol = 'tcp'
  let port = ''
  let interface_name = 'eth0'

  async function loadRules() {
    loading = true
    error = ''
    try {
      const response = await fetch('/api/firewall/rules', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        rules = data.data
      } else {
        error = data.error || 'Failed to load rules'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }

  async function applyRule() {
    if (!port) {
      error = 'Please enter a port'
      return
    }

    error = ''
    try {
      const response = await fetch('/api/firewall/apply', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
          action,
          interface: interface_name,
          protocol,
          port: parseInt(port)
        })
      })

      const data = await response.json()
      if (data.success) {
        port = ''
        await loadRules()
      } else {
        error = data.error || 'Failed to apply rule'
      }
    } catch (err: any) {
      error = err.message
    }
  }

  async function saveRules() {
    error = ''
    try {
      const response = await fetch('/api/firewall/save', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })

      const data = await response.json()
      if (!data.success) {
        error = data.error || 'Failed to save rules'
      } else {
        alert('Firewall rules saved successfully!')
      }
    } catch (err: any) {
      error = err.message
    }
  }
</script>

<div class="firewall-panel">
  <div class="panel-header">
    <h2>Firewall Management</h2>
    <div class="buttons">
      <button class="btn btn-primary" on:click={loadRules} disabled={loading}>
        {loading ? '⏳ Loading...' : '↻ Refresh'}
      </button>
      <button class="btn btn-success" on:click={saveRules}>💾 Save Rules</button>
    </div>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  <div class="grid">
    <div class="card">
      <h3>Apply New Rule</h3>
      <div class="form-group">
        <label>Interface</label>
        <select bind:value={interface_name}>
          <option value="eth0">eth0 (Ethernet)</option>
          <option value="wlan0">wlan0 (WiFi)</option>
        </select>
      </div>

      <div class="form-group">
        <label>Protocol</label>
        <select bind:value={protocol}>
          <option value="tcp">TCP</option>
          <option value="udp">UDP</option>
        </select>
      </div>

      <div class="form-group">
        <label>Port</label>
        <input
          type="number"
          bind:value={port}
          placeholder="Enter port number"
          min="1"
          max="65535"
        />
      </div>

      <div class="form-group">
        <label>Action</label>
        <select bind:value={action}>
          <option value="allow">Allow</option>
          <option value="block">Block</option>
        </select>
      </div>

      <button class="btn btn-primary" on:click={applyRule}>
        Apply Rule
      </button>
    </div>

    <div class="card">
      <h3>Default Rules</h3>
      <ul class="default-rules">
        <li>📩 Port 22 (SSH) - ALLOW on eth0</li>
        <li>🌐 Port 53 (DNS) - ALLOW on eth0</li>
        <li>📡 Port 67-68 (DHCP) - ALLOW on eth0</li>
        <li>🔗 Port 80 (HTTP) - ALLOW on eth0</li>
        <li>🔐 Port 443 (HTTPS) - ALLOW on eth0</li>
        <li>📶 ICMP (Ping) - ALLOW</li>
        <li>🚫 All other INPUT - DROP</li>
      </ul>
    </div>
  </div>

  <div class="card">
    <h3>Current Rules</h3>
    {#if rules}
      <pre class="rules-output">{rules}</pre>
    {:else}
      <p class="muted">Click 'Refresh' to load firewall rules</p>
    {/if}
  </div>
</div>

<style>
  .firewall-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .panel-header h2 {
    margin: 0;
  }

  .buttons {
    display: flex;
    gap: 0.5rem;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .card {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1.5rem;
  }

  .card h3 {
    margin-top: 0;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    color: #cbd5e1;
    font-size: 0.9rem;
    font-weight: 500;
  }

  input,
  select {
    width: 100%;
    padding: 0.75rem;
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 6px;
    color: #e2e8f0;
    font-size: 0.9rem;
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: #60a5fa;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }

  .btn {
    padding: 0.625rem 1rem;
    background: #334155;
    color: #e2e8f0;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn:hover:not(:disabled) {
    background: #475569;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #3b82f6;
    color: white;
  }

  .btn-primary:hover {
    background: #2563eb;
  }

  .btn-success {
    background: #22c55e;
    color: white;
  }

  .btn-success:hover {
    background: #16a34a;
  }

  .default-rules {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .default-rules li {
    padding: 0.5rem;
    background: #0f172a;
    border-left: 3px solid #60a5fa;
    color: #cbd5e1;
    font-size: 0.9rem;
  }

  .rules-output {
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 6px;
    padding: 1rem;
    color: #60a5fa;
    font-family: 'Courier New', monospace;
    font-size: 0.8rem;
    overflow-x: auto;
    max-height: 400px;
    margin: 0;
  }

  .error-message {
    padding: 1rem;
    background: #7f1d1d;
    border: 1px solid #991b1b;
    border-radius: 6px;
    color: #fca5a5;
  }

  .muted {
    color: #64748b;
    font-style: italic;
  }
</style>
