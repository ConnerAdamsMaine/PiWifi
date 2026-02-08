<script lang="ts">
  import { onMount } from 'svelte'

  export let token: string

  let terminal: HTMLDivElement
  let input: HTMLInputElement
  let output: string[] = []
  let command = ''
  let connected = false
  let sessionId = ''

  const allowedCommands = [
    'help',
    'status',
    'wifi-scan',
    'wifi-status',
    'network-status',
    'firewall-rules',
    'ifconfig',
    'ping 8.8.8.8',
    'nslookup google.com',
    'clear'
  ]

  onMount(() => {
    sessionId = Math.random().toString(36).substring(7)
    addOutput('🔌 PiWifi Terminal')
    addOutput('Type "help" for available commands\n')
    input?.focus()
  })

  function addOutput(text: string) {
    output = [...output, text]
    setTimeout(() => {
      if (terminal) {
        terminal.scrollTop = terminal.scrollHeight
      }
    }, 0)
  }

  async function executeCommand(cmd: string) {
    if (!cmd.trim()) return

    addOutput(`$ ${cmd}`)
    command = ''

    // Simulate command execution
    if (cmd === 'help') {
      addOutput('Available commands:')
      allowedCommands.forEach(c => addOutput(`  ${c}`))
    } else if (cmd === 'status') {
      addOutput('PiWifi Status:')
      addOutput('  WiFi: Connected')
      addOutput('  Network: 192.168.100.0/24')
      addOutput('  DHCP: Active')
      addOutput('  Firewall: Enabled')
    } else if (cmd === 'wifi-scan') {
      addOutput('Scanning for WiFi networks...')
      addOutput('  MyNetwork (Signal: -45 dBm)')
      addOutput('  NeighborWiFi (Signal: -68 dBm)')
    } else if (cmd === 'wifi-status') {
      addOutput('WiFi Status:')
      addOutput('  Interface: wlan0')
      addOutput('  SSID: MyNetwork')
      addOutput('  IP: 192.168.1.100')
      addOutput('  Signal: -45 dBm')
    } else if (cmd === 'network-status') {
      addOutput('Network Status:')
      addOutput('  eth0: 192.168.100.1/24')
      addOutput('  DHCP: Enabled')
      addOutput('  DNS: piwifi.local')
    } else if (cmd === 'firewall-rules') {
      addOutput('Firewall Rules (INPUT):')
      addOutput('  Chain INPUT (policy DROP)')
      addOutput('  ACCEPT  all -- lo')
      addOutput('  ACCEPT  all -- * *  state RELATED,ESTABLISHED')
      addOutput('  ACCEPT  udp -- eth0 * dpt:53')
      addOutput('  ACCEPT  tcp -- eth0 * dpt:22')
    } else if (cmd === 'ifconfig') {
      addOutput('eth0: flags=UP,RUNNING')
      addOutput('  inet 192.168.100.1  netmask 255.255.255.0')
      addOutput('  RX packets: 1234  RX bytes: 567890')
      addOutput('  TX packets: 5678  TX bytes: 123456')
      addOutput('')
      addOutput('wlan0: flags=UP,RUNNING')
      addOutput('  inet 192.168.1.100  netmask 255.255.255.0')
      addOutput('  RX packets: 4567  RX bytes: 234567')
      addOutput('  TX packets: 8901  TX bytes: 345678')
    } else if (cmd === 'ping 8.8.8.8') {
      addOutput('PING 8.8.8.8 (8.8.8.8) 56(84) bytes of data.')
      addOutput('64 bytes from 8.8.8.8: icmp_seq=1 ttl=119 time=25.3 ms')
      addOutput('64 bytes from 8.8.8.8: icmp_seq=2 ttl=119 time=24.8 ms')
      addOutput('--- 8.8.8.8 statistics ---')
      addOutput('2 packets transmitted, 2 received, 0% packet loss')
    } else if (cmd === 'nslookup google.com') {
      addOutput('Server: 192.168.100.1')
      addOutput('Address: 192.168.100.1#53')
      addOutput('Non-authoritative answer:')
      addOutput('Name: google.com')
      addOutput('Address: 142.250.185.46')
    } else if (cmd === 'clear') {
      output = []
    } else {
      addOutput(`Unknown command: ${cmd}`)
      addOutput('Type "help" for available commands')
    }

    addOutput('')
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      executeCommand(command)
    }
  }
</script>

<div class="terminal-panel">
  <div class="panel-header">
    <h2>⌨️ Terminal Emulator</h2>
    <span class="status" class:connected>
      {connected ? '🟢 Connected' : '🔴 Read-only'}
    </span>
  </div>

  <div class="terminal" bind:this={terminal}>
    {#each output as line (line + Math.random())}
      <div class="line">{line}</div>
    {/each}
  </div>

  <div class="input-area">
    <span class="prompt">$</span>
    <input
      type="text"
      bind:value={command}
      bind:this={input}
      on:keydown={handleKeydown}
      placeholder="Type command and press Enter (try 'help')"
    />
  </div>

  <div class="info">
    <p>
      💡 This is a read-only terminal simulator for demonstration. Full PTY support with WebSocket
      streaming coming soon.
    </p>
  </div>
</div>

<style>
  .terminal-panel {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 300px);
    gap: 1rem;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .panel-header h2 {
    margin: 0;
  }

  .status {
    font-size: 0.9rem;
    color: #ef4444;
  }

  .status.connected {
    color: #22c55e;
  }

  .terminal {
    flex: 1;
    background: #0f172a;
    border: 1px solid #334155;
    border-radius: 6px;
    padding: 1rem;
    overflow-y: auto;
    font-family: 'Courier New', monospace;
    font-size: 0.9rem;
    line-height: 1.5;
    color: #60a5fa;
  }

  .line {
    white-space: pre-wrap;
    word-break: break-word;
  }

  .line:empty {
    min-height: 1.5em;
  }

  .input-area {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 6px;
    padding: 0.75rem;
  }

  .prompt {
    color: #60a5fa;
    font-weight: 600;
    font-family: 'Courier New', monospace;
  }

  input {
    flex: 1;
    background: transparent;
    border: none;
    color: #e2e8f0;
    font-family: 'Courier New', monospace;
    font-size: 0.9rem;
  }

  input:focus {
    outline: none;
  }

  .info {
    padding: 1rem;
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 6px;
    font-size: 0.85rem;
    color: #94a3b8;
    margin: 0;
  }

  .info p {
    margin: 0;
  }
</style>
