/**
 * Session Recorder
 *
 * Records terminal output and input with timing information.
 * Compatible with asciicast v2 format (asciinema).
 */

import type { Terminal } from '@xterm/xterm'

export interface RecordingFrame {
  timestamp: number // milliseconds from recording start
  type: 'output' | 'input'
  data: string
}

export interface RecordingMetadata {
  width: number
  height: number
  shell: string
  env: Record<string, string>
  title?: string
  sessionId?: string
  hostname?: string
}

export interface Recording {
  id: string
  sessionId: string
  name: string
  startTime: string
  duration: number // milliseconds
  frames: RecordingFrame[]
  metadata: RecordingMetadata
  sizeBytes: number
}

export interface RecordingState {
  isRecording: boolean
  isPaused: boolean
  startTime: number
  pausedTime: number
  totalPausedDuration: number
  frameCount: number
  sizeBytes: number
}

/**
 * Session Recorder
 *
 * Captures terminal I/O and timing for playback
 */
export class SessionRecorder {
  private terminal: Terminal
  private sessionId: string
  private recording: Recording | null = null
  private state: RecordingState
  private dataDisposable: any = null
  private lastFrameTime: number = 0

  constructor(terminal: Terminal, sessionId: string) {
    this.terminal = terminal
    this.sessionId = sessionId
    this.state = {
      isRecording: false,
      isPaused: false,
      startTime: 0,
      pausedTime: 0,
      totalPausedDuration: 0,
      frameCount: 0,
      sizeBytes: 0,
    }
  }

  /**
   * Start recording
   */
  startRecording(name?: string): void {
    if (this.state.isRecording) {
      throw new Error('Recording already in progress')
    }

    const now = Date.now()
    const dimensions = { cols: this.terminal.cols, rows: this.terminal.rows }

    this.recording = {
      id: `rec-${now}-${Math.random().toString(36).substring(2, 11)}`,
      sessionId: this.sessionId,
      name: name || `Recording ${new Date().toLocaleString()}`,
      startTime: new Date().toISOString(),
      duration: 0,
      frames: [],
      metadata: {
        width: dimensions.cols,
        height: dimensions.rows,
        shell: process.env.SHELL || '/bin/bash',
        env: {
          TERM: process.env.TERM || 'xterm-256color',
          SHELL: process.env.SHELL || '/bin/bash',
        },
        sessionId: this.sessionId,
      },
      sizeBytes: 0,
    }

    this.state = {
      isRecording: true,
      isPaused: false,
      startTime: now,
      pausedTime: 0,
      totalPausedDuration: 0,
      frameCount: 0,
      sizeBytes: 0,
    }

    // Capture terminal output
    this.dataDisposable = this.terminal.onData((data: string) => {
      this.recordFrame('input', data)
    })

    console.log(`Started recording: ${this.recording.id}`)
  }

  /**
   * Stop recording
   */
  stopRecording(): Recording | null {
    if (!this.state.isRecording) {
      return null
    }

    const now = Date.now()
    if (this.recording) {
      this.recording.duration = now - this.state.startTime - this.state.totalPausedDuration
    }

    this.state.isRecording = false
    this.state.isPaused = false

    // Clean up event listener
    if (this.dataDisposable) {
      this.dataDisposable.dispose()
      this.dataDisposable = null
    }

    console.log(`Stopped recording: ${this.recording?.id}`)
    console.log(`Recorded ${this.state.frameCount} frames in ${this.recording?.duration}ms`)

    const finalRecording = this.recording
    this.recording = null

    return finalRecording
  }

  /**
   * Pause recording
   */
  pauseRecording(): void {
    if (!this.state.isRecording || this.state.isPaused) {
      return
    }

    this.state.isPaused = true
    this.state.pausedTime = Date.now()
    console.log('Recording paused')
  }

  /**
   * Resume recording
   */
  resumeRecording(): void {
    if (!this.state.isRecording || !this.state.isPaused) {
      return
    }

    const pauseDuration = Date.now() - this.state.pausedTime
    this.state.totalPausedDuration += pauseDuration
    this.state.isPaused = false
    this.state.pausedTime = 0

    console.log(`Recording resumed (paused for ${pauseDuration}ms)`)
  }

  /**
   * Record a frame
   */
  private recordFrame(type: 'output' | 'input', data: string): void {
    if (!this.state.isRecording || this.state.isPaused || !this.recording) {
      return
    }

    const now = Date.now()
    const timestamp = now - this.state.startTime - this.state.totalPausedDuration

    // Avoid duplicate frames (< 1ms apart)
    if (timestamp - this.lastFrameTime < 1) {
      return
    }

    const frame: RecordingFrame = {
      timestamp,
      type,
      data,
    }

    this.recording.frames.push(frame)
    this.state.frameCount++
    this.state.sizeBytes += data.length
    this.recording.sizeBytes = this.state.sizeBytes
    this.lastFrameTime = timestamp
  }

  /**
   * Record terminal output manually
   */
  recordOutput(data: string): void {
    this.recordFrame('output', data)
  }

  /**
   * Get recording state
   */
  getState(): RecordingState {
    return { ...this.state }
  }

  /**
   * Get current recording
   */
  getCurrentRecording(): Recording | null {
    return this.recording
  }

  /**
   * Is currently recording
   */
  isRecording(): boolean {
    return this.state.isRecording
  }

  /**
   * Is recording paused
   */
  isPaused(): boolean {
    return this.state.isPaused
  }
}

/**
 * Calculate recording statistics
 */
export function getRecordingStats(recording: Recording): {
  totalFrames: number
  inputFrames: number
  outputFrames: number
  averageFrameSize: number
  framesPerSecond: number
  durationSeconds: number
  sizeKB: number
} {
  const inputFrames = recording.frames.filter((f) => f.type === 'input').length
  const outputFrames = recording.frames.filter((f) => f.type === 'output').length
  const durationSeconds = recording.duration / 1000

  return {
    totalFrames: recording.frames.length,
    inputFrames,
    outputFrames,
    averageFrameSize: recording.sizeBytes / recording.frames.length,
    framesPerSecond: recording.frames.length / durationSeconds,
    durationSeconds,
    sizeKB: recording.sizeBytes / 1024,
  }
}

/**
 * Estimate recording size
 */
export function estimateRecordingSize(durationMinutes: number, activityLevel: 'low' | 'medium' | 'high'): number {
  // Average bytes per second based on activity
  const bytesPerSecond = {
    low: 500, // Mostly idle
    medium: 2000, // Moderate typing and output
    high: 10000, // Heavy output (logs, compilation, etc.)
  }

  return durationMinutes * 60 * bytesPerSecond[activityLevel]
}

/**
 * Compress recording (remove idle periods)
 */
export function compressRecording(recording: Recording, maxIdleMs: number = 2000): Recording {
  if (recording.frames.length === 0) {
    return recording
  }

  const compressed: RecordingFrame[] = []
  let lastFrameTime = 0
  let timeOffset = 0

  for (const frame of recording.frames) {
    const idleTime = frame.timestamp - lastFrameTime

    if (idleTime > maxIdleMs && compressed.length > 0) {
      // Compress idle period to maxIdleMs
      timeOffset += idleTime - maxIdleMs
    }

    compressed.push({
      ...frame,
      timestamp: frame.timestamp - timeOffset,
    })

    lastFrameTime = frame.timestamp
  }

  return {
    ...recording,
    frames: compressed,
    duration: compressed[compressed.length - 1].timestamp,
  }
}

/**
 * Trim recording (remove beginning/end)
 */
export function trimRecording(
  recording: Recording,
  startMs: number,
  endMs: number
): Recording {
  const trimmed = recording.frames.filter(
    (frame) => frame.timestamp >= startMs && frame.timestamp <= endMs
  )

  // Adjust timestamps to start at 0
  const adjusted = trimmed.map((frame) => ({
    ...frame,
    timestamp: frame.timestamp - startMs,
  }))

  return {
    ...recording,
    frames: adjusted,
    duration: endMs - startMs,
    sizeBytes: adjusted.reduce((sum, frame) => sum + frame.data.length, 0),
  }
}
