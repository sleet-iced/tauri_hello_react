import { invoke } from '@tauri-apps/api/tauri';

export interface NearCredential {
  accountId: string;
  publicKey: string;
  network: 'mainnet' | 'testnet';
  privateKey?: string;
}

export interface CredentialResponse {
  credentials: NearCredential[];
  error?: string;
}

export async function loadNearCredentials(): Promise<CredentialResponse> {
  try {
    // This will be implemented in Rust to securely read credentials
    const response = await invoke<CredentialResponse>('load_near_credentials');
    return response;
  } catch (error) {
    return {
      credentials: [],
      error: error instanceof Error ? error.message : 'Failed to load credentials'
    };
  }
}

export function formatAccountId(accountId: string): string {
  if (accountId.length > 20) {
    return `${accountId.slice(0, 8)}...${accountId.slice(-8)}`;
  }
  return accountId;
}