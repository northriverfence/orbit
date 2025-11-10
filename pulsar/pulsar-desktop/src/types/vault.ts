/**
 * Vault Types
 *
 * TypeScript types for the Vault credential management system
 */

export type VaultState = 'uninitialized' | 'locked' | 'unlocked';

export type CredentialType = 'ssh_key' | 'password' | 'certificate';

/**
 * SSH Key credential data
 */
export interface SshKeyData {
  private_key: string;
  public_key?: string;
  passphrase?: string;
}

/**
 * Password credential data
 */
export interface PasswordData {
  password: string;
  username?: string;
}

/**
 * Certificate credential data
 */
export interface CertificateData {
  certificate: string;
  private_key?: string;
  passphrase?: string;
}

/**
 * Decrypted credential data (union type)
 */
export type DecryptedCredentialData =
  | { type: 'ssh_key'; ssh_key: SshKeyData }
  | { type: 'password'; password: PasswordData }
  | { type: 'certificate'; certificate: CertificateData };

/**
 * Credential summary (without decrypted data)
 */
export interface CredentialSummary {
  id: string;
  name: string;
  credential_type: CredentialType;
  tags: string[];
  created_at: number;
  updated_at: number;
  username?: string;
  host_pattern?: string;
}

/**
 * Decrypted credential (full data)
 */
export interface DecryptedCredential {
  id: string;
  name: string;
  data: DecryptedCredentialData;
  tags: string[];
  created_at: number;
  updated_at: number;
  username?: string;
  host_pattern?: string;
}

/**
 * Request to store a credential
 */
export interface StoreCredentialRequest {
  name: string;
  data: DecryptedCredentialData;
  tags: string[];
  username?: string;
  host_pattern?: string;
}

/**
 * Credential filter options
 */
export interface CredentialFilter {
  type?: CredentialType;
  search?: string;
  host?: string;
  tags?: string[];
}
