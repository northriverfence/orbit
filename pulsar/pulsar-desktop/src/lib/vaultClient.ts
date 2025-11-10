/**
 * Vault API Client
 *
 * Provides a clean interface for interacting with the vault system
 * via Tauri commands to the backend
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  VaultState,
  CredentialType,
  CredentialSummary,
  DecryptedCredential,
  StoreCredentialRequest,
} from '../types/vault';

export class VaultClient {
  /**
   * Get the current vault state
   */
  static async getState(): Promise<VaultState> {
    try {
      const state = await invoke<string>('vault_get_state');
      return state as VaultState;
    } catch (error) {
      console.error('Failed to get vault state:', error);
      throw new Error(`Failed to get vault state: ${error}`);
    }
  }

  /**
   * Check if vault is initialized
   */
  static async isInitialized(): Promise<boolean> {
    try {
      return await invoke<boolean>('vault_is_initialized');
    } catch (error) {
      console.error('Failed to check if vault is initialized:', error);
      throw new Error(`Failed to check if vault is initialized: ${error}`);
    }
  }

  /**
   * Check if vault is unlocked
   */
  static async isUnlocked(): Promise<boolean> {
    try {
      return await invoke<boolean>('vault_is_unlocked');
    } catch (error) {
      console.error('Failed to check if vault is unlocked:', error);
      throw new Error(`Failed to check if vault is unlocked: ${error}`);
    }
  }

  /**
   * Initialize vault with a master password
   */
  static async initialize(masterPassword: string): Promise<void> {
    try {
      await invoke('vault_initialize', { masterPassword });
    } catch (error) {
      console.error('Failed to initialize vault:', error);
      throw new Error(`Failed to initialize vault: ${error}`);
    }
  }

  /**
   * Unlock vault with master password
   */
  static async unlock(masterPassword: string): Promise<void> {
    try {
      await invoke('vault_unlock', { masterPassword });
    } catch (error) {
      console.error('Failed to unlock vault:', error);
      throw new Error(`Failed to unlock vault: ${error}`);
    }
  }

  /**
   * Lock vault
   */
  static async lock(): Promise<void> {
    try {
      await invoke('vault_lock');
    } catch (error) {
      console.error('Failed to lock vault:', error);
      throw new Error(`Failed to lock vault: ${error}`);
    }
  }

  /**
   * Store a generic credential
   */
  static async storeCredential(request: StoreCredentialRequest): Promise<string> {
    try {
      return await invoke<string>('vault_store_credential', { request });
    } catch (error) {
      console.error('Failed to store credential:', error);
      throw new Error(`Failed to store credential: ${error}`);
    }
  }

  /**
   * Store an SSH key
   */
  static async storeSshKey(
    name: string,
    privateKey: string,
    publicKey?: string,
    passphrase?: string,
    tags: string[] = [],
    username?: string,
    hostPattern?: string
  ): Promise<string> {
    try {
      return await invoke<string>('vault_store_ssh_key', {
        name,
        privateKey,
        publicKey,
        passphrase,
        tags,
        username,
        hostPattern,
      });
    } catch (error) {
      console.error('Failed to store SSH key:', error);
      throw new Error(`Failed to store SSH key: ${error}`);
    }
  }

  /**
   * Store a password
   */
  static async storePassword(
    name: string,
    password: string,
    username?: string,
    tags: string[] = [],
    hostPattern?: string
  ): Promise<string> {
    try {
      return await invoke<string>('vault_store_password', {
        name,
        password,
        username,
        tags,
        hostPattern,
      });
    } catch (error) {
      console.error('Failed to store password:', error);
      throw new Error(`Failed to store password: ${error}`);
    }
  }

  /**
   * Store a certificate
   */
  static async storeCertificate(
    name: string,
    certificate: string,
    privateKey?: string,
    passphrase?: string,
    tags: string[] = [],
    username?: string,
    hostPattern?: string
  ): Promise<string> {
    try {
      return await invoke<string>('vault_store_certificate', {
        name,
        certificate,
        privateKey,
        passphrase,
        tags,
        username,
        hostPattern,
      });
    } catch (error) {
      console.error('Failed to store certificate:', error);
      throw new Error(`Failed to store certificate: ${error}`);
    }
  }

  /**
   * Get a decrypted credential by ID
   */
  static async getCredential(id: string): Promise<DecryptedCredential> {
    try {
      return await invoke<DecryptedCredential>('vault_get_credential', { id });
    } catch (error) {
      console.error('Failed to get credential:', error);
      throw new Error(`Failed to get credential: ${error}`);
    }
  }

  /**
   * List all credentials (summaries only, without decrypting)
   */
  static async listCredentials(): Promise<CredentialSummary[]> {
    try {
      return await invoke<CredentialSummary[]>('vault_list_credentials');
    } catch (error) {
      console.error('Failed to list credentials:', error);
      throw new Error(`Failed to list credentials: ${error}`);
    }
  }

  /**
   * List credentials by type
   */
  static async listCredentialsByType(credentialType: CredentialType): Promise<CredentialSummary[]> {
    try {
      return await invoke<CredentialSummary[]>('vault_list_credentials_by_type', {
        credentialType,
      });
    } catch (error) {
      console.error('Failed to list credentials by type:', error);
      throw new Error(`Failed to list credentials by type: ${error}`);
    }
  }

  /**
   * Find credentials by host pattern
   */
  static async findCredentialsByHost(host: string): Promise<CredentialSummary[]> {
    try {
      return await invoke<CredentialSummary[]>('vault_find_credentials_by_host', { host });
    } catch (error) {
      console.error('Failed to find credentials by host:', error);
      throw new Error(`Failed to find credentials by host: ${error}`);
    }
  }

  /**
   * Delete a credential
   */
  static async deleteCredential(id: string): Promise<void> {
    try {
      await invoke('vault_delete_credential', { id });
    } catch (error) {
      console.error('Failed to delete credential:', error);
      throw new Error(`Failed to delete credential: ${error}`);
    }
  }

  /**
   * Get SSH keys only
   */
  static async getSshKeys(): Promise<CredentialSummary[]> {
    return this.listCredentialsByType('ssh_key');
  }

  /**
   * Get passwords only
   */
  static async getPasswords(): Promise<CredentialSummary[]> {
    return this.listCredentialsByType('password');
  }

  /**
   * Get certificates only
   */
  static async getCertificates(): Promise<CredentialSummary[]> {
    return this.listCredentialsByType('certificate');
  }

  /**
   * Search credentials (client-side filtering)
   */
  static async searchCredentials(query: string): Promise<CredentialSummary[]> {
    const all = await this.listCredentials();
    const lowerQuery = query.toLowerCase();

    return all.filter(
      (cred) =>
        cred.name.toLowerCase().includes(lowerQuery) ||
        cred.tags.some((tag) => tag.toLowerCase().includes(lowerQuery)) ||
        (cred.username && cred.username.toLowerCase().includes(lowerQuery)) ||
        (cred.host_pattern && cred.host_pattern.toLowerCase().includes(lowerQuery))
    );
  }
}

export default VaultClient;
