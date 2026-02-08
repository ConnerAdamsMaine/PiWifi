<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let devices: any[] = []
  let loading = false
  let error = ''
  let successMessage = ''
  let editingMac = ''
  let editingAlias = ''
  let showStaticIpForm = false
  let staticIpForm = {
    mac: '',
    ip: '',
    hostname: ''
  }

  async function loadDevices() {
    loading = true
    error = ''
    try {
      const response = await fetch('/api/devices', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        devices = data.data || []
      } else {
        error = data.error || 'Failed to load devices'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }

  async function setAlias(mac: string, alias: string) {
    if (!alias.trim()) return
    try {
      const response = await fetch(`/api/devices/${mac}/alias`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ alias })
      })
      const data = await response.json()
      if (data.success) {
        successMessage = `✓ Renamed to "${alias}"`
        editingMac = ''
        editingAlias = ''
        await loadDevices()
        setTimeout(() => (successMessage = ''), 2000)
      } else {
        error = data.error || 'Failed to set alias'
      }
    } catch (err: any) {
      error = err.message
    }
  }

  async function setStaticIp() {
    if (!staticIpForm.mac || !staticIpForm.ip || !staticIpForm.hostname) {
      error = 'Please fill all fields'
      return
    }
    try {
      const response = await fetch('/api/dhcp/static', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify(staticIpForm)
      })
      const data = await response.json()
      if (data.success) {
        successMessage = `✓ Static IP assigned: ${staticIpForm.ip}`
        showStaticIpForm = false
        staticIpForm = { mac: '', ip: '', hostname: '' }
        await loadDevices()
        setTimeout(() => (successMessage = ''), 3000)
      } else {
        error = data.error || 'Failed to set static IP'
      }
    } catch (err: any) {
      error = err.message
    }
  }

  function startEdit(mac: string, currentAlias: string) {
    editingMac = mac
    editingAlias = currentAlias || ''
  }

  function cancelEdit() {
    editingMac = ''
    editingAlias = ''
  }

  onMount(() => {
    loadDevices()
  })
</script>

<div class="device-panel">
  <div class="panel-header">
    <h2>Device Management</h2>
    <button class="btn btn-primary" on:click={loadDevices} disabled={loading}>
      {loading ? '⏳ Loading...' : '🔄 Refresh'}
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if successMessage}
    <div class="success-message">{successMessage}</div>
  {/if}

  <div class="content">
    {#if devices.length === 0}
      <p class="muted">No devices found</p>
    {:else}
      <div class="devices-list">
        {#each devices as device (device.mac)}
          <div class="device-card">
            <div class="device-header">
              <div class="device-info">
                <div class="device-name">
                  {#if editingMac === device.mac}
                    <input
                      type="text"
                      bind:value={editingAlias}
                      placeholder="Enter device name"
                      class="alias-input"
                    />
                  {:else}
                    <span class="alias">
                      {device.alias || '(no alias)'}
                    </span>
                  {/if}
                </div>
                <div class="device-details">
                  <span class="detail">{device.ip}</span>
                  <span class="detail-sep">•</span>
                  <span class="detail">{device.mac}</span>
                  {#if device.vendor}
                    <span class="detail-sep">•</span>
                    <span class="detail vendor">{device.vendor}</span>
                  {/if}
                </div>
              </div>
              <div class="device-actions">
                {#if editingMac === device.mac}
                  <button
                    class="btn btn-small btn-success"
                    on:click={() => setAlias(device.mac, editingAlias)}
                  >
                    Save
                  </button>
                  <button class="btn btn-small" on:click={cancelEdit}>Cancel</button>
                {:else}
                  <button
                    class="btn btn-small"
                    on:click={() => startEdit(device.mac, device.alias)}
                  >
                    ✏️ Rename
                  </button>
                {/if}
                <button
                  class="btn btn-small"
                  on:click={() => {
                    staticIpForm.mac = device.mac
                    staticIpForm.hostname = device.alias || device.hostname
                    showStaticIpForm = true
                  }}
                >
                  🔗 Static IP
                </button>
              </div>
            </div>

            {#if device.is_static}
              <div class="static-badge">📌 Static IP</div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  {#if showStaticIpForm}
    <div class="modal-overlay" on:click={() => (showStaticIpForm = false)}>
      <div class="modal" on:click|stopPropagation>
        <h3>Assign Static IP</h3>

        <div class="form-group">
          <label>MAC Address</label>
          <input type="text" value={staticIpForm.mac} disabled class="input-disabled" />
        </div>

        <div class="form-group">
          <label>IP Address</label>
          <input
            type="text"
            bind:value={staticIpForm.ip}
            placeholder="e.g., 192.168.100.50"
          />
        </div>

        <div class="form-group">
          <label>Hostname</label>
          <input
            type="text"
            bind:value={staticIpForm.hostname}
            placeholder="e.g., my-printer"
          />
        </div>

        <div class="modal-buttons">
          <button class="btn btn-primary" on:click={setStaticIp}>Assign</button>
          <button class="btn" on:click={() => (showStaticIpForm = false)}>Cancel</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .device-panel {
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

  .success-message {
    padding: 1rem;
    background: #1f4d2f;
    border: 1px solid #22863a;
    border-radius: 6px;
    color: #85e89d;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .devices-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .device-card {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .device-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
  }

  .device-info {
    flex: 1;
  }

  .device-name {
    font-weight: 600;
    color: #e2e8f0;
    font-size: 1rem;
    margin-bottom: 0.5rem;
  }

  .alias {
    display: block;
  }

  .alias-input {
    width: 100%;
    max-width: 250px;
    padding: 0.5rem;
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 4px;
    color: #e2e8f0;
    font-size: 0.95rem;
  }

  .device-details {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    font-size: 0.85rem;
  }

  .detail {
    color: #94a3b8;
    font-family: monospace;
  }

  .detail-sep {
    color: #475569;
  }

  .detail.vendor {
    background: #334155;
    padding: 0.25rem 0.5rem;
    border-radius: 3px;
    font-family: system-ui;
  }

  .device-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .static-badge {
    font-size: 0.85rem;
    color: #fbbf24;
    background: #78350f;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    width: fit-content;
  }

  .btn {
    padding: 0.5rem 0.75rem;
    background: #334155;
    color: #e2e8f0;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
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

  .btn-small {
    padding: 0.375rem 0.625rem;
    font-size: 0.8rem;
  }

  .btn-success {
    background: #22c55e;
    color: white;
  }

  .btn-success:hover {
    background: #16a34a;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 2rem;
    max-width: 400px;
    width: 90%;
  }

  .modal h3 {
    margin-top: 0;
    color: #e2e8f0;
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

  .form-group input {
    width: 100%;
    padding: 0.75rem;
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 6px;
    color: #e2e8f0;
  }

  .form-group input:focus {
    outline: none;
    border-color: #60a5fa;
  }

  .input-disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .modal-buttons {
    display: flex;
    gap: 0.5rem;
    margin-top: 1.5rem;
  }

  .muted {
    color: #64748b;
    font-style: italic;
    text-align: center;
    padding: 2rem;
  }
</style>
