<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  export let token: string

  let status = ''
  let loading = false
  let error = ''

  async function loadStatus() {
    loading = true
    error = ''
    try {
      const response = await fetch('/api/network/status', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        status = data.data
      } else {
        error = data.error || 'Failed to load status'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }
</script>

<div class="network-panel">
  <div class="panel-header">
    <h2>Network Configuration</h2>
    <button class="btn btn-primary" on:click={loadStatus} disabled={loading}>
      {loading ? '⏳ Loading...' : '↻ Refresh'}
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  <div class="card">
    <h3>Network Status</h3>
    {#if status}
      <pre class="status-output">{status}</pre>
    {:else}
      <p class="muted">Click 'Refresh' to load network status</p>
    {/if}
  </div>

  <div class="card">
    <h3>Configuration</h3>
    <div class="info-grid">
      <div class="info-item">
        <span class="label">Ethernet IP:</span>
        <span class="value">192.168.100.1/24</span>
      </div>
      <div class="info-item">
        <span class="label">DHCP Range:</span>
        <span class="value">192.168.100.50 - 200</span>
      </div>
      <div class="info-item">
        <span class="label">DNS Domain:</span>
        <span class="value">piwifi.local</span>
      </div>
      <div class="info-item">
        <span class="label">Upstream DNS:</span>
        <span class="value">8.8.8.8, 8.8.4.4</span>
      </div>
    </div>
  </div>
</div>

<style>
  .network-panel {
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

  .card {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1.5rem;
  }

  .card h3 {
    margin-top: 0;
  }

  .status-output {
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 6px;
    padding: 1rem;
    color: #60a5fa;
    font-family: 'Courier New', monospace;
    font-size: 0.85rem;
    overflow-x: auto;
    max-height: 400px;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem;
    background: #0f172a;
    border: 1px solid #334155;
    border-radius: 6px;
  }

  .label {
    color: #94a3b8;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .value {
    color: #60a5fa;
    font-weight: 600;
    font-family: monospace;
  }

  .btn {
    padding: 0.625rem 1rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #3b82f6;
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
