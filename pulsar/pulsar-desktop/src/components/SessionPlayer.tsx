/**
 * Session Player Component
 *
 * Plays back recorded terminal sessions with speed control and seeking.
 */

import { useState, useCallback, useEffect, useRef } from 'react'
import { Terminal as XTerm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import type { Recording } from '../lib/sessionRecorder'
import { getRecordingStats } from '../lib/sessionRecorder'
import '@xterm/xterm/css/xterm.css'

interface SessionPlayerProps {
  recording: Recording
  onClose?: () => void
}

type PlaybackSpeed = 0.5 | 1 | 1.5 | 2 | 4

export default function SessionPlayer({ recording, onClose }: SessionPlayerProps) {
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<XTerm | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)

  const [isPlaying, setIsPlaying] = useState(false)
  const [currentTime, setCurrentTime] = useState(0)
  const [playbackSpeed, setPlaybackSpeed] = useState<PlaybackSpeed>(1)
  const [progress, setProgress] = useState(0) // 0-100

  const playbackTimerRef = useRef<NodeJS.Timeout | null>(null)
  const currentFrameRef = useRef(0)
  const lastUpdateTimeRef = useRef(0)

  const stats = getRecordingStats(recording)

  // Initialize terminal
  useEffect(() => {
    if (!terminalRef.current) return

    // Create terminal instance
    const term = new XTerm({
      cols: recording.metadata.width,
      rows: recording.metadata.height,
      cursorBlink: false,
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

    // Add fit addon
    const fitAddon = new FitAddon()
    term.loadAddon(fitAddon)

    // Open terminal
    term.open(terminalRef.current)
    fitAddon.fit()

    // Store references
    xtermRef.current = term
    fitAddonRef.current = fitAddon

    // Show welcome message
    term.writeln('\x1b[1;36m╔══════════════════════════════════════╗\x1b[0m')
    term.writeln('\x1b[1;36m║      Session Recording Player        ║\x1b[0m')
    term.writeln('\x1b[1;36m╚══════════════════════════════════════╝\x1b[0m')
    term.writeln('')
    term.writeln(`\x1b[90mRecording: ${recording.name}\x1b[0m`)
    term.writeln(`\x1b[90mDuration: ${formatDuration(recording.duration)}\x1b[0m`)
    term.writeln(`\x1b[90mFrames: ${recording.frames.length}\x1b[0m`)
    term.writeln('')
    term.writeln('\x1b[1;32mPress Play to start\x1b[0m')
    term.writeln('')

    // Handle resize
    const handleResize = () => {
      fitAddon.fit()
    }

    window.addEventListener('resize', handleResize)

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize)
      term.dispose()
    }
  }, [recording])

  // Playback loop
  useEffect(() => {
    if (!isPlaying || !xtermRef.current) return

    const playFrame = () => {
      const now = Date.now()
      const elapsed = now - lastUpdateTimeRef.current

      // Update current time based on playback speed
      const newTime = currentTime + elapsed * playbackSpeed

      if (newTime >= recording.duration) {
        // Playback finished
        setIsPlaying(false)
        setCurrentTime(recording.duration)
        setProgress(100)
        currentFrameRef.current = recording.frames.length
        return
      }

      setCurrentTime(newTime)
      setProgress((newTime / recording.duration) * 100)

      // Render frames up to current time
      while (currentFrameRef.current < recording.frames.length) {
        const frame = recording.frames[currentFrameRef.current]

        if (frame.timestamp > newTime) {
          break
        }

        // Render frame
        if (frame.type === 'output' && xtermRef.current) {
          xtermRef.current.write(frame.data)
        }

        currentFrameRef.current++
      }

      lastUpdateTimeRef.current = now
    }

    // Start playback loop
    lastUpdateTimeRef.current = Date.now()
    playbackTimerRef.current = setInterval(playFrame, 16) // ~60fps

    return () => {
      if (playbackTimerRef.current) {
        clearInterval(playbackTimerRef.current)
      }
    }
  }, [isPlaying, currentTime, playbackSpeed, recording])

  // Play/Pause
  const handlePlayPause = useCallback(() => {
    if (currentTime >= recording.duration) {
      // Restart from beginning
      handleRestart()
      setIsPlaying(true)
    } else {
      setIsPlaying(!isPlaying)
      if (!isPlaying) {
        lastUpdateTimeRef.current = Date.now()
      }
    }
  }, [isPlaying, currentTime, recording.duration])

  // Restart
  const handleRestart = useCallback(() => {
    setIsPlaying(false)
    setCurrentTime(0)
    setProgress(0)
    currentFrameRef.current = 0

    // Clear terminal
    if (xtermRef.current) {
      xtermRef.current.clear()
    }
  }, [])

  // Seek to time
  const handleSeek = useCallback((newTime: number) => {
    setIsPlaying(false)
    setCurrentTime(newTime)
    setProgress((newTime / recording.duration) * 100)

    // Find frame index for this time
    let frameIndex = 0
    while (frameIndex < recording.frames.length && recording.frames[frameIndex].timestamp < newTime) {
      frameIndex++
    }

    currentFrameRef.current = frameIndex

    // Re-render terminal from beginning to this point
    if (xtermRef.current) {
      xtermRef.current.clear()

      for (let i = 0; i < frameIndex; i++) {
        const frame = recording.frames[i]
        if (frame.type === 'output') {
          xtermRef.current.write(frame.data)
        }
      }
    }
  }, [recording])

  // Handle progress bar click
  const handleProgressClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const rect = e.currentTarget.getBoundingClientRect()
      const x = e.clientX - rect.left
      const percent = (x / rect.width) * 100
      const newTime = (percent / 100) * recording.duration

      handleSeek(newTime)
    },
    [recording.duration, handleSeek]
  )

  // Change playback speed
  const handleSpeedChange = useCallback((speed: PlaybackSpeed) => {
    setPlaybackSpeed(speed)
  }, [])

  // Format duration
  const formatDuration = (ms: number) => {
    const seconds = Math.floor(ms / 1000)
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = seconds % 60

    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
  }

  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Header */}
      <div className="flex-shrink-0 bg-gray-800 border-b border-gray-700 px-4 py-3">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-white">{recording.name}</h2>
            <div className="flex items-center gap-4 mt-1 text-sm text-gray-400">
              <span>Duration: {formatDuration(recording.duration)}</span>
              <span>Frames: {recording.frames.length}</span>
              <span>Size: {(stats.sizeKB).toFixed(1)} KB</span>
            </div>
          </div>
          {onClose && (
            <button
              onClick={onClose}
              className="p-2 hover:bg-gray-700 rounded transition-colors text-gray-400 hover:text-white"
              title="Close"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          )}
        </div>
      </div>

      {/* Terminal */}
      <div className="flex-1 overflow-hidden">
        <div ref={terminalRef} className="h-full w-full p-2" />
      </div>

      {/* Controls */}
      <div className="flex-shrink-0 bg-gray-800 border-t border-gray-700 px-4 py-3">
        {/* Progress Bar */}
        <div
          className="h-2 bg-gray-700 rounded-full cursor-pointer mb-3 relative"
          onClick={handleProgressClick}
        >
          <div
            className="h-full bg-blue-600 rounded-full transition-all"
            style={{ width: `${progress}%` }}
          />
          {/* Playhead */}
          <div
            className="absolute top-1/2 -translate-y-1/2 w-4 h-4 bg-white rounded-full shadow-lg"
            style={{ left: `${progress}%`, transform: 'translate(-50%, -50%)' }}
          />
        </div>

        {/* Time Display */}
        <div className="flex items-center justify-between text-sm text-gray-400 mb-3">
          <span>{formatDuration(currentTime)}</span>
          <span>{formatDuration(recording.duration)}</span>
        </div>

        {/* Buttons */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {/* Play/Pause */}
            <button
              onClick={handlePlayPause}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors flex items-center gap-2"
            >
              {isPlaying ? (
                <>
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                  Pause
                </>
              ) : (
                <>
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                    />
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                  {currentTime >= recording.duration ? 'Replay' : 'Play'}
                </>
              )}
            </button>

            {/* Restart */}
            <button
              onClick={handleRestart}
              className="px-4 py-2 bg-gray-700 text-gray-200 rounded hover:bg-gray-600 transition-colors"
              title="Restart"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
            </button>
          </div>

          {/* Speed Control */}
          <div className="flex items-center gap-2">
            <span className="text-sm text-gray-400 mr-2">Speed:</span>
            {[0.5, 1, 1.5, 2, 4].map((speed) => (
              <button
                key={speed}
                onClick={() => handleSpeedChange(speed as PlaybackSpeed)}
                className={`px-3 py-1 text-sm rounded transition-colors ${
                  playbackSpeed === speed
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
              >
                {speed}x
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
