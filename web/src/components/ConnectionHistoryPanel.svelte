<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let history: any[] = []
  let favorites: any[] = []
  let activeTab = 'history'
  let loading = false

  async function loadHistory() {
    loading = true
    try {
      const response = await fetch('/api/wifi/history', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        history = data.data || []
      }
    } catch (err) {
      console.error('Failed to load history:', err)
    } finally {
      loading = false
    }
  }

  async function loadFavorites() {
    loading = true
    try {
      const response = await fetch('/api/wifi/favorites', {
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        favorites = data.data || []
      }
    } catch (err) {
      console.error('Failed to load favorites:', err)
    } finally {
      loading = false
    }
  }

  async function clearHistory() {
    if (!confirm('Are you sure? This will delete all connection history.')) {
      return
    }

    try {
      const response = await fetch('/api/wifi/history/clear', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        history = []
        favorites = []
      }
    } catch (err) {
      console.error('Failed to clear history:', err)
    }
  }

  function formatTime(timestamp: string): string {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = (now.getTime() - date.getTime()) / 1000

    if (diff < 60) return 'just now'
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`
    return date.toLocaleDateString()
  }

  function formatDuration(seconds?: number): string {
    if (!seconds) return '—'
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    const s = seconds % 60
    if (h > 0) return `${h}h ${m}m`
    if (m > 0) return `${m}m ${s}s`
    return `${s}s`
  }

  onMount(() => {
    loadHistory()
  })
</script>

<div class="history-panel">
  <div class="panel-header">
    <h2>⏱️ WiFi Connection History</h2>
    <button
      class="btn btn-primary"
      on:click={() => (activeTab === 'history' ? loadHistory() : loadFavorites())}
      disabled={loading}
    >
      {loading ? '⏳ Loading...' : '🔄 Refresh'}
    </button>
  </div>

  <div class="tabs">
    <button
      class="tab-btn"
      class:active={activeTab === 'history'}
      on:click={() => {
        activeTab = 'history'
        loadHistory()
      }}
    >
      📋 Recent Connections
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'favorites'}
      on:click={() => {
        activeTab = 'favorites'
        loadFavorites()
      }}
    >
      ⭐ Favorite Networks
    </button>
  </div>

  {#if activeTab === 'history'}
    <div class="content">
      {#if history.length === 0 && !loading}
        <p class="muted">No connection history yet. Connect to a WiFi network to start tracking.</p>
      {:else if loading}
        <p class="muted">Loading history...</p>
      {:else}
        <div class="history-list">
          {#each history as entry (entry.timestamp)}
            <div class="history-item">
              <div class="item-header">
                <span class="ssid">{entry.ssid}</span>
                <span class="status" class:success={entry.success} class:failed={!entry.success}>
                  {entry.success ? '✅ Success' : '❌ Failed'}
                </span>
              </div>
              <div class="item-details">
                <span class="time">{formatTime(entry.timestamp)}</span>
                {#if entry.duration_seconds}
                  <span class="duration">Duration: {formatDuration(entry.duration_seconds)}</span>
                {/if}
                {#if entry.disconnection_reason}
                  <span class="reason">Reason: {entry.disconnection_reason}</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>

        <button class="btn btn-danger" on:click={clearHistory}>🗑️ Clear All History</button>
      {/if}
    </div>
  {/if}

  {#if activeTab === 'favorites'}
    <div class="content">
      {#if favorites.length === 0 && !loading}
        <p class="muted">No favorite networks yet. Connect to networks to build history.</p>
      {:else if loading}
        <p class="muted">Loading favorites...</p>
      {:else}
        <div class="favorites-grid">
          {#each favorites as fav}
            <div class="favorite-card">
              <div class="card-header">
                <h4>{fav.ssid}</h4>
              </div>
              <div class="card-stats">
                <div class="stat">
                  <span class="label">Connections</span>
                  <span class="value">{fav.connection_count}</span>
                </div>
                <div class="stat">
                  <span class="label">Success Rate</span>
                  <span class="value">{fav.success_rate.toFixed(1)}%</span>
                </div>
              </div>
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  style="width: {fav.success_rate}%"
                ></div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .history-panel {
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

  .content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .history-item {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1rem;
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .ssid {
    font-weight: 600;
    color: #e2e8f0;
    font-size: 1rem;
  }

  .status {
    font-size: 0.85rem;
    font-weight: 500;
  }

  .status.success {
    color: #85e89d;
  }

  .status.failed {
    color: #fca5a5;
  }

  .item-details {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    font-size: 0.9rem;
    color: #94a3b8;
  }

  .time {
    font-style: italic;
  }

  .duration,
  .reason {
    padding: 0.25rem 0.5rem;
    background: #0f172a;
    border-radius: 4px;
    font-size: 0.85rem;
  }

  .favorites-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
  }

  .favorite-card {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .card-header h4 {
    margin: 0;
    color: #e2e8f0;
    word-break: break-word;
  }

  .card-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .stat .label {
    font-size: 0.85rem;
    color: #94a3b8;
  }

  .stat .value {
    font-size: 1.5rem;
    font-weight: 700;
    color: #60a5fa;
  }

  .progress-bar {
    height: 8px;
    background: #0f172a;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #3b82f6, #60a5fa);
    transition: width 0.3s ease;
  }

  .muted {
    color: #64748b;
    font-style: italic;
    text-align: center;
    padding: 2rem 0;
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
    align-self: flex-start;
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

  .btn-danger {
    background: #ef4444;
    color: white;
  }

  .btn-danger:hover {
    background: #dc2626;
  }
</style>
