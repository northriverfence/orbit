/**
 * PulsarTerminal - React component for interactive terminal sessions
 *
 * Features:
 * - Real-time PTY I/O via Pulsar daemon
 * - xterm.js terminal emulation
 * - Auto-fit terminal sizing
 * - Session lifecycle management
 * - Base64 encoding/decoding
 */

import React, { useEffect, useRef, useState } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { invoke } from '@tauri-apps/api/core';
import '@xterm/xterm/css/xterm.css';

interface PulsarTerminalProps {
  sessionId?: string;
  onSessionCreated?: (sessionId: string) => void;
  onSessionClosed?: () => void;
  cols?: number;
  rows?: number;
  websocketUrl?: string; // WebSocket URL (default: ws://127.0.0.1:3030)
}

export const PulsarTerminal: React.FC<PulsarTerminalProps> = ({
  sessionId: providedSessionId,
  onSessionCreated,
  onSessionClosed,
  cols = 80,
  rows = 24,
  websocketUrl = 'ws://127.0.0.1:3030',
}) => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const websocketRef = useRef<WebSocket | null>(null);

  const [sessionId, setSessionId] = useState<string | null>(providedSessionId || null);
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Initialize terminal
  useEffect(() => {
    if (!terminalRef.current) return;

    // Create terminal instance
    const terminal = new Terminal({
      cols,
      rows,
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#d4d4d4',
        cursorAccent: '#1e1e1e',
      },
    });

    // Add fit addon for auto-sizing
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);

    // Add web links addon
    terminal.loadAddon(new WebLinksAddon());

    // Open terminal in DOM
    terminal.open(terminalRef.current);
    fitAddon.fit();

    // Store refs
    xtermRef.current = terminal;
    fitAddonRef.current = fitAddon;

    // Handle terminal resize
    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      if (sessionId) {
        handleResize(terminal.cols, terminal.rows);
      }
    });
    resizeObserver.observe(terminalRef.current);

    // Handle user input
    terminal.onData(async (data) => {
      if (!sessionId) return;

      try {
        // Encode input as base64
        const base64Data = btoa(data);

        // Send to PTY
        await invoke<number>('daemon_send_input', {
          sessionId,
          data: base64Data,
        });
      } catch (err) {
        console.error('Failed to send input:', err);
        terminal.write(`\r\n\x1b[31mError sending input: ${err}\x1b[0m\r\n`);
      }
    });

    setIsReady(true);

    return () => {
      resizeObserver.disconnect();
      terminal.dispose();
      xtermRef.current = null;
      fitAddonRef.current = null;
    };
  }, [terminalRef.current]); // Only run once when ref is set

  // Create or connect to session
  useEffect(() => {
    if (!isReady) return;

    const initSession = async () => {
      try {
        let sid: string;

        if (providedSessionId) {
          // Use existing session
          sid = providedSessionId;
        } else {
          // Create new session
          sid = await invoke<string>('daemon_create_local_session', {
            name: `Terminal ${Date.now()}`,
            cols,
            rows,
          });

          if (onSessionCreated) {
            onSessionCreated(sid);
          }
        }

        setSessionId(sid);
        console.log(`Terminal connected to session: ${sid}`);
      } catch (err) {
        const errorMsg = `Failed to initialize session: ${err}`;
        setError(errorMsg);
        console.error(errorMsg);
        xtermRef.current?.write(`\x1b[31m${errorMsg}\x1b[0m\r\n`);
      }
    };

    initSession();
  }, [isReady, providedSessionId]);

  // Connect to WebSocket for real-time output streaming
  useEffect(() => {
    if (!sessionId || !xtermRef.current) return;

    const terminal = xtermRef.current;
    const wsUrl = `${websocketUrl}/ws/${sessionId}`;

    console.log(`Connecting to WebSocket: ${wsUrl}`);

    // Create WebSocket connection
    const ws = new WebSocket(wsUrl);
    websocketRef.current = ws;

    ws.onopen = () => {
      console.log(`WebSocket connected for session: ${sessionId}`);
    };

    ws.onmessage = (event) => {
      try {
        // Decode base64 output from server
        const output = atob(event.data);

        // Write to terminal
        terminal.write(output);
      } catch (err) {
        console.error('Failed to decode WebSocket message:', err);
      }
    };

    ws.onerror = (event) => {
      console.error('WebSocket error:', event);
      terminal.write('\r\n\x1b[31mWebSocket connection error\x1b[0m\r\n');
    };

    ws.onclose = (event) => {
      console.log(`WebSocket closed: ${event.code} ${event.reason}`);
      if (event.code !== 1000) {
        // Abnormal closure
        terminal.write('\r\n\x1b[31mWebSocket connection closed\x1b[0m\r\n');
      }
    };

    return () => {
      if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
        ws.close(1000, 'Component unmounting');
      }
      websocketRef.current = null;
    };
  }, [sessionId, websocketUrl]);

  // Handle resize
  const handleResize = async (newCols: number, newRows: number) => {
    if (!sessionId) return;

    try {
      await invoke('daemon_resize_terminal', {
        sessionId,
        cols: newCols,
        rows: newRows,
      });
    } catch (err) {
      console.error('Failed to resize terminal:', err);
    }
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (sessionId && !providedSessionId) {
        // Only terminate if we created the session
        invoke('daemon_terminate_session', { sessionId }).catch((err) => {
          console.error('Failed to terminate session:', err);
        });

        if (onSessionClosed) {
          onSessionClosed();
        }
      }
    };
  }, [sessionId, providedSessionId, onSessionClosed]);

  return (
    <div className="pulsar-terminal-container" style={{ width: '100%', height: '100%' }}>
      <div
        ref={terminalRef}
        style={{
          width: '100%',
          height: '100%',
        }}
      />
      {error && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            padding: '10px',
            background: '#ff4444',
            color: 'white',
            fontFamily: 'monospace',
          }}
        >
          {error}
        </div>
      )}
    </div>
  );
};

export default PulsarTerminal;
