import { useState } from "react";
import sleetLogo from "./assets/sleet_icon.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [nearGreeting, setNearGreeting] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  async function fetchNearGreeting() {
    try {
      setIsLoading(true);
      setError("");
      const greeting = await invoke<string>("get_near_greeting");
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
      <img src={sleetLogo} alt="Sleet logo" className="sleet-logo" />
      <h1>hello.sleet.near</h1>
      <p>🧜‍♂️ a tauri hello project by sleet<br/>to interact with a hello smart contract on near</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">GET RUST GREETING</button>
      </form>
      <p>{greetMsg}</p>

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
