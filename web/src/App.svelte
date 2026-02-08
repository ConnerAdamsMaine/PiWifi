<script lang="ts">
  import { onMount } from 'svelte'
  import Login from './components/Login.svelte'
  import Dashboard from './components/Dashboard.svelte'

  let isAuthenticated = false
  let authToken = ''

  onMount(() => {
    const token = localStorage.getItem('auth_token')
    if (token) {
      authToken = token
      verifyToken()
    }
  })

  async function verifyToken() {
    try {
      const response = await fetch('/api/auth/verify', {
        headers: {
          'Authorization': `Bearer ${authToken}`
        }
      })
      if (response.ok) {
        isAuthenticated = true
      } else {
        logout()
      }
    } catch (err) {
      logout()
    }
  }

  function handleLogin(event: any) {
    authToken = event.detail.token
    localStorage.setItem('auth_token', authToken)
    isAuthenticated = true
  }

  function logout() {
    isAuthenticated = false
    authToken = ''
    localStorage.removeItem('auth_token')
  }
</script>

<div class="app">
  {#if isAuthenticated}
    <Dashboard token={authToken} on:logout={logout} />
  {:else}
    <Login on:login={handleLogin} />
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
      'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
      sans-serif;
    background: #0f172a;
    color: #e2e8f0;
  }

  .app {
    width: 100%;
    min-height: 100vh;
  }
</style>
