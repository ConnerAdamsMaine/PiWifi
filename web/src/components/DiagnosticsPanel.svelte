<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  export let token: string

  type TabType = 'ping' | 'dns' | 'traceroute' | 'interfaces'

  let activeTab: TabType = 'ping'
  let input = ''
  let loading = false
  let error = ''
  let result = ''
  let interfaces: any[] = []

  async function runDiagnostic() {
    if (!input.trim() && activeTab !== 'interfaces') {
      error = 'Please enter a hostname or domain'
      return
    }

    loading = true
    error = ''
    result = ''
    interfaces = []

    try {
      let endpoint = ''
      let method = 'GET'
      let body = null

      switch (activeTab) {
        case 'ping':
          endpoint = `/api/system/diagnostics/ping?target=${encodeURIComponent(input)}`
          break
        case 'dns':
          endpoint = `/api/system/diagnostics/dns?domain=${encodeURIComponent(input)}`
          break
        case 'traceroute':
          endpoint = `/api/system/diagnostics/traceroute?target=${encodeURIComponent(input)}`
          break
        case 'interfaces':
          endpoint = `/api/system/diagnostics/interfaces`
          break
      }

      const response = await fetch(endpoint, {
        method,
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body
      })

      const data = await response.json()
      if (data.success) {
        if (activeTab === 'interfaces') {
          interfaces = data.data || []
        } else {
          result = data.data || ''
        }
        dispatch('refresh')
      } else {
        error = data.error || 'Diagnostic failed'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }

  function switchTab(tab: TabType) {
    activeTab = tab
    error = ''
    result = ''
    interfaces = []
    input = ''
  }

  function getSignalColor(signal: string): string {
    if (!signal) return '#94a3b8'
    const quality = signal.toLowerCase()
    if (quality.includes('excellent')) return '#85e89d'
    if (quality.includes('good')) return '#60a5fa'
    if (quality.includes('fair')) return '#f59e0b'
    if (quality.includes('poor')) return '#ef4444'
    return '#94a3b8'
  }
</script>

<div class="diagnostics-panel">
  <div class="panel-header">
    <h2>Network Diagnostics</h2>
    <p class="subtitle">Troubleshoot network connectivity and configuration</p>
  </div>

  <div class="tabs">
    <button
      class="tab {activeTab === 'ping' ? 'active' : ''}"
      on:click={() => switchTab('ping')}
    >
      Ping
    </button>
    <button
      class="tab {activeTab === 'dns' ? 'active' : ''}"
      on:click={() => switchTab('dns')}
    >
      DNS
    </button>
    <button
      class="tab {activeTab === 'traceroute' ? 'active' : ''}"
      on:click={() => switchTab('traceroute')}
    >
      Traceroute
    </button>
    <button
      class="tab {activeTab === 'interfaces' ? 'active' : ''}"
      on:click={() => switchTab('interfaces')}
    >
      Interfaces
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  <div class="card">
    {#if activeTab !== 'interfaces'}
      <div class="form-group">
        <label>
          {activeTab === 'ping'
            ? 'Hostname or IP'
            : activeTab === 'dns'
              ? 'Domain to resolve'
              : 'Target host'}
        </label>
        <div class="input-group">
          <input
            type="text"
            bind:value={input}
            placeholder={activeTab === 'ping'
              ? 'e.g., google.com'
              : activeTab === 'dns'
                ? 'e.g., example.com'
                : 'e.g., google.com'}
            disabled={loading}
          />
          <button
            class="btn btn-primary"
            on:click={runDiagnostic}
            disabled={loading || !input.trim()}
          >
            {loading ? '⏳ Running...' : '▶ Run Diagnostic'}
          </button>
        </div>
      </div>
    {:else}
      <button
        class="btn btn-primary"
        on:click={runDiagnostic}
        disabled={loading}
      >
        {loading ? '⏳ Loading...' : '⟳ Refresh Interfaces'}
      </button>
    {/if}

    {#if loading}
      <div class="loading-spinner">
        <div class="spinner" />
        <p>Running diagnostic...</p>
      </div>
    {/if}

    {#if result}
      <div class="result-section">
        <h4>Results</h4>
        <pre class="result-output">{result}</pre>
      </div>
    {/if}

    {#if interfaces.length > 0}
      <div class="result-section">
        <h4>Network Interfaces</h4>
        <div class="table-wrapper">
          <table class="interfaces-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Status</th>
                <th>Addresses</th>
                <th>MAC</th>
              </tr>
            </thead>
            <tbody>
              {#each interfaces as iface (iface.name)}
                <tr>
                  <td class="name">{iface.name}</td>
                  <td>
                    <span
                      class="status-badge {iface.up ? 'up' : 'down'}"
                    >
                      {iface.up ? '● Up' : '○ Down'}
                    </span>
                  </td>
                  <td class="addresses">
                    {#if iface.addresses && iface.addresses.length > 0}
                      {#each iface.addresses as addr (addr)}
                        <div class="address">{addr}</div>
                      {/each}
                    {:else}
                      <span class="muted">No addresses</span>
                    {/if}
                  </td>
                  <td class="mac">{iface.mac || 'N/A'}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}

    {#if !loading && !result && interfaces.length === 0 && activeTab !== 'interfaces'}
      <p class="muted">Enter a target and click "Run Diagnostic" to begin</p>
    {/if}

    {#if !loading && !result && interfaces.length === 0 && activeTab === 'interfaces'}
      <p class="muted">Click "Refresh Interfaces" to load network interfaces</p>
    {/if}
  </div>
</div>

<style>
  .diagnostics-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .panel-header {
    margin-bottom: 0.5rem;
  }

  .panel-header h2 {
    margin: 0 0 0.25rem 0;
    color: #e2e8f0;
  }

  .subtitle {
    margin: 0;
    color: #94a3b8;
    font-size: 0.9rem;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    border-bottom: 1px solid #334155;
    overflow-x: auto;
  }

  .tab {
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    border-bottom: 3px solid transparent;
    color: #94a3b8;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .tab:hover {
    color: #cbd5e1;
  }

  .tab.active {
    color: #60a5fa;
    border-bottom-color: #3b82f6;
  }

  .card {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1.5rem;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    color: #cbd5e1;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .input-group {
    display: flex;
    gap: 0.75rem;
  }

  input {
    flex: 1;
    padding: 0.75rem;
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 6px;
    color: #e2e8f0;
    font-size: 0.9rem;
    font-family: 'Courier New', monospace;
  }

  input:focus {
    outline: none;
    border-color: #60a5fa;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }

  input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn {
    padding: 0.75rem 1.5rem;
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

  .loading-spinner {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 2rem;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #334155;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .result-section {
    margin-top: 1.5rem;
  }

  .result-section h4 {
    margin: 0 0 1rem 0;
    color: #e2e8f0;
  }

  .result-output {
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 6px;
    padding: 1rem;
    color: #60a5fa;
    font-family: 'Courier New', monospace;
    font-size: 0.85rem;
    overflow-x: auto;
    max-height: 400px;
    margin: 0;
  }

  .table-wrapper {
    overflow-x: auto;
  }

  .interfaces-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  .interfaces-table th {
    background: #0f172a;
    color: #cbd5e1;
    padding: 0.75rem;
    text-align: left;
    font-weight: 600;
    border-bottom: 2px solid #334155;
  }

  .interfaces-table td {
    padding: 0.75rem;
    border-bottom: 1px solid #334155;
    color: #e2e8f0;
  }

  .interfaces-table tbody tr:hover {
    background: #0f172a;
  }

  .name {
    font-weight: 600;
    color: #60a5fa;
    font-family: 'Courier New', monospace;
  }

  .status-badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    font-weight: 500;
    font-size: 0.85rem;
  }

  .status-badge.up {
    background: #1f4d2f;
    color: #85e89d;
  }

  .status-badge.down {
    background: #7f1d1d;
    color: #fca5a5;
  }

  .addresses {
    font-family: 'Courier New', monospace;
    font-size: 0.85rem;
  }

  .address {
    color: #60a5fa;
    margin: 0.25rem 0;
  }

  .mac {
    font-family: 'Courier New', monospace;
    color: #94a3b8;
    font-size: 0.85rem;
  }

  .error-message {
    padding: 1rem;
    background: #7f1d1d;
    border: 1px solid #991b1b;
    border-radius: 6px;
    color: #fca5a5;
    margin-bottom: 1rem;
  }

  .muted {
    color: #64748b;
    font-style: italic;
  }

  @media (max-width: 640px) {
    .tabs {
      gap: 0.25rem;
    }

    .tab {
      padding: 0.5rem 0.75rem;
      font-size: 0.85rem;
    }

    .input-group {
      flex-direction: column;
    }

    .interfaces-table th,
    .interfaces-table td {
      padding: 0.5rem;
      font-size: 0.8rem;
    }
  }
</style>
