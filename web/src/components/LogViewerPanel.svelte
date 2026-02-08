<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let systemLogs: any[] = []
  let dnsmasqLogs: any[] = []
  let activeTab = 'system'
  let loading = false
  let filter = ''
  let search = ''
  let lineCount = 100
  let filteredLogs: any[] = []

  async function loadSystemLogs() {
    loading = true
    try {
      const response = await fetch(`/api/system/logs?lines=${lineCount}&filter=${filter}`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        systemLogs = data.data || []
      }
    } catch (err) {
      console.error('Failed to load system logs:', err)
    } finally {
      loading = false
    }
  }

  async function loadDnsmasqLogs() {
    loading = true
    try {
      const response = await fetch(`/api/system/logs/dnsmasq?lines=${lineCount}`, {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        dnsmasqLogs = data.data || []
      }
    } catch (err) {
      console.error('Failed to load dnsmasq logs:', err)
    } finally {
      loading = false
    }
  }

  function getLogColor(level: string): string {
    switch (level.toLowerCase()) {
      case 'error':
        return '#ef4444'
      case 'warn':
        return '#f59e0b'
      case 'info':
        return '#60a5fa'
      case 'debug':
        return '#8b5cf6'
      default:
        return '#cbd5e1'
    }
  }

  function handleTabChange(tab: string) {
    activeTab = tab
    search = ''
    if (tab === 'system') {
      loadSystemLogs()
    } else {
      loadDnsmasqLogs()
    }
  }

  function getFilteredLogs(): any[] {
    const logs = activeTab === 'system' ? systemLogs : dnsmasqLogs
    if (!search.trim()) return logs
    return logs.filter(
      (entry) =>
        entry.message.toLowerCase().includes(search.toLowerCase()) ||
        entry.level.toLowerCase().includes(search.toLowerCase())
    )
  }

  function exportLogs() {
    const logs = getFilteredLogs()
    const csv = logs
      .map((entry) => `"${entry.timestamp}","${entry.level}","${entry.message.replace(/"/g, '""')}"`)
      .join('\n')
    const header = '"Timestamp","Level","Message"\n'
    const content = header + csv

    const element = document.createElement('a')
    element.setAttribute('href', 'data:text/csv;charset=utf-8,' + encodeURIComponent(content))
    element.setAttribute('download', `logs-${activeTab}-${new Date().toISOString().split('T')[0]}.csv`)
    element.style.display = 'none'
    document.body.appendChild(element)
    element.click()
    document.body.removeChild(element)
  }

  onMount(() => {
    loadSystemLogs()
  })

  $: filteredLogs = getFilteredLogs()
</script>

<div class="log-viewer">
  <div class="panel-header">
    <h2>📋 System Logs</h2>
    <button class="btn btn-primary" on:click={() => activeTab === 'system' ? loadSystemLogs() : loadDnsmasqLogs()} disabled={loading}>
      {loading ? '⏳ Loading...' : '🔄 Refresh'}
    </button>
  </div>

  <div class="controls">
    <div class="control-group">
      <label>Lines to show:</label>
      <input type="number" bind:value={lineCount} min="10" max="500" />
    </div>
    {#if activeTab === 'system'}
      <div class="control-group">
        <label>Filter:</label>
        <input type="text" bind:value={filter} placeholder="Leave empty for all" />
        <button class="btn btn-secondary" on:click={loadSystemLogs}>Apply</button>
      </div>
    {/if}
    <div class="control-group">
      <label>Search:</label>
      <input type="text" bind:value={search} placeholder="Search in logs" />
    </div>
    <button class="btn btn-secondary" on:click={exportLogs}>📥 Export CSV</button>
  </div>

  <div class="tabs">
    <button
      class="tab-btn"
      class:active={activeTab === 'system'}
      on:click={() => handleTabChange('system')}
    >
      PiWifi System
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'dnsmasq'}
      on:click={() => handleTabChange('dnsmasq')}
    >
      DHCP/DNS (dnsmasq)
    </button>
  </div>

  <div class="logs-container">
    {#if systemLogs.length === 0 && dnsmasqLogs.length === 0 && !loading}
      <p class="muted">No logs found. Click Refresh to load.</p>
    {:else if loading}
      <p class="muted">Loading logs...</p>
    {:else if activeTab === 'system'}
      <div class="log-entries">
        {#each filteredLogs as entry (entry.timestamp)}
          <div class="log-entry">
            <span class="timestamp">{entry.timestamp}</span>
            <span class="level" style="color: {getLogColor(entry.level)}">
              [{entry.level.toUpperCase()}]
            </span>
            <span class="message">{entry.message}</span>
          </div>
        {/each}
        {#if filteredLogs.length === 0 && search}
          <p class="muted">No logs match your search.</p>
        {/if}
      </div>
    {:else}
      <div class="log-entries">
        {#each filteredLogs as entry (entry.timestamp)}
          <div class="log-entry">
            <span class="timestamp">{entry.timestamp}</span>
            <span class="level" style="color: {getLogColor(entry.level)}">
              [{entry.level.toUpperCase()}]
            </span>
            <span class="message">{entry.message}</span>
          </div>
        {/each}
        {#if filteredLogs.length === 0 && search}
          <p class="muted">No logs match your search.</p>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .log-viewer {
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

  .controls {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    padding: 1rem;
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
  }

  .control-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .control-group label {
    font-size: 0.9rem;
    color: #cbd5e1;
  }

  .control-group input {
    padding: 0.5rem;
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 4px;
    color: #e2e8f0;
    font-size: 0.9rem;
    width: 80px;
  }

  .control-group input[type='text'] {
    width: 150px;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    border-bottom: 1px solid #334155;
  }

  .tab-btn {
    padding: 0.75rem 1rem;
    background: transparent;
    color: #94a3b8;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: all 0.2s;
    font-weight: 500;
  }

  .tab-btn:hover {
    color: #e2e8f0;
  }

  .tab-btn.active {
    color: #60a5fa;
    border-bottom-color: #60a5fa;
  }

  .logs-container {
    background: #0f172a;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1rem;
    max-height: 600px;
    overflow-y: auto;
    font-family: monospace;
    font-size: 0.85rem;
  }

  .log-entries {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .log-entry {
    display: flex;
    gap: 0.75rem;
    padding: 0.5rem;
    background: #1e293b;
    border-radius: 4px;
    border-left: 3px solid #334155;
    word-break: break-word;
  }

  .timestamp {
    color: #64748b;
    flex-shrink: 0;
    min-width: 150px;
  }

  .level {
    flex-shrink: 0;
    font-weight: 600;
    min-width: 70px;
  }

  .message {
    color: #cbd5e1;
    flex-grow: 1;
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
    padding: 0.5rem 0.75rem;
    font-size: 0.85rem;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #4f46e5;
  }

  .muted {
    color: #64748b;
    font-style: italic;
    text-align: center;
    padding: 2rem;
  }
</style>
