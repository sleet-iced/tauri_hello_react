import './ProfileSelector.css';

type Profile = {
  accountId: string;
  publicKey: string;
};

interface ProfileSelectorProps {
  onProfileChange: (profile: Profile) => void;
  currentProfile: Profile;
  availableProfiles: Profile[];
}

export function ProfileSelector({ onProfileChange, currentProfile, availableProfiles }: ProfileSelectorProps) {
  return (
    <div className="profile-selector">
      <select
        value={currentProfile.accountId}
        onChange={(e) => {
          const selectedProfile = availableProfiles.find(p => p.accountId === e.target.value);
          if (selectedProfile) {
            onProfileChange(selectedProfile);
          }
        }}
        className="profile-select"
      >
        {availableProfiles.map((profile) => (
          <option key={profile.publicKey} value={profile.accountId}>
            {profile.accountId}
          </option>
        ))}
      </select>
    </div>
  );
}