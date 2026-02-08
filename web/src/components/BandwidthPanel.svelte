<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let bandwidth: any[] = []
  let loading = false
  let error = ''

  async function loadBandwidth() {
    loading = true
    error = ''
    try {
      const response = await fetch('/api/network/bandwidth', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        bandwidth = data.data || []
      } else {
        error = data.error || 'Failed to load bandwidth data'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
  }

  onMount(() => {
    loadBandwidth()
  })
</script>

<div class="bandwidth-panel">
  <div class="panel-header">
    <h2>📊 Bandwidth Usage</h2>
    <button class="btn btn-primary" on:click={loadBandwidth} disabled={loading}>
      {loading ? '⏳ Loading...' : '🔄 Refresh'}
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading bandwidth data...</p>
  {:else if bandwidth.length === 0}
    <p class="muted">No bandwidth data available. Try refreshing or check if iptables is installed.</p>
  {:else}
    <div class="bandwidth-list">
      {#each bandwidth as device (device.device_mac)}
        <div class="bandwidth-item">
          <div class="device-col">
            <div class="device-name">{device.device_name || device.ip}</div>
            <div class="device-mac">{device.device_mac}</div>
          </div>

          <div class="stats-col">
            <div class="stat">
              <span class="label">Sent</span>
              <span class="value">{formatBytes(device.bytes_sent)}</span>
            </div>
            <div class="stat">
              <span class="label">Received</span>
              <span class="value">{formatBytes(device.bytes_recv)}</span>
            </div>
            <div class="stat">
              <span class="label">Total</span>
              <span class="value">
                {formatBytes(device.bytes_sent + device.bytes_recv)}
              </span>
            </div>
          </div>

          <div class="packet-col">
            <div class="packets">↑ {device.packets_sent} ↓ {device.packets_recv}</div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .bandwidth-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .panel-header h2 {
    margin: 0;
  }

  .error-message {
    padding: 1rem;
    background: #7f1d1d;
    border: 1px solid #991b1b;
    border-radius: 6px;
    color: #fca5a5;
  }

  .bandwidth-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .bandwidth-item {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1rem;
    display: grid;
    grid-template-columns: 200px 1fr 150px;
    gap: 1rem;
    align-items: center;
  }

  .device-col {
    min-width: 0;
  }

  .device-name {
    font-weight: 600;
    color: #e2e8f0;
    margin-bottom: 0.25rem;
    word-break: break-word;
  }

  .device-mac {
    font-size: 0.85rem;
    color: #94a3b8;
    font-family: monospace;
  }

  .stats-col {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .stat .label {
    font-size: 0.75rem;
    color: #64748b;
    text-transform: uppercase;
  }

  .stat .value {
    font-size: 0.95rem;
    font-weight: 600;
    color: #60a5fa;
  }

  .packet-col {
    text-align: right;
  }

  .packets {
    font-size: 0.85rem;
    color: #94a3b8;
    font-family: monospace;
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

  .muted {
    color: #64748b;
    font-style: italic;
    text-align: center;
    padding: 2rem;
  }

  @media (max-width: 1024px) {
    .bandwidth-item {
      grid-template-columns: 1fr;
      gap: 0.75rem;
    }

    .stats-col {
      grid-template-columns: repeat(3, 1fr);
    }

    .packet-col {
      text-align: left;
    }
  }
</style>
