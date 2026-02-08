<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let showBackupText = false
  let backupJson = ''
  let restoreJson = ''
  let loading = false
  let error = ''
  let successMessage = ''

  async function createBackup() {
    loading = true
    error = ''
    try {
      const response = await fetch('/api/config/backup', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()
      if (data.success) {
        backupJson = data.data
        showBackupText = true
        successMessage = '✓ Backup created successfully'
        setTimeout(() => (successMessage = ''), 3000)
      } else {
        error = data.error || 'Backup failed'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      loading = false
    }
  }

  function downloadBackup() {
    const element = document.createElement('a')
    element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(backupJson))
    element.setAttribute('download', `piwifi-backup-${new Date().toISOString().split('T')[0]}.json`)
    element.style.display = 'none'
    document.body.appendChild(element)
    element.click()
    document.body.removeChild(element)
  }

  function copyBackupToClipboard() {
    navigator.clipboard.writeText(backupJson)
    successMessage = '✓ Copied to clipboard'
    setTimeout(() => (successMessage = ''), 2000)
  }

  async function restoreFromBackup() {
    if (!restoreJson.trim()) {
      error = 'Please paste a backup JSON'
      return
    }

    loading = true
    error = ''

    try {
      const backupData = JSON.parse(restoreJson)
      const response = await fetch('/api/config/restore', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify(backupData)
      })

      const data = await response.json()
      if (data.success) {
        successMessage = '✓ Configuration restored successfully'
        restoreJson = ''
        setTimeout(() => (successMessage = ''), 3000)
      } else {
        error = data.error || 'Restore failed'
      }
    } catch (err: any) {
      if (err instanceof SyntaxError) {
        error = 'Invalid JSON format'
      } else {
        error = err.message
      }
    } finally {
      loading = false
    }
  }

  function clearRestoreText() {
    restoreJson = ''
  }
</script>

<div class="config-panel">
  <div class="panel-header">
    <h2>💾 Configuration Backup & Restore</h2>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if successMessage}
    <div class="success-message">{successMessage}</div>
  {/if}

  <div class="grid">
    <!-- Backup Section -->
    <div class="card">
      <h3>📤 Backup Configuration</h3>
      <p class="description">Export all settings (WiFi history, DHCP config, network settings)</p>

      <button class="btn btn-primary" on:click={createBackup} disabled={loading}>
        {loading ? '⏳ Creating backup...' : '📥 Create Backup'}
      </button>

      {#if showBackupText}
        <div class="backup-section">
          <p class="section-label">Backup JSON:</p>
          <textarea readonly bind:value={backupJson} class="backup-textarea"></textarea>

          <div class="button-group">
            <button class="btn btn-secondary" on:click={downloadBackup}>⬇️ Download</button>
            <button class="btn btn-secondary" on:click={copyBackupToClipboard}>📋 Copy</button>
            <button
              class="btn btn-secondary"
              on:click={() => (showBackupText = false)}
            >
              ✕ Hide
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- Restore Section -->
    <div class="card">
      <h3>📥 Restore Configuration</h3>
      <p class="description">Paste a previously created backup to restore settings</p>

      <div class="form-group">
        <label>Backup JSON:</label>
        <textarea
          bind:value={restoreJson}
          placeholder="Paste backup JSON here..."
          class="restore-textarea"
        ></textarea>
      </div>

      <div class="button-group">
        <button
          class="btn btn-primary"
          on:click={restoreFromBackup}
          disabled={loading || !restoreJson.trim()}
        >
          {loading ? '⏳ Restoring...' : '🔄 Restore'}
        </button>
        <button class="btn btn-secondary" on:click={clearRestoreText}>Clear</button>
      </div>
    </div>
  </div>

  <div class="info-box">
    <p>
      <strong>💡 How to use:</strong><br />
      1. Click "Create Backup" to export all your settings<br />
      2. Save the file somewhere safe (email, cloud, USB drive)<br />
      3. To restore: Click "Create Backup" on another Pi, paste the old backup, and click "Restore"<br />
      <br />
      <strong>⚠️ Warning:</strong> Restoring will overwrite all current settings including WiFi history and
      DHCP config
    </p>
  </div>
</div>

<style>
  .config-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
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

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 1.5rem;
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

  .description {
    color: #94a3b8;
    font-size: 0.9rem;
    margin: 0.5rem 0 1rem 0;
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

  .backup-section {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #334155;
  }

  .section-label {
    color: #94a3b8;
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
  }

  textarea {
    width: 100%;
    padding: 0.75rem;
    background: #0f172a;
    border: 1px solid #475569;
    border-radius: 6px;
    color: #e2e8f0;
    font-family: monospace;
    font-size: 0.85rem;
    resize: vertical;
  }

  .backup-textarea {
    height: 200px;
    margin-bottom: 1rem;
  }

  .restore-textarea {
    height: 250px;
  }

  textarea:focus {
    outline: none;
    border-color: #60a5fa;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }

  .button-group {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
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
  }

  .btn-secondary:hover:not(:disabled) {
    background: #4f46e5;
  }

  .info-box {
    padding: 1rem;
    background: #1e293b;
    border-left: 4px solid #60a5fa;
    border-radius: 4px;
    color: #cbd5e1;
    font-size: 0.9rem;
    line-height: 1.6;
  }

  .info-box p {
    margin: 0;
  }
</style>
