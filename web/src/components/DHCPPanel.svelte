<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte'

  const dispatch = createEventDispatcher()

  export let token: string

  let config = {
    dhcp_start: '',
    dhcp_end: '',
    lease_time: 0,
    dns_servers: '',
    local_domain: ''
  }

  let originalConfig = { ...config }
  let loading = false
  let saving = false
  let restarting = false
  let error = ''
  let successMessage = ''

  onMount(() => {
    loadConfiguration()
  })

  async function loadConfiguration() {
    loading = true
    error = ''
    try {
      const response = await fetch('/api/dhcp/config', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        config = {
          dhcp_start: data.data.dhcp_start || '',
          dhcp_end: data.data.dhcp_end || '',
          lease_time: data.data.lease_time || 0,
          dns_servers: data.data.dns_servers || '',
          local_domain: data.data.local_domain || ''
        }
        originalConfig = { ...config }
      } else {
        error = data.error || 'Failed to load configuration'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }

  async function saveConfiguration() {
    if (!config.dhcp_start || !config.dhcp_end) {
      error = 'DHCP Start IP and End IP are required'
      return
    }

    if (!config.lease_time || config.lease_time <= 0) {
      error = 'Lease Time must be greater than 0'
      return
    }

    saving = true
    error = ''
    successMessage = ''

    try {
      const response = await fetch('/api/dhcp/configure', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
          dhcp_start: config.dhcp_start,
          dhcp_end: config.dhcp_end,
          lease_time: config.lease_time,
          dns_servers: config.dns_servers
            .split(',')
            .map((s: string) => s.trim())
            .filter((s: string) => s.length > 0),
          local_domain: config.local_domain
        })
      })

      const data = await response.json()
      if (data.success) {
        successMessage = '✓ Configuration saved successfully'
        originalConfig = { ...config }
        dispatch('refresh')
        await new Promise(resolve => setTimeout(resolve, 2000))
        successMessage = ''
      } else {
        error = data.error || 'Failed to save configuration'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      saving = false
    }
  }

  async function restartDnsmasq() {
    if (!confirm('Are you sure you want to restart dnsmasq? Network services may be interrupted.')) {
      return
    }

    restarting = true
    error = ''
    successMessage = ''

    try {
      const response = await fetch('/api/dhcp/restart', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })

      const data = await response.json()
      if (data.success) {
        successMessage = '✓ dnsmasq restarted successfully'
        dispatch('refresh')
        await new Promise(resolve => setTimeout(resolve, 2000))
        successMessage = ''
      } else {
        error = data.error || 'Failed to restart dnsmasq'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      restarting = false
    }
  }

  function hasChanges(): boolean {
    return JSON.stringify(config) !== JSON.stringify(originalConfig)
  }

  function resetForm() {
    config = { ...originalConfig }
    error = ''
  }
</script>

<div class="dhcp-panel">
  <div class="panel-header">
    <h2>DHCP & DNS Configuration</h2>
    <p class="subtitle">Manage dnsmasq DHCP and DNS server settings</p>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if successMessage}
    <div class="success-message">{successMessage}</div>
  {/if}

  <div class="card">
    <h3>Current Configuration</h3>

    {#if loading}
      <div class="loading-spinner">
        <div class="spinner" />
        <p>Loading configuration...</p>
      </div>
    {:else}
      <form on:submit|preventDefault={saveConfiguration}>
        <div class="form-row">
          <div class="form-group">
            <label for="dhcp-start">DHCP Start IP</label>
            <input
              id="dhcp-start"
              type="text"
              bind:value={config.dhcp_start}
              placeholder="e.g., 192.168.100.50"
              disabled={saving || restarting}
            />
          </div>

          <div class="form-group">
            <label for="dhcp-end">DHCP End IP</label>
            <input
              id="dhcp-end"
              type="text"
              bind:value={config.dhcp_end}
              placeholder="e.g., 192.168.100.200"
              disabled={saving || restarting}
            />
          </div>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="lease-time">Lease Time (seconds)</label>
            <input
              id="lease-time"
              type="number"
              bind:value={config.lease_time}
              placeholder="e.g., 3600"
              min="60"
              disabled={saving || restarting}
            />
            <small>Minimum 60 seconds</small>
          </div>

          <div class="form-group">
            <label for="local-domain">Local Domain</label>
            <input
              id="local-domain"
              type="text"
              bind:value={config.local_domain}
              placeholder="e.g., piwifi.local"
              disabled={saving || restarting}
            />
          </div>
        </div>

        <div class="form-group">
          <label for="dns-servers">DNS Servers (comma-separated)</label>
          <input
            id="dns-servers"
            type="text"
            bind:value={config.dns_servers}
            placeholder="e.g., 8.8.8.8, 8.8.4.4"
            disabled={saving || restarting}
          />
          <small>Leave empty to use system default</small>
        </div>

        <div class="button-group">
          <div class="primary-buttons">
            <button
              type="submit"
              class="btn btn-primary"
              disabled={saving || restarting || !hasChanges()}
            >
              {saving ? '⏳ Saving...' : '💾 Save Configuration'}
            </button>

            <button
              type="button"
              class="btn btn-secondary"
              on:click={resetForm}
              disabled={saving || restarting || !hasChanges()}
            >
              ↺ Reset
            </button>
          </div>

          <button
            type="button"
            class="btn btn-danger"
            on:click={restartDnsmasq}
            disabled={saving || restarting}
          >
            {restarting ? '⏳ Restarting...' : '⟲ Restart dnsmasq'}
          </button>
        </div>
      </form>
    {/if}
  </div>

  <div class="card">
    <h3>Information</h3>
    <div class="info-grid">
      <div class="info-item">
        <span class="label">Service:</span>
        <span class="value">dnsmasq</span>
      </div>
      <div class="info-item">
        <span class="label">Configuration File:</span>
        <span class="value">/etc/dnsmasq.conf</span>
      </div>
      <div class="info-item">
        <span class="label">Purpose:</span>
        <span class="value">DHCP & DNS server</span>
      </div>
    </div>
  </div>
</div>

<style>
  .dhcp-panel {
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

  form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .form-group label {
    color: #cbd5e1;
    font-size: 0.9rem;
    font-weight: 500;
  }

  input {
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

  small {
    color: #94a3b8;
    font-size: 0.8rem;
    margin-top: -0.25rem;
  }

  .button-group {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .primary-buttons {
    display: flex;
    gap: 0.75rem;
    flex: 1;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .btn:hover:not(:disabled) {
    transform: translateY(-2px);
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
    background: #334155;
    color: #e2e8f0;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #475569;
  }

  .btn-danger {
    background: #ef4444;
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
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
    font-family: 'Courier New', monospace;
  }

  @media (max-width: 768px) {
    .form-row {
      grid-template-columns: 1fr;
    }

    .button-group {
      flex-direction: column;
      align-items: stretch;
    }

    .primary-buttons {
      flex-direction: column;
    }

    .btn {
      width: 100%;
    }

    .card {
      padding: 1rem;
    }
  }
</style>
