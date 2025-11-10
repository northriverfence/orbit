/**
 * Asciicast Format Converter
 *
 * Converts recordings to/from asciicast v2 format (asciinema compatible).
 * Spec: https://github.com/asciinema/asciinema/blob/develop/doc/asciicast-v2.md
 */

import type { Recording, RecordingFrame } from './sessionRecorder'

/**
 * Asciicast v2 Header
 */
export interface AsciicastHeader {
  version: 2
  width: number
  height: number
  timestamp?: number
  duration?: number
  idle_time_limit?: number
  command?: string
  title?: string
  env?: Record<string, string>
}

/**
 * Asciicast v2 Event
 * [time, type, data]
 */
export type AsciicastEvent = [number, 'o' | 'i', string]

/**
 * Export recording to asciicast v2 format
 */
export function exportToAsciicast(recording: Recording): string {
  // Header (first line)
  const header: AsciicastHeader = {
    version: 2,
    width: recording.metadata.width,
    height: recording.metadata.height,
    timestamp: Math.floor(new Date(recording.startTime).getTime() / 1000),
    duration: recording.duration / 1000, // Convert to seconds
    title: recording.name,
    env: recording.metadata.env,
  }

  // Events (following lines)
  const events: AsciicastEvent[] = recording.frames.map((frame) => {
    const time = frame.timestamp / 1000 // Convert to seconds
    const type = frame.type === 'output' ? 'o' : 'i'
    return [parseFloat(time.toFixed(6)), type, frame.data]
  })

  // Format as NDJSON (newline-delimited JSON)
  const lines = [JSON.stringify(header), ...events.map((event) => JSON.stringify(event))]

  return lines.join('\n')
}

/**
 * Import recording from asciicast v2 format
 */
export function importFromAsciicast(content: string, sessionId: string): Recording {
  const lines = content.trim().split('\n')

  if (lines.length < 1) {
    throw new Error('Invalid asciicast: empty file')
  }

  // Parse header
  const header: AsciicastHeader = JSON.parse(lines[0])

  if (header.version !== 2) {
    throw new Error(`Unsupported asciicast version: ${header.version}`)
  }

  // Parse events
  const frames: RecordingFrame[] = []

  for (let i = 1; i < lines.length; i++) {
    try {
      const event: AsciicastEvent = JSON.parse(lines[i])
      const [time, type, data] = event

      frames.push({
        timestamp: Math.round(time * 1000), // Convert to milliseconds
        type: type === 'o' ? 'output' : 'input',
        data,
      })
    } catch (error) {
      console.warn(`Failed to parse event on line ${i + 1}:`, error)
    }
  }

  // Calculate duration from last frame
  const duration = frames.length > 0 ? frames[frames.length - 1].timestamp : 0

  // Calculate size
  const sizeBytes = frames.reduce((sum, frame) => sum + frame.data.length, 0)

  // Create Recording object
  const recording: Recording = {
    id: `imported-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`,
    sessionId,
    name: header.title || 'Imported Recording',
    startTime: header.timestamp
      ? new Date(header.timestamp * 1000).toISOString()
      : new Date().toISOString(),
    duration,
    frames,
    metadata: {
      width: header.width,
      height: header.height,
      shell: header.env?.SHELL || '/bin/bash',
      env: header.env || {},
      title: header.title,
      sessionId,
    },
    sizeBytes,
  }

  return recording
}

/**
 * Validate asciicast format
 */
export function validateAsciicast(content: string): {
  valid: boolean
  errors: string[]
  warnings: string[]
} {
  const errors: string[] = []
  const warnings: string[] = []

  try {
    const lines = content.trim().split('\n')

    if (lines.length < 1) {
      errors.push('Empty file')
      return { valid: false, errors, warnings }
    }

    // Validate header
    try {
      const header: AsciicastHeader = JSON.parse(lines[0])

      if (header.version !== 2) {
        errors.push(`Unsupported version: ${header.version}. Only version 2 is supported.`)
      }

      if (!header.width || header.width <= 0) {
        errors.push('Invalid width in header')
      }

      if (!header.height || header.height <= 0) {
        errors.push('Invalid height in header')
      }
    } catch (error) {
      errors.push(`Invalid header JSON: ${error}`)
    }

    // Validate events
    let lastTime = 0
    for (let i = 1; i < lines.length; i++) {
      try {
        const event: AsciicastEvent = JSON.parse(lines[i])

        if (!Array.isArray(event) || event.length !== 3) {
          errors.push(`Line ${i + 1}: Event must be an array of 3 elements`)
          continue
        }

        const [time, type, data] = event

        if (typeof time !== 'number' || time < 0) {
          errors.push(`Line ${i + 1}: Invalid timestamp`)
        }

        if (type !== 'o' && type !== 'i') {
          errors.push(`Line ${i + 1}: Type must be 'o' or 'i'`)
        }

        if (typeof data !== 'string') {
          errors.push(`Line ${i + 1}: Data must be a string`)
        }

        if (time < lastTime) {
          warnings.push(`Line ${i + 1}: Timestamp goes backwards (${lastTime} -> ${time})`)
        }

        lastTime = time
      } catch (error) {
        errors.push(`Line ${i + 1}: Invalid JSON: ${error}`)
      }
    }

    if (lines.length === 1) {
      warnings.push('No events found (header only)')
    }
  } catch (error) {
    errors.push(`Validation error: ${error}`)
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  }
}

/**
 * Get asciicast statistics
 */
export function getAsciicastStats(content: string): {
  version: number
  dimensions: { width: number; height: number }
  eventCount: number
  durationSeconds: number
  sizeBytes: number
  inputEvents: number
  outputEvents: number
} | null {
  try {
    const lines = content.trim().split('\n')
    const header: AsciicastHeader = JSON.parse(lines[0])

    let inputEvents = 0
    let outputEvents = 0
    let lastTime = 0

    for (let i = 1; i < lines.length; i++) {
      const event: AsciicastEvent = JSON.parse(lines[i])
      const [time, type] = event

      if (type === 'i') inputEvents++
      if (type === 'o') outputEvents++
      if (time > lastTime) lastTime = time
    }

    return {
      version: header.version,
      dimensions: { width: header.width, height: header.height },
      eventCount: lines.length - 1,
      durationSeconds: lastTime,
      sizeBytes: content.length,
      inputEvents,
      outputEvents,
    }
  } catch (error) {
    console.error('Failed to get asciicast stats:', error)
    return null
  }
}

/**
 * Convert asciicast to plain text
 */
export function asciicastToText(content: string): string {
  try {
    const lines = content.trim().split('\n')
    const output: string[] = []

    for (let i = 1; i < lines.length; i++) {
      const event: AsciicastEvent = JSON.parse(lines[i])
      const [_time, type, data] = event

      // Only include output events
      if (type === 'o') {
        output.push(data)
      }
    }

    return output.join('')
  } catch (error) {
    console.error('Failed to convert asciicast to text:', error)
    return ''
  }
}

/**
 * Compress asciicast (remove duplicate consecutive output)
 */
export function compressAsciicast(content: string, mergeThresholdSeconds: number = 0.1): string {
  try {
    const lines = content.trim().split('\n')
    const header = lines[0]
    const compressed: string[] = [header]

    let lastOutputTime = 0
    let accumulatedOutput = ''

    for (let i = 1; i < lines.length; i++) {
      const event: AsciicastEvent = JSON.parse(lines[i])
      const [time, type, data] = event

      if (type === 'o') {
        // Accumulate output if within threshold
        if (time - lastOutputTime <= mergeThresholdSeconds && accumulatedOutput) {
          accumulatedOutput += data
          continue
        }

        // Flush accumulated output
        if (accumulatedOutput) {
          compressed.push(JSON.stringify([lastOutputTime, 'o', accumulatedOutput]))
        }

        // Start new accumulation
        accumulatedOutput = data
        lastOutputTime = time
      } else {
        // Flush any accumulated output before input event
        if (accumulatedOutput) {
          compressed.push(JSON.stringify([lastOutputTime, 'o', accumulatedOutput]))
          accumulatedOutput = ''
        }

        // Add input event
        compressed.push(lines[i])
      }
    }

    // Flush remaining output
    if (accumulatedOutput) {
      compressed.push(JSON.stringify([lastOutputTime, 'o', accumulatedOutput]))
    }

    return compressed.join('\n')
  } catch (error) {
    console.error('Failed to compress asciicast:', error)
    return content
  }
}
