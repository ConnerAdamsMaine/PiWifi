<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import DashboardSummary from './DashboardSummary.svelte'
  import WiFiPanel from './WiFiPanel.svelte'
  import NetworkPanel from './NetworkPanel.svelte'
  import FirewallPanel from './FirewallPanel.svelte'
  import TerminalPanel from './TerminalPanel.svelte'
  import DiagnosticsPanel from './DiagnosticsPanel.svelte'
  import DHCPPanel from './DHCPPanel.svelte'
  import LogViewerPanel from './LogViewerPanel.svelte'
  import ConnectionHistoryPanel from './ConnectionHistoryPanel.svelte'
  import ClientsPanel from './ClientsPanel.svelte'
  import ConfigPanel from './ConfigPanel.svelte'
  import DeviceManagementPanel from './DeviceManagementPanel.svelte'
  import BandwidthPanel from './BandwidthPanel.svelte'
  import SpeedTestPanel from './SpeedTestPanel.svelte'
  import WakeOnLanPanel from './WakeOnLanPanel.svelte'

  const dispatch = createEventDispatcher()

  export let token: string

  let activeTab = 'wifi'
  let systemStatus = {
    uptime: '',
    cpu_temp: 0,
    ram_usage: 0,
    disk_usage: 0
  }

  function logout() {
    dispatch('logout')
  }

  async function loadSystemStatus() {
    try {
      const response = await fetch('/api/system/status', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      if (response.ok) {
        const data = await response.json()
        systemStatus = data.data
      }
    } catch (err) {
      console.error('Failed to load system status:', err)
    }
  }

  function handleRefresh() {
    loadSystemStatus()
  }
</script>

<div class="dashboard">
  <header class="header">
    <div class="header-left">
      <h1 class="title">🔌 PiWifi</h1>
      <p class="subtitle">Raspberry Pi WiFi Router</p>
    </div>
    
    <div class="header-right">
      <button class="btn-logout" on:click={logout}>Logout</button>
    </div>
  </header>

  <nav class="tabs">
    <button
      class="tab"
      class:active={activeTab === 'wifi'}
      on:click={() => (activeTab = 'wifi')}
    >
      📶 WiFi
    </button>
    <button
      class="tab"
      class:active={activeTab === 'network'}
      on:click={() => (activeTab = 'network')}
    >
      🌐 Network
    </button>
    <button
      class="tab"
      class:active={activeTab === 'firewall'}
      on:click={() => (activeTab = 'firewall')}
    >
      🛡️ Firewall
    </button>
    <button
      class="tab"
      class:active={activeTab === 'terminal'}
      on:click={() => (activeTab = 'terminal')}
    >
      ⌨️ Terminal
    </button>
    <button
      class="tab"
      class:active={activeTab === 'diagnostics'}
      on:click={() => (activeTab = 'diagnostics')}
    >
      🔍 Diagnostics
    </button>
    <button
      class="tab"
      class:active={activeTab === 'dhcp'}
      on:click={() => (activeTab = 'dhcp')}
    >
      🔧 DHCP Config
    </button>
    <button
      class="tab"
      class:active={activeTab === 'logs'}
      on:click={() => (activeTab = 'logs')}
    >
      📋 Logs
    </button>
    <button
      class="tab"
      class:active={activeTab === 'history'}
      on:click={() => (activeTab = 'history')}
    >
      ⏱️ History
    </button>
    <button
      class="tab"
      class:active={activeTab === 'clients'}
      on:click={() => (activeTab = 'clients')}
    >
      🖥️ Clients
    </button>
    <button
      class="tab"
      class:active={activeTab === 'config'}
      on:click={() => (activeTab = 'config')}
    >
      💾 Config
    </button>
    <button
      class="tab"
      class:active={activeTab === 'devices'}
      on:click={() => (activeTab = 'devices')}
    >
      🏷️ Devices
    </button>
    <button
      class="tab"
      class:active={activeTab === 'bandwidth'}
      on:click={() => (activeTab = 'bandwidth')}
    >
      📊 Bandwidth
    </button>
    <button
      class="tab"
      class:active={activeTab === 'speedtest'}
      on:click={() => (activeTab = 'speedtest')}
    >
      ⚡ Speed Test
    </button>
    <button
      class="tab"
      class:active={activeTab === 'wol'}
      on:click={() => (activeTab = 'wol')}
    >
      🔌 Wake-on-LAN
    </button>
  </nav>

  <main class="content">
    {#if activeTab !== 'logs' && activeTab !== 'history' && activeTab !== 'clients' && activeTab !== 'config' && activeTab !== 'devices' && activeTab !== 'bandwidth' && activeTab !== 'speedtest' && activeTab !== 'wol'}
      <DashboardSummary {token} />
    {/if}
    {#if activeTab === 'wifi'}
      <WiFiPanel {token} on:refresh={handleRefresh} />
    {:else if activeTab === 'network'}
      <NetworkPanel {token} on:refresh={handleRefresh} />
    {:else if activeTab === 'firewall'}
      <FirewallPanel {token} on:refresh={handleRefresh} />
    {:else if activeTab === 'terminal'}
      <TerminalPanel {token} />
    {:else if activeTab === 'diagnostics'}
      <DiagnosticsPanel {token} on:refresh={handleRefresh} />
    {:else if activeTab === 'dhcp'}
      <DHCPPanel {token} on:refresh={handleRefresh} />
    {:else if activeTab === 'logs'}
      <LogViewerPanel {token} />
    {:else if activeTab === 'history'}
      <ConnectionHistoryPanel {token} />
    {:else if activeTab === 'clients'}
      <ClientsPanel {token} />
    {:else if activeTab === 'config'}
      <ConfigPanel {token} />
    {:else if activeTab === 'devices'}
      <DeviceManagementPanel {token} />
    {:else if activeTab === 'bandwidth'}
      <BandwidthPanel {token} />
    {:else if activeTab === 'speedtest'}
      <SpeedTestPanel {token} />
    {:else if activeTab === 'wol'}
      <WakeOnLanPanel {token} />
    {/if}
  </main>
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #0f172a;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    background: #1e293b;
    border-bottom: 1px solid #334155;
  }

  .header-left {
    flex: 1;
  }

  .title {
    margin: 0;
    font-size: 1.5rem;
    color: #e2e8f0;
  }

  .subtitle {
    margin: 0;
    font-size: 0.85rem;
    color: #94a3b8;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 2rem;
  }

  .status {
    display: flex;
    gap: 1.5rem;
    font-size: 0.85rem;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .label {
    color: #94a3b8;
  }

  .value {
    color: #60a5fa;
    font-weight: 600;
  }

  .btn-logout {
    padding: 0.5rem 1rem;
    background: #7f1d1d;
    color: #fca5a5;
    border: 1px solid #991b1b;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-logout:hover {
    background: #991b1b;
  }

  .tabs {
    display: flex;
    gap: 1rem;
    padding: 1rem 1.5rem;
    background: #1e293b;
    border-bottom: 1px solid #334155;
    overflow-x: auto;
  }

  .tab {
    padding: 0.5rem 1rem;
    background: transparent;
    color: #94a3b8;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .tab:hover {
    color: #e2e8f0;
  }

  .tab.active {
    color: #60a5fa;
    border-bottom-color: #60a5fa;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }
</style>
