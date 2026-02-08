<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let devices: any[] = []
  let selectedMac = ''
  let loading = false
  let error = ''
  let successMessage = ''
  let manualMac = ''

  async function loadDevices() {
    try {
      const response = await fetch('/api/devices', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        devices = data.data || []
      }
    } catch (err) {
      console.error('Failed to load devices:', err)
    }
  }

  async function wakeDevice(mac: string) {
    if (!mac) {
      error = 'Please select or enter a MAC address'
      return
    }

    loading = true
    error = ''
    successMessage = ''

    try {
      const response = await fetch(`/api/network/wake/${mac}`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()

      if (data.success) {
        successMessage = `✓ Magic packet sent to ${mac}`
        selectedMac = ''
        manualMac = ''
        setTimeout(() => (successMessage = ''), 3000)
      } else {
        error = data.error || 'Failed to send magic packet'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }

  onMount(() => {
    loadDevices()
  })
</script>

<div class="wol-panel">
  <div class="panel-header">
    <h2>🔌 Wake-on-LAN</h2>
    <button class="btn btn-primary" on:click={loadDevices}>↻ Refresh Devices</button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if successMessage}
    <div class="success-message">{successMessage}</div>
  {/if}

  <div class="content">
    <div class="section">
      <h3>From Device List</h3>
      {#if devices.length === 0}
        <p class="muted">No devices found. Make sure DHCP is active.</p>
      {:else}
        <div class="device-selector">
          {#each devices as device (device.mac)}
            <button
              class="device-btn"
              class:selected={selectedMac === device.mac}
              on:click={() => (selectedMac = device.mac)}
            >
              <div class="device-name">{device.alias || device.hostname || 'Unknown'}</div>
              <div class="device-mac">{device.mac}</div>
            </button>
          {/each}
        </div>

        <button
          class="btn btn-primary btn-wide"
          on:click={() => wakeDevice(selectedMac)}
          disabled={!selectedMac || loading}
        >
          {loading ? '⏳ Sending...' : '🔌 Wake Selected Device'}
        </button>
      {/if}
    </div>

    <div class="divider">OR</div>

    <div class="section">
      <h3>Manual MAC Address</h3>
      <div class="form-group">
        <label>MAC Address</label>
        <input
          type="text"
          bind:value={manualMac}
          placeholder="AA:BB:CC:DD:EE:FF or AABBCCDDEEFF"
          class="mac-input"
        />
      </div>

      <button
        class="btn btn-primary btn-wide"
        on:click={() => wakeDevice(manualMac)}
        disabled={!manualMac || loading}
      >
        {loading ? '⏳ Sending...' : '🔌 Send Magic Packet'}
      </button>
    </div>

    <div class="info-box">
      <p>
        <strong>💡 How Wake-on-LAN works:</strong><br/>
        Sends a special "magic packet" to a device's MAC address, waking it up from sleep or
        standby mode.<br/>
        <br/>
        <strong>Requirements:</strong><br/>
        • Device must support WoL (most modern computers/servers do)<br/>
        • WoL must be enabled in device BIOS/UEFI<br/>
        • Device must be plugged in and not powered off<br/>
        • Device on same network or broadcast address<br/>
        <br/>
        <strong>MAC Address Formats:</strong><br/>
        <code>AA:BB:CC:DD:EE:FF</code> or <code>AABBCCDDEEFF</code>
      </p>
    </div>
  </div>
</div>

<style>
  .wol-panel {
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
    gap: 2rem;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .section h3 {
    margin: 0;
    color: #e2e8f0;
    font-size: 1rem;
  }

  .divider {
    text-align: center;
    color: #64748b;
    font-weight: 600;
    padding: 1rem 0;
    border-top: 1px solid #334155;
    border-bottom: 1px solid #334155;
  }

  .device-selector {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .device-btn {
    padding: 1rem;
    background: #1e293b;
    border: 2px solid #334155;
    border-radius: 6px;
    color: #e2e8f0;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .device-btn:hover {
    border-color: #60a5fa;
    background: #334155;
  }

  .device-btn.selected {
    border-color: #3b82f6;
    background: #1e40af;
    color: white;
  }

  .device-name {
    font-weight: 600;
  }

  .device-mac {
    font-size: 0.8rem;
    opacity: 0.8;
    font-family: monospace;
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

  .mac-input {
    padding: 0.75rem;
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 6px;
    color: #e2e8f0;
    font-family: monospace;
    font-size: 0.95rem;
  }

  .mac-input:focus {
    outline: none;
    border-color: #60a5fa;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }

  .btn {
    padding: 0.75rem 1.5rem;
    background: #334155;
    color: #e2e8f0;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 600;
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

  .btn-wide {
    width: 100%;
  }

  .info-box {
    padding: 1.5rem;
    background: #1e293b;
    border-left: 4px solid #60a5fa;
    border-radius: 6px;
    color: #cbd5e1;
    font-size: 0.9rem;
    line-height: 1.6;
  }

  .info-box p {
    margin: 0;
  }

  code {
    background: #0f172a;
    padding: 0.25rem 0.5rem;
    border-radius: 3px;
    font-family: monospace;
    color: #60a5fa;
  }

  .muted {
    color: #64748b;
    font-style: italic;
  }
</style>
