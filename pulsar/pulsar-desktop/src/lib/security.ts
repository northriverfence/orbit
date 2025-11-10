/**
 * Security Utilities
 *
 * Input validation, sanitization, and encryption for secure data handling
 */

/**
 * Escapes a shell argument by wrapping it in single quotes and escaping internal quotes
 * @param arg - The argument to escape
 * @returns Escaped shell argument safe for command execution
 * @example
 * escapeShellArg("hello world") // => 'hello world'
 * escapeShellArg("it's") // => 'it'\''s'
 */
export function escapeShellArg(arg: string): string {
  // Replace single quotes with '\'' (end quote, escaped quote, start quote)
  // This is the safest method that works across all shells
  return `'${arg.replace(/'/g, "'\\''")}'`;
}

/**
 * Validates an environment variable name
 * Must start with letter or underscore, followed by letters, numbers, or underscores
 * @param name - The variable name to validate
 * @returns True if valid, false otherwise
 * @example
 * isValidEnvVarName("PATH") // => true
 * isValidEnvVarName("MY_VAR_123") // => true
 * isValidEnvVarName("123_INVALID") // => false
 * isValidEnvVarName("HAS-DASH") // => false
 */
export function isValidEnvVarName(name: string): boolean {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(name);
}

/**
 * Validates a file path to prevent directory traversal and suspicious patterns
 * @param path - The path to validate
 * @returns Object with validation result and error message if invalid
 */
export function validatePath(path: string): { valid: boolean; error?: string } {
  // Check for null bytes (command injection attempt)
  if (path.includes('\0')) {
    return { valid: false, error: 'Path contains null bytes' };
  }

  // Check for suspicious patterns
  const suspiciousPatterns = [
    /;\s*rm\s+-rf/i,  // rm -rf commands
    /&&\s*rm\s+-rf/i, // chained rm -rf
    /\|\s*rm\s+-rf/i, // piped rm -rf
    /`.*`/,            // Command substitution
    /\$\(.*\)/,        // Command substitution
  ];

  for (const pattern of suspiciousPatterns) {
    if (pattern.test(path)) {
      return { valid: false, error: 'Path contains suspicious patterns' };
    }
  }

  // Warn about unusual but not necessarily malicious patterns
  if (path.includes('..')) {
    return { valid: false, error: 'Path contains ".." (directory traversal)' };
  }

  return { valid: true };
}

/**
 * Validates a hostname for SSH connections
 * @param host - The hostname to validate
 * @returns Object with validation result and error message if invalid
 */
export function validateHostname(host: string): { valid: boolean; error?: string } {
  // Check for null bytes
  if (host.includes('\0')) {
    return { valid: false, error: 'Hostname contains null bytes' };
  }

  // Basic hostname validation (RFC 1123)
  // Allow: letters, numbers, dots, hyphens
  // Max 253 characters, labels max 63 characters
  const hostnameRegex = /^([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$/;

  if (!hostnameRegex.test(host)) {
    // Also allow IPv4 addresses
    const ipv4Regex = /^(\d{1,3}\.){3}\d{1,3}$/;
    if (!ipv4Regex.test(host)) {
      return { valid: false, error: 'Invalid hostname format' };
    }

    // Validate IPv4 octets are 0-255
    const octets = host.split('.').map(Number);
    if (octets.some((octet) => octet > 255)) {
      return { valid: false, error: 'Invalid IPv4 address' };
    }
  }

  if (host.length > 253) {
    return { valid: false, error: 'Hostname too long (max 253 characters)' };
  }

  return { valid: true };
}

/**
 * Validates a port number
 * @param port - The port number to validate
 * @returns Object with validation result and error message if invalid
 */
export function validatePort(port: number): { valid: boolean; error?: string } {
  if (!Number.isInteger(port)) {
    return { valid: false, error: 'Port must be an integer' };
  }

  if (port < 1 || port > 65535) {
    return { valid: false, error: 'Port must be between 1 and 65535' };
  }

  return { valid: true };
}

/**
 * Sanitizes a command string by removing potentially dangerous characters
 * Note: This is a defense-in-depth measure. Commands should still be validated.
 * @param command - The command to sanitize
 * @returns Sanitized command
 */
export function sanitizeCommand(command: string): string {
  // Remove null bytes
  return command.replace(/\0/g, '');
}

/**
 * Checks if an environment variable value contains sensitive-looking data
 * @param value - The value to check
 * @returns True if the value might contain sensitive data
 */
export function containsSensitiveData(value: string): boolean {
  const sensitivePatterns = [
    /password/i,
    /secret/i,
    /token/i,
    /api[_-]?key/i,
    /private[_-]?key/i,
    /auth/i,
    /credential/i,
  ];

  return sensitivePatterns.some((pattern) => pattern.test(value));
}

/**
 * Simple encryption/decryption using Web Crypto API
 * Note: This provides basic obfuscation for localStorage.
 * For production, consider backend storage with proper encryption.
 */
export class StorageEncryption {
  private static readonly ALGORITHM = 'AES-GCM';
  private static readonly KEY_LENGTH = 256;
  private static key: CryptoKey | null = null;

  /**
   * Initialize encryption with a key derived from a password
   * @param password - Password to derive key from (should be user-specific)
   */
  static async initialize(password: string): Promise<void> {
    const encoder = new TextEncoder();
    const passwordData = encoder.encode(password);

    // Derive key from password using PBKDF2
    const baseKey = await crypto.subtle.importKey(
      'raw',
      passwordData,
      'PBKDF2',
      false,
      ['deriveBits', 'deriveKey']
    );

    this.key = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: encoder.encode('pulsar-workspace-salt'), // In production, use random salt per user
        iterations: 100000,
        hash: 'SHA-256',
      },
      baseKey,
      { name: this.ALGORITHM, length: this.KEY_LENGTH },
      false,
      ['encrypt', 'decrypt']
    );
  }

  /**
   * Encrypt data for storage
   * @param data - Data to encrypt
   * @returns Encrypted data as base64 string
   */
  static async encrypt(data: string): Promise<string> {
    if (!this.key) {
      throw new Error('Encryption not initialized. Call initialize() first.');
    }

    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);

    // Generate random IV
    const iv = crypto.getRandomValues(new Uint8Array(12));

    // Encrypt
    const encrypted = await crypto.subtle.encrypt(
      {
        name: this.ALGORITHM,
        iv,
      },
      this.key,
      dataBuffer
    );

    // Combine IV and encrypted data
    const combined = new Uint8Array(iv.length + encrypted.byteLength);
    combined.set(iv);
    combined.set(new Uint8Array(encrypted), iv.length);

    // Convert to base64
    return btoa(String.fromCharCode(...combined));
  }

  /**
   * Decrypt stored data
   * @param encryptedData - Encrypted data as base64 string
   * @returns Decrypted data
   */
  static async decrypt(encryptedData: string): Promise<string> {
    if (!this.key) {
      throw new Error('Encryption not initialized. Call initialize() first.');
    }

    // Decode from base64
    const combined = Uint8Array.from(atob(encryptedData), (c) => c.charCodeAt(0));

    // Extract IV and encrypted data
    const iv = combined.slice(0, 12);
    const data = combined.slice(12);

    // Decrypt
    const decrypted = await crypto.subtle.decrypt(
      {
        name: this.ALGORITHM,
        iv,
      },
      this.key,
      data
    );

    // Decode
    const decoder = new TextDecoder();
    return decoder.decode(decrypted);
  }

  /**
   * Check if encryption is initialized
   */
  static isInitialized(): boolean {
    return this.key !== null;
  }
}

/**
 * Secure localStorage wrapper with encryption
 */
export class SecureStorage {
  /**
   * Store data securely in localStorage
   * @param key - Storage key
   * @param value - Value to store (will be encrypted)
   */
  static async setItem(key: string, value: string): Promise<void> {
    if (StorageEncryption.isInitialized()) {
      const encrypted = await StorageEncryption.encrypt(value);
      localStorage.setItem(key, encrypted);
    } else {
      // Fallback to unencrypted if not initialized (with warning)
      console.warn('Storage encryption not initialized. Storing data unencrypted.');
      localStorage.setItem(key, value);
    }
  }

  /**
   * Retrieve data securely from localStorage
   * @param key - Storage key
   * @returns Decrypted value or null if not found
   */
  static async getItem(key: string): Promise<string | null> {
    const stored = localStorage.getItem(key);
    if (!stored) return null;

    if (StorageEncryption.isInitialized()) {
      try {
        return await StorageEncryption.decrypt(stored);
      } catch (error) {
        console.error('Failed to decrypt stored data:', error);
        return null;
      }
    } else {
      // Fallback to unencrypted
      console.warn('Storage encryption not initialized. Reading unencrypted data.');
      return stored;
    }
  }

  /**
   * Remove item from localStorage
   * @param key - Storage key
   */
  static removeItem(key: string): void {
    localStorage.removeItem(key);
  }
}
