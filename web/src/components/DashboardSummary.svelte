<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let stats = {
    connected_clients: 0,
    wifi_signal: 0,
    uptime: '—',
    cpu_temp: 0,
    ram_usage: 0,
    dhcp_leases: 0
  }
  let loading = false

  async function loadStats() {
    loading = true
    try {
      // Get system status
      const sysRes = await fetch('/api/system/status', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const sysData = await sysRes.json()
      if (sysData.success) {
        stats.uptime = sysData.data.uptime
        stats.cpu_temp = sysData.data.cpu_temp
        stats.ram_usage = sysData.data.ram_usage
      }

      // Get WiFi status
      const wifiRes = await fetch('/api/wifi/status', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const wifiData = await wifiRes.json()
      if (wifiData.success && wifiData.data.signal) {
        stats.wifi_signal = wifiData.data.signal
      }

      // Get connected clients
      const clientsRes = await fetch('/api/network/clients', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const clientsData = await clientsRes.json()
      if (clientsData.success) {
        stats.connected_clients = (clientsData.data || []).length
      }
    } catch (err) {
      console.error('Failed to load stats:', err)
    } finally {
      loading = false
    }
  }

  onMount(() => {
    loadStats()
    const interval = setInterval(loadStats, 30000) // Refresh every 30s
    return () => clearInterval(interval)
  })

  function getSignalColor(signal: number): string {
    if (signal >= -50) return '#22c55e' // green
    if (signal >= -70) return '#eab308' // yellow
    return '#ef4444' // red
  }

  function getTempColor(temp: number): string {
    if (temp < 60) return '#60a5fa' // blue
    if (temp < 80) return '#f59e0b' // orange
    return '#ef4444' // red
  }
</script>

<div class="summary">
  <div class="stat-card">
    <div class="stat-label">Connected Devices</div>
    <div class="stat-value">{stats.connected_clients}</div>
    <div class="stat-icon">🖥️</div>
  </div>

  <div class="stat-card">
    <div class="stat-label">WiFi Signal</div>
    <div class="stat-value" style="color: {getSignalColor(stats.wifi_signal)}">
      {stats.wifi_signal} dBm
    </div>
    <div class="stat-icon">📶</div>
  </div>

  <div class="stat-card">
    <div class="stat-label">CPU Temp</div>
    <div class="stat-value" style="color: {getTempColor(stats.cpu_temp)}">
      {stats.cpu_temp.toFixed(1)}°C
    </div>
    <div class="stat-icon">🌡️</div>
  </div>

  <div class="stat-card">
    <div class="stat-label">RAM Usage</div>
    <div class="stat-value">{stats.ram_usage}%</div>
    <div class="stat-icon">💾</div>
  </div>

  <div class="stat-card">
    <div class="stat-label">Uptime</div>
    <div class="stat-value small">{stats.uptime}</div>
    <div class="stat-icon">⏱️</div>
  </div>

  <button class="refresh-btn" on:click={loadStats} disabled={loading} title="Refresh stats">
    {loading ? '...' : '↻'}
  </button>
</div>

<style>
  .summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
    position: relative;
  }

  .stat-card {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    position: relative;
  }

  .stat-icon {
    position: absolute;
    top: 8px;
    right: 8px;
    font-size: 1.2rem;
    opacity: 0.6;
  }

  .stat-label {
    font-size: 0.75rem;
    color: #94a3b8;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 600;
    text-align: center;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: #60a5fa;
  }

  .stat-value.small {
    font-size: 0.95rem;
  }

  .refresh-btn {
    position: absolute;
    bottom: 1rem;
    right: 1rem;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #3b82f6;
    color: white;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .refresh-btn:hover:not(:disabled) {
    background: #2563eb;
    transform: rotate(180deg);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (max-width: 768px) {
    .summary {
      grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    }

    .stat-value {
      font-size: 1.2rem;
    }

    .refresh-btn {
      position: static;
      width: 100%;
    }
  }
</style>
