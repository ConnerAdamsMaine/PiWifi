<script lang="ts">
  let testing = false
  let testResult = null
  let error = ''
  let successMessage = ''

  export let token: string

  async function runSpeedTest() {
    testing = true
    error = ''
    successMessage = ''
    testResult = null

    try {
      const response = await fetch('/api/speedtest/run', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}` }
      })
      const data = await response.json()

      if (data.success) {
        testResult = data.data
        successMessage = '✓ Speed test completed'
        setTimeout(() => (successMessage = ''), 3000)
      } else {
        error = data.error || 'Speed test failed'
      }
    } catch (err: any) {
      error = err.message
    } finally {
      testing = false
    }
  }
</script>

<div class="speedtest-panel">
  <div class="panel-header">
    <h2>⚡ Speed Test</h2>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if successMessage}
    <div class="success-message">{successMessage}</div>
  {/if}

  <div class="content">
    <button
      class="btn btn-primary btn-large"
      on:click={runSpeedTest}
      disabled={testing}
    >
      {testing ? '🔄 Testing... (30-60s)' : '⚡ Run Speed Test'}
    </button>

    {#if testResult}
      <div class="results">
        <div class="result-card">
          <div class="result-label">Download</div>
          <div class="result-value">{testResult.download_mbps.toFixed(2)}</div>
          <div class="result-unit">Mbps</div>
        </div>

        <div class="result-card">
          <div class="result-label">Upload</div>
          <div class="result-value">{testResult.upload_mbps.toFixed(2)}</div>
          <div class="result-unit">Mbps</div>
        </div>

        <div class="result-card">
          <div class="result-label">Ping</div>
          <div class="result-value">{testResult.ping_ms.toFixed(1)}</div>
          <div class="result-unit">ms</div>
        </div>

        <div class="result-card">
          <div class="result-label">Timestamp</div>
          <div class="result-timestamp">
            {new Date(testResult.timestamp).toLocaleString()}
          </div>
        </div>
      </div>

      <div class="info-box">
        <p>
          💡 <strong>Interpretation:</strong><br/>
          • Download: Speed to receive data (typical 10-100+ Mbps)<br/>
          • Upload: Speed to send data (typical 5-50 Mbps)<br/>
          • Ping: Latency (lower is better, ideal &lt;50ms)<br/>
          <br/>
          💥 Requires <code>speedtest-cli</code> installed. Install with:<br/>
          <code>sudo pip3 install speedtest-cli</code>
        </p>
      </div>
    {/if}
  </div>
</div>

<style>
  .speedtest-panel {
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
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    align-items: center;
  }

  .btn {
    padding: 1rem 2rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1.1rem;
    font-weight: 600;
    transition: all 0.3s;
  }

  .btn:hover:not(:disabled) {
    background: #2563eb;
    transform: scale(1.05);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-large {
    width: 100%;
    max-width: 300px;
    padding: 1.5rem 2rem;
    font-size: 1.2rem;
  }

  .results {
    width: 100%;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
  }

  .result-card {
    background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
    border: 1px solid #475569;
    border-radius: 8px;
    padding: 1.5rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .result-label {
    font-size: 0.9rem;
    color: #94a3b8;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 600;
  }

  .result-value {
    font-size: 2.5rem;
    font-weight: 700;
    color: #60a5fa;
  }

  .result-unit {
    font-size: 0.85rem;
    color: #64748b;
  }

  .result-timestamp {
    font-size: 0.85rem;
    color: #94a3b8;
    font-family: monospace;
  }

  .info-box {
    width: 100%;
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
</style>
