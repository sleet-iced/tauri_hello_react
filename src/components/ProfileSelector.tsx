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
        value={currentProfile ? `${currentProfile.accountId}-${currentProfile.network}` : ''}
        onChange={(e) => {
          const [accountId, network] = e.target.value.split('-');
          const selectedProfile = availableProfiles.find(p => p.accountId === accountId && p.network === network);
          if (selectedProfile) {
            onProfileChange(selectedProfile);
          }
        }}
        className="profile-select"
      >
        {availableProfiles
          .map((profile) => {
            const displayName = `${profile.accountId}.${profile.network}`;
            return (
              <option key={`${profile.accountId}-${profile.network}`} value={`${profile.accountId}-${profile.network}`}>
                {displayName}
              </option>
            );
          })}
      </select>
    </div>
  );
}