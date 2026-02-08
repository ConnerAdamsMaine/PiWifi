<script lang="ts">
  import { onMount } from 'svelte'
  import { createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  export let token: string

  let networks: any[] = []
  let status: any = null
  let scanning = false
  let connecting = false
  let selectedSSID = ''
  let password = ''
  let error = ''
  let successMessage = ''

  onMount(() => {
    // Auto-load status and scan networks on component mount
    getStatus()
    scanNetworks()
  })

  async function scanNetworks() {
    scanning = true
    error = ''
    try {
      const response = await fetch('/api/wifi/scan', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        networks = data.data || []
        networks.sort((a, b) => b.signal - a.signal)
      } else {
        error = data.error || 'Scan failed'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      scanning = false
    }
  }

  async function getStatus() {
    try {
      const response = await fetch('/api/wifi/status', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        status = data.data
      }
    } catch (err) {
      console.error('Failed to get WiFi status:', err)
    }
  }

  async function connectToNetwork() {
    if (!selectedSSID || !password) {
      error = 'Please select a network and enter password'
      return
    }

    connecting = true
    error = ''
    successMessage = ''

    try {
      const response = await fetch('/api/wifi/connect', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
          ssid: selectedSSID,
          password: password
        })
      })

      const data = await response.json()
      if (data.success) {
        successMessage = `✓ Connecting to ${selectedSSID}...`
        password = ''
        selectedSSID = ''
        await new Promise(resolve => setTimeout(resolve, 3000))
        await getStatus()
        successMessage = ''
      } else {
        error = data.error || 'Connection failed'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      connecting = false
    }
  }

  async function disconnect() {
    try {
      const response = await fetch('/api/wifi/disconnect', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        await getStatus()
      } else {
        error = data.error || 'Disconnect failed'
      }
    } catch (err: any) {
      error = err.message
    }
  }

  function selectNetwork(ssid: string) {
    selectedSSID = ssid
  }
</script>

<div class="wifi-panel">
  <div class="panel-header">
    <h2>WiFi Management</h2>
    <div class="buttons">
      <button class="btn btn-primary" on:click={scanNetworks} disabled={scanning}>
        {scanning ? '🔄 Scanning...' : '🔍 Scan Networks'}
      </button>
      <button class="btn btn-secondary" on:click={getStatus}>↻ Status</button>
    </div>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if successMessage}
    <div class="success-message">{successMessage}</div>
  {/if}

  <div class="grid">
    <div class="card">
      <h3>Current Connection</h3>
      {#if status}
        <div class="status-info">
          <div class="info-row">
            <span class="label">Connected:</span>
            <span class="value">
              {status.connected ? '✅ Yes' : '❌ No'}
            </span>
          </div>
          {#if status.connected}
            <div class="info-row">
              <span class="label">Network:</span>
              <span class="value">{status.ssid || 'Unknown'}</span>
            </div>
            <div class="info-row">
              <span class="label">IP Address:</span>
              <span class="value">{status.ip || 'N/A'}</span>
            </div>
            {#if status.signal}
              <div class="info-row">
                <span class="label">Signal:</span>
                <span class="value">{status.signal} dBm</span>
              </div>
            {/if}
            <button class="btn btn-danger" on:click={disconnect}>
              Disconnect
            </button>
          {/if}
        </div>
      {:else}
        <p class="muted">Click 'Status' to check connection</p>
      {/if}
    </div>

    <div class="card">
      <h3>Connect to Network</h3>
      <div class="form-group">
        <label>Select Network</label>
        <select bind:value={selectedSSID}>
          <option value="">-- Choose a network --</option>
          {#each networks as network (network.ssid)}
            <option value={network.ssid}>
              {network.ssid} ({network.signal} dBm)
            </option>
          {/each}
        </select>
      </div>

      <div class="form-group">
        <label>Password</label>
        <input
          type="password"
          bind:value={password}
          placeholder="Enter WiFi password"
          disabled={connecting || !selectedSSID}
        />
      </div>

      <button
        class="btn btn-primary"
        on:click={connectToNetwork}
        disabled={connecting || !selectedSSID || !password}
      >
        {connecting ? '⏳ Connecting...' : '📡 Connect'}
      </button>
    </div>
  </div>

  <div class="card">
    <h3>Available Networks ({networks.length})</h3>
    <div class="network-list">
      {#if networks.length === 0}
        <p class="muted">No networks found. Click "Scan Networks" to search.</p>
      {:else}
        {#each networks as network (network.ssid)}
          <div class="network-item">
            <div class="network-info">
              <div class="network-name">{network.ssid}</div>
              <div class="network-details">
                <span>Signal: {network.signal} dBm</span>
                <span>Security: {network.security}</span>
              </div>
            </div>
            <button
              class="btn btn-small"
              on:click={() => selectNetwork(network.ssid)}
              disabled={!password || selectedSSID === network.ssid}
            >
              Select
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .wifi-panel {
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
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
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
    color: #e2e8f0;
  }

  .status-info {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .label {
    color: #94a3b8;
    font-weight: 500;
  }

  .value {
    color: #60a5fa;
    font-weight: 600;
    font-family: monospace;
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

  .btn-primary:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn-secondary {
    background: #6366f1;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #4f46e5;
  }

  .btn-danger {
    background: #ef4444;
    color: white;
    margin-top: 1rem;
  }

  .btn-danger:hover {
    background: #dc2626;
  }

  .btn-small {
    padding: 0.5rem 0.75rem;
    font-size: 0.85rem;
  }

  .network-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .network-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: #0f172a;
    border: 1px solid #334155;
    border-radius: 6px;
  }

  .network-info {
    flex: 1;
  }

  .network-name {
    font-weight: 600;
    color: #e2e8f0;
    margin-bottom: 0.25rem;
  }

  .network-details {
    display: flex;
    gap: 1rem;
    font-size: 0.85rem;
    color: #94a3b8;
  }

  .error-message {
    padding: 1rem;
    background: #7f1d1d;
    border: 1px solid #991b1b;
    border-radius: 6px;
    color: #fca5a5;
  }

  .success-message {
    padding: 1rem;
    background: #1f4d2f;
    border: 1px solid #22863a;
    border-radius: 6px;
    color: #85e89d;
    animation: fadeIn 0.3s ease-in;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .muted {
    color: #64748b;
    font-style: italic;
  }
</style>
