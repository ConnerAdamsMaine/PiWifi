<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let clients: any[] = []
  let loading = false
  let error = ''

  async function loadClients() {
    loading = true
    error = ''
    try {
      const response = await fetch('/api/network/clients', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        clients = data.data || []
      } else {
        error = data.error || 'Failed to load clients'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }

  function sortBy(key: string) {
    clients.sort((a, b) => {
      const aVal = a[key]
      const bVal = b[key]
      if (typeof aVal === 'string') {
        return aVal.localeCompare(bVal)
      }
      return aVal - bVal
    })
    clients = clients // trigger reactivity
  }

  onMount(() => {
    loadClients()
  })
</script>

<div class="clients-panel">
  <div class="panel-header">
    <h2>🖥️ Connected Clients</h2>
    <button class="btn btn-primary" on:click={loadClients} disabled={loading}>
      {loading ? '⏳ Loading...' : '🔄 Refresh'}
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading connected clients...</p>
  {:else if clients.length === 0}
    <p class="muted">No connected clients found</p>
  {:else}
    <div class="stats">
      <div class="stat-box">
        <span class="label">Total Devices</span>
        <span class="value">{clients.length}</span>
      </div>
    </div>

    <div class="table-wrapper">
      <table class="clients-table">
        <thead>
          <tr>
            <th on:click={() => sortBy('hostname')}>Hostname ↕</th>
            <th on:click={() => sortBy('ip')}>IP Address ↕</th>
            <th on:click={() => sortBy('mac')}>MAC Address ↕</th>
            <th on:click={() => sortBy('lease_expires')}>Lease Expires ↕</th>
          </tr>
        </thead>
        <tbody>
          {#each clients as client (client.mac)}
            <tr>
              <td class="hostname">
                {#if client.hostname === 'N/A'}
                  <span class="muted">{client.hostname}</span>
                {:else}
                  {client.hostname}
                {/if}
              </td>
              <td class="ip-address">
                <code>{client.ip}</code>
              </td>
              <td class="mac-address">
                <code>{client.mac}</code>
              </td>
              <td class="lease-expires">
                {client.lease_expires}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="info-box">
      <p>💡 Click column headers to sort. Hostnames from DHCP leases and ARP table.</p>
    </div>
  {/if}
</div>

<style>
  .clients-panel {
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

  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
  }

  .stat-box {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .stat-box .label {
    font-size: 0.9rem;
    color: #94a3b8;
  }

  .stat-box .value {
    font-size: 2rem;
    font-weight: 700;
    color: #60a5fa;
  }

  .table-wrapper {
    overflow-x: auto;
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
  }

  .clients-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  .clients-table thead {
    background: #0f172a;
    border-bottom: 2px solid #334155;
  }

  .clients-table th {
    padding: 1rem;
    text-align: left;
    color: #60a5fa;
    font-weight: 600;
    cursor: pointer;
    user-select: none;
    transition: background 0.2s;
  }

  .clients-table th:hover {
    background: #334155;
  }

  .clients-table td {
    padding: 1rem;
    border-bottom: 1px solid #334155;
    color: #e2e8f0;
  }

  .clients-table tbody tr:hover {
    background: #334155;
  }

  .hostname {
    font-weight: 500;
  }

  .ip-address,
  .mac-address {
    font-family: monospace;
    font-size: 0.85rem;
    color: #60a5fa;
  }

  .lease-expires {
    font-size: 0.85rem;
    color: #94a3b8;
  }

  code {
    background: #0f172a;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-family: monospace;
  }

  .muted {
    color: #64748b;
    font-style: italic;
  }

  .info-box {
    padding: 1rem;
    background: #1e293b;
    border-left: 4px solid #60a5fa;
    border-radius: 4px;
    color: #cbd5e1;
    font-size: 0.9rem;
  }

  .info-box p {
    margin: 0;
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
</style>
