<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  
  const dispatch = createEventDispatcher()
  
  let username = 'admin'
  let password = ''
  let error = ''
  let loading = false

  async function handleLogin() {
    error = ''
    loading = true
    
    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password })
      })
      
      const data = await response.json()
      
      if (!response.ok) {
        error = data.error || 'Login failed'
        return
      }
      
      dispatch('login', { token: data.data.access_token })
    } catch (err: any) {
      error = err.message || 'Connection failed'
    } finally {
      loading = false
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleLogin()
  }
</script>

<div class="login-container">
  <div class="login-card">
    <div class="logo">🔌</div>
    <h1>PiWifi</h1>
    <p class="subtitle">WiFi Router Control Panel</p>
    
    <form on:submit|preventDefault={handleLogin}>
      <input
        type="text"
        placeholder="Username"
        bind:value={username}
        on:keydown={handleKeydown}
        disabled={loading}
        autocomplete="username"
      />
      <input
        type="password"
        placeholder="Password"
        bind:value={password}
        on:keydown={handleKeydown}
        disabled={loading}
        autocomplete="current-password"
      />
      
      {#if error}
        <div class="error">{error}</div>
      {/if}
      
      <button type="submit" disabled={loading}>
        {loading ? 'Logging in...' : 'Login'}
      </button>
    </form>
    
    <p class="hint">Default: admin / piwifi</p>
  </div>
</div>

<style>
  .login-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  }

  .login-card {
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 12px;
    padding: 2rem;
    width: 100%;
    max-width: 350px;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.3);
  }

  .logo {
    font-size: 3rem;
    text-align: center;
    margin-bottom: 1rem;
  }

  h1 {
    margin: 0;
    font-size: 2rem;
    text-align: center;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .subtitle {
    text-align: center;
    color: #94a3b8;
    margin-bottom: 1.5rem;
    font-size: 0.9rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  input {
    padding: 0.75rem;
    border: 1px solid #475569;
    border-radius: 6px;
    background: #0f172a;
    color: #e2e8f0;
    font-size: 1rem;
    transition: all 0.2s;
  }

  input:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button {
    padding: 0.75rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 10px 15px -3px rgba(102, 126, 234, 0.2);
  }

  button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .error {
    background: #7f1d1d;
    border: 1px solid #991b1b;
    color: #fca5a5;
    padding: 0.75rem;
    border-radius: 6px;
    font-size: 0.9rem;
  }

  .hint {
    text-align: center;
    color: #64748b;
    font-size: 0.8rem;
    margin-top: 1rem;
  }
</style>
