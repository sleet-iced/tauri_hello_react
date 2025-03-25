import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { type NearCredential } from '../utils/near-credentials';
import './UpdateGreeting.css';

interface UpdateGreetingProps {
  currentProfile: NearCredential | null;
  network: 'mainnet' | 'testnet';
}

export function UpdateGreeting({ currentProfile, network }: UpdateGreetingProps) {
  const [newGreeting, setNewGreeting] = useState('');
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentProfile?.privateKey) {
      setError('No profile selected or private key not available');
      return;
    }

    setIsUpdating(true);
    setError(null);

    try {
      const result = await invoke<string>('update_near_greeting', {
        network,
        accountId: currentProfile.accountId,
        privateKey: currentProfile.privateKey,
        newGreeting,
      });
      setNewGreeting('');
      console.log(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update greeting');
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <div className="update-greeting">
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          value={newGreeting}
          onChange={(e) => setNewGreeting(e.target.value)}
          placeholder="Enter new greeting"
          disabled={isUpdating}
          className="greeting-input"
        />
        <button
          type="submit"
          disabled={!currentProfile || isUpdating || !newGreeting.trim()}
          className="update-button"
        >
          {isUpdating ? 'Updating...' : 'Update Greeting'}
        </button>
      </form>
      {error && <div className="error-message">{error}</div>}
    </div>
  );
}