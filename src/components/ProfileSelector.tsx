import './ProfileSelector.css';

import { type NearCredential } from '../utils/near-credentials';

interface ProfileSelectorProps {
  onProfileChange: (profile: NearCredential) => void;
  currentProfile: NearCredential | null;
  availableProfiles: NearCredential[];
}

export function ProfileSelector({ onProfileChange, currentProfile, availableProfiles }: ProfileSelectorProps) {
  return (
    <div className="profile-selector">
      <select
        value={currentProfile?.accountId || ''}
        onChange={(e) => {
          const selectedProfile = availableProfiles.find(p => p.accountId === e.target.value) || null;
          if (selectedProfile) {
            onProfileChange(selectedProfile);
          }
        }}
        className="profile-select"
      >
        {availableProfiles.map((profile) => (
          <option key={`${profile.accountId}-${profile.network}`} value={`${profile.accountId}-${profile.network}`}>
            {`${profile.accountId} (${profile.network})`}
          </option>
        ))}
      </select>
    </div>
  );
}