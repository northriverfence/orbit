import { useEffect, useRef, useState } from 'react'
import { Terminal as XTerm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { SearchAddon } from '@xterm/addon-search'
import { invoke } from '@tauri-apps/api/core'
import LoadingOverlay from './LoadingOverlay'
import '@xterm/xterm/css/xterm.css'

interface TerminalProps {
  sessionId?: string
  host?: string
  port?: number
  username?: string
  password?: string
  authType?: 'password' | 'publickey' | 'agent'
  keyPath?: string
  keyPassphrase?: string
  credentialId?: string | null
}

export default function Terminal({
  sessionId,
  host = 'localhost',
  port = 22,
  username = 'user',
  password = 'password',
  authType = 'password',
  keyPath,
  keyPassphrase,
  credentialId,
}: TerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<XTerm | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)
  const [sshSessionId, setSshSessionId] = useState<string | null>(null)
  const [isConnecting, setIsConnecting] = useState(false)

  useEffect(() => {
    if (!terminalRef.current) return

    // Create terminal instance
    const term = new XTerm({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#ffffff',
        selectionBackground: 'rgba(255, 255, 255, 0.3)',
      },
      scrollback: 10000,
      allowProposedApi: true,
    })

    // Add addons
    const fitAddon = new FitAddon()
    const webLinksAddon = new WebLinksAddon()
    const searchAddon = new SearchAddon()

    term.loadAddon(fitAddon)
    term.loadAddon(webLinksAddon)
    term.loadAddon(searchAddon)

    // Open terminal
    term.open(terminalRef.current)
    fitAddon.fit()

    // Store references
    xtermRef.current = term
    fitAddonRef.current = fitAddon

    // Welcome message
    term.writeln('\x1b[1;32m╔══════════════════════════════════════╗\x1b[0m')
    term.writeln('\x1b[1;32m║      Welcome to Pulsar Terminal      ║\x1b[0m')
    term.writeln('\x1b[1;32m╚══════════════════════════════════════╝\x1b[0m')
    term.writeln('')

    // Connect to SSH backend if sessionId provided
    let outputPollInterval: NodeJS.Timeout | null = null

    if (sessionId) {
      term.writeln(`\x1b[90mConnecting to ${username}@${host}:${port}...\x1b[0m`)
      setIsConnecting(true)

      const dimensions = fitAddon.proposeDimensions() || { cols: 80, rows: 24 }

      // Build auth_method based on authType
      let authMethod
      if (authType === 'publickey') {
        authMethod = {
          type: 'public_key',
          key_path: keyPath || '',
          passphrase: keyPassphrase || null,
        }
      } else if (authType === 'agent') {
        authMethod = {
          type: 'agent',
        }
      } else {
        authMethod = {
          type: 'password',
          password: password || '',
        }
      }

      invoke<string>('connect_ssh', {
        config: {
          host,
          port,
          username,
          auth_method: authMethod,
          cols: dimensions.cols,
          rows: dimensions.rows,
          credential_id: credentialId || null,
        },
      })
        .then(async (sessionId) => {
          setSshSessionId(sessionId)
          setIsConnecting(false)
          term.writeln(`\x1b[1;32m✓ Connected\x1b[0m (Session: ${sessionId.substring(0, 8)}...)`)

          // Fetch and display host key fingerprint
          try {
            const fingerprint = await invoke<string>('get_fingerprint', { session_id: sessionId })
            term.writeln(`\x1b[1;36m🔑 Host Key: ${fingerprint}\x1b[0m`)
          } catch (err) {
            console.error('Failed to get fingerprint:', err)
          }

          term.writeln('')
          term.writeln('\x1b[90mType to interact with the session\x1b[0m')
          term.writeln('')

          // Start polling for output
          outputPollInterval = setInterval(async () => {
            try {
              const output = await invoke<number[] | null>('receive_output', {
                session_id: sessionId,
              })
              if (output && output.length > 0) {
                term.write(new Uint8Array(output))
              }
            } catch (err) {
              console.error('Failed to receive output:', err)
            }
          }, 50) // Poll every 50ms
        })
        .catch((err) => {
          setIsConnecting(false)
          term.writeln(`\x1b[1;31m✗ Connection failed: ${err}\x1b[0m`)
          term.writeln('')
        })
    } else {
      term.writeln('\x1b[90mNo active connection. Select a server from the sidebar.\x1b[0m')
      term.writeln('')
    }

    // Handle terminal input
    term.onData((data) => {
      if (sshSessionId) {
        // Send input to SSH backend
        invoke('send_input', {
          session_id: sshSessionId,
          data,
        }).catch((err) => {
          console.error('Failed to send input:', err)
          term.writeln(`\r\n\x1b[1;31mError sending input: ${err}\x1b[0m\r\n`)
        })
      }
    })

    // Handle resize
    const handleResize = () => {
      fitAddon.fit()
      const dimensions = fitAddon.proposeDimensions()
      if (dimensions && sshSessionId) {
        invoke('resize_terminal', {
          session_id: sshSessionId,
          cols: dimensions.cols,
          rows: dimensions.rows,
        }).catch((err) => {
          console.error('Failed to resize terminal:', err)
        })
      }
    }

    window.addEventListener('resize', handleResize)

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize)

      // Clear output polling
      if (outputPollInterval) {
        clearInterval(outputPollInterval)
      }

      // Disconnect SSH session
      if (sshSessionId) {
        invoke('disconnect_ssh', { session_id: sshSessionId }).catch((err) => {
          console.error('Failed to disconnect SSH:', err)
        })
      }

      term.dispose()
    }
  }, [sessionId, sshSessionId, host, port, username, password])

  return (
    <div className="h-full w-full bg-[#1e1e1e] rounded-lg overflow-hidden relative">
      <div ref={terminalRef} className="h-full w-full p-2" />
      {isConnecting && (
        <LoadingOverlay
          message={`Connecting to ${username}@${host}...`}
          transparent={true}
        />
      )}
    </div>
  )
}
