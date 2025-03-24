import { useState, useEffect } from "react";
import sleetLogo from "./assets/sleet_icon.svg";
import { invoke } from "@tauri-apps/api/core";
import { NetworkSelector } from "./components/NetworkSelector";
import { ProfileSelector } from "./components/ProfileSelector";
import { loadNearCredentials, type NearCredential } from "./utils/near-credentials";
import "./App.css";

function App() {
  const [nearGreeting, setNearGreeting] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [network, setNetwork] = useState<'testnet' | 'mainnet'>('testnet');
  const [profiles, setProfiles] = useState<NearCredential[]>([]);
  const [currentProfile, setCurrentProfile] = useState<NearCredential | null>(null);

  useEffect(() => {
    async function loadProfiles() {
      const response = await loadNearCredentials();
      if (response.error) {
        setError(response.error);
        return;
      }
      const networkProfiles = response.credentials.filter(cred => cred.network === network);
      setProfiles(networkProfiles);
      if (networkProfiles.length > 0) {
        setCurrentProfile(networkProfiles[0]);
      }
    }
    loadProfiles();
  }, [network]);

  async function fetchNearGreeting() {
    try {
      setIsLoading(true);
      setError("");
      const greeting = await invoke<string>("get_near_greeting", { network });
      setNearGreeting(greeting);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error("NEAR greeting error:", err);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <main className="container">
      <div className="header">
        <NetworkSelector onNetworkChange={setNetwork} currentNetwork={network} />
        {currentProfile && (
          <ProfileSelector
            onProfileChange={setCurrentProfile}
            currentProfile={currentProfile}
            availableProfiles={profiles}
          />
        )}
      </div>
      <img src={sleetLogo} alt="Sleet logo" className="sleet-logo" />
      <h1>hello.sleet.near</h1>
      <p>🧜‍♂️ a tauri hello project by sleet<br/>to interact with a hello smart contract on near</p>

      <div className="near-greeting">
        <button 
          onClick={fetchNearGreeting}
          disabled={isLoading}
        >
          {isLoading ? "Loading..." : "Get NEAR Contract Greeting"}
        </button>
        {nearGreeting && <p>Contract says: {nearGreeting}</p>}
        {error && <p className="error">{error}</p>}
      </div>
    </main>
  );
}

export default App;
