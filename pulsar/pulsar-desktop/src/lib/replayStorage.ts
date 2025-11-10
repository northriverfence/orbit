/**
 * Recording Storage
 *
 * Manages persistence of session recordings to disk.
 * Storage location: ~/.config/pulsar/recordings/
 */

import { exists, readTextFile, writeTextFile, BaseDirectory, mkdir, readDir } from '@tauri-apps/plugin-fs'
import type { Recording } from './sessionRecorder'

const CONFIG_DIR = 'pulsar/recordings'

/**
 * Initialize recordings directory
 */
async function ensureRecordingsDir(): Promise<void> {
  try {
    const dirExists = await exists(CONFIG_DIR, { baseDir: BaseDirectory.AppConfig })
    if (!dirExists) {
      await mkdir(CONFIG_DIR, { baseDir: BaseDirectory.AppConfig, recursive: true })
    }
  } catch (error) {
    console.error('Failed to create recordings directory:', error)
    throw error
  }
}

/**
 * Save recording to disk
 */
export async function saveRecording(recording: Recording): Promise<void> {
  try {
    await ensureRecordingsDir()

    const filename = `${recording.id}.json`
    const filePath = `${CONFIG_DIR}/${filename}`

    await writeTextFile(filePath, JSON.stringify(recording, null, 2), {
      baseDir: BaseDirectory.AppConfig,
    })

    console.log(`Saved recording: ${recording.id} (${recording.frames.length} frames)`)
  } catch (error) {
    console.error('Failed to save recording:', error)
    throw error
  }
}

/**
 * Load recording from disk
 */
export async function loadRecording(recordingId: string): Promise<Recording | null> {
  try {
    await ensureRecordingsDir()

    const filename = `${recordingId}.json`
    const filePath = `${CONFIG_DIR}/${filename}`

    const fileExists = await exists(filePath, { baseDir: BaseDirectory.AppConfig })
    if (!fileExists) {
      console.log(`Recording not found: ${recordingId}`)
      return null
    }

    const content = await readTextFile(filePath, { baseDir: BaseDirectory.AppConfig })
    const recording: Recording = JSON.parse(content)

    console.log(`Loaded recording: ${recordingId}`)
    return recording
  } catch (error) {
    console.error('Failed to load recording:', error)
    return null
  }
}

/**
 * List all recordings
 */
export async function listRecordings(): Promise<Recording[]> {
  try {
    await ensureRecordingsDir()

    const entries = await readDir(CONFIG_DIR, { baseDir: BaseDirectory.AppConfig })
    const recordings: Recording[] = []

    for (const entry of entries) {
      if (entry.name.endsWith('.json')) {
        const recordingId = entry.name.replace('.json', '')
        const recording = await loadRecording(recordingId)
        if (recording) {
          recordings.push(recording)
        }
      }
    }

    // Sort by start time (newest first)
    recordings.sort((a, b) => new Date(b.startTime).getTime() - new Date(a.startTime).getTime())

    console.log(`Loaded ${recordings.length} recordings`)
    return recordings
  } catch (error) {
    console.error('Failed to list recordings:', error)
    return []
  }
}

/**
 * Delete recording from disk
 */
export async function deleteRecording(recordingId: string): Promise<void> {
  try {
    const filename = `${recordingId}.json`
    const filePath = `${CONFIG_DIR}/${filename}`

    // Note: Tauri plugin-fs doesn't have remove function yet
    // We'll need to use invoke to call a Rust function
    console.log(`Would delete: ${filePath}`)
    // await invoke('delete_file', { path: filePath })
  } catch (error) {
    console.error('Failed to delete recording:', error)
    throw error
  }
}

/**
 * Get recording metadata without loading full recording
 */
export async function getRecordingMetadata(recordingId: string): Promise<Omit<Recording, 'frames'> | null> {
  try {
    const recording = await loadRecording(recordingId)
    if (!recording) return null

    // Return everything except frames
    const { frames, ...metadata } = recording
    return metadata
  } catch (error) {
    console.error('Failed to get recording metadata:', error)
    return null
  }
}

/**
 * Search recordings
 */
export function searchRecordings(
  recordings: Recording[],
  query: string
): Recording[] {
  if (!query) return recordings

  const lowerQuery = query.toLowerCase()
  return recordings.filter((recording) => {
    return (
      recording.name.toLowerCase().includes(lowerQuery) ||
      recording.sessionId.toLowerCase().includes(lowerQuery) ||
      (recording.metadata.hostname && recording.metadata.hostname.toLowerCase().includes(lowerQuery))
    )
  })
}

/**
 * Filter recordings by date
 */
export function filterRecordingsByDate(
  recordings: Recording[],
  startDate?: string,
  endDate?: string
): Recording[] {
  let filtered = recordings

  if (startDate) {
    const start = new Date(startDate).getTime()
    filtered = filtered.filter((r) => new Date(r.startTime).getTime() >= start)
  }

  if (endDate) {
    const end = new Date(endDate).getTime()
    filtered = filtered.filter((r) => new Date(r.startTime).getTime() <= end)
  }

  return filtered
}

/**
 * Get total recordings size
 */
export async function getTotalRecordingsSize(): Promise<number> {
  const recordings = await listRecordings()
  return recordings.reduce((total, recording) => total + recording.sizeBytes, 0)
}

/**
 * Clean up old recordings (keep only N most recent)
 */
export async function cleanupOldRecordings(keepCount: number = 50): Promise<number> {
  try {
    const recordings = await listRecordings()
    if (recordings.length <= keepCount) {
      return 0
    }

    const toDelete = recordings.slice(keepCount)
    let deletedCount = 0

    for (const recording of toDelete) {
      try {
        await deleteRecording(recording.id)
        deletedCount++
      } catch (error) {
        console.error(`Failed to delete recording ${recording.id}:`, error)
      }
    }

    console.log(`Cleaned up ${deletedCount} old recordings`)
    return deletedCount
  } catch (error) {
    console.error('Failed to cleanup recordings:', error)
    return 0
  }
}
