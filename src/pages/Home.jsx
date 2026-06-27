import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

function Home({ onNavigate }) {
  const [isRunning, setIsRunning] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    checkRunningState();
    const unlisten = listen("instalock-stopped", () => {
      setIsRunning(false);
    });
    return () => unlisten.then((fn) => fn());
  }, []);

  const checkRunningState = async () => {
    try {
      const running = await invoke("is_instalock_running");
      setIsRunning(running);
    } catch (error) {
      console.error("Failed to check running state:", error);
    }
  };

  const handleToggle = async () => {
    setIsLoading(true);
    try {
      if (isRunning) {
        await invoke("stop_instalock");
        setIsRunning(false);
      } else {
        await invoke("start_instalock");
        setIsRunning(true);
      }
    } catch (error) {
      alert(error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div>
      <div className="content-header">
        <h1>VALOCK</h1>
        <p>Valorant Insta-Lock Assistant</p>
      </div>

      <div style={{ padding: "32px" }}>
        <div className="home-grid">
          <div className="home-card" onClick={() => onNavigate("allmap")}>
            <div className="card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="3" y="3" width="7" height="7" />
                <rect x="14" y="3" width="7" height="7" />
                <rect x="14" y="14" width="7" height="7" />
                <rect x="3" y="14" width="7" height="7" />
              </svg>
            </div>
            <h3>All Maps</h3>
            <p>Choose one agent that will be auto-locked for every map</p>
          </div>

          <div className="home-card" onClick={() => onNavigate("eachmap")}>
            <div className="card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polygon points="12 2 2 7 12 12 22 7 12 2" />
                <polyline points="2 17 12 22 22 17" />
                <polyline points="2 12 12 17 22 12" />
              </svg>
            </div>
            <h3>Each Map</h3>
            <p>Configure a different agent for each individual map</p>
          </div>

          <div className="home-card" onClick={() => onNavigate("profile")}>
            <div className="card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                <circle cx="12" cy="7" r="4" />
              </svg>
            </div>
            <h3>Profiles</h3>
            <p>Create and manage different agent configurations</p>
          </div>

          <div className="home-card" onClick={handleToggle} style={{ borderColor: isRunning ? "var(--accent)" : "var(--border)" }}>
            <div className="card-icon" style={{ background: isRunning ? "rgba(255, 70, 85, 0.2)" : "var(--bg-tertiary)" }}>
<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                {isRunning ? (
                  <>
                    <rect x="6" y="4" width="4" height="16" />
                    <rect x="14" y="4" width="4" height="16" />
                  </>
                ) : (
                  <polygon points="5 3 19 12 5 21 5 3" />
                )}
              </svg>
            </div>
            <h3>{isRunning ? "Stop Insta-Lock" : "Start Insta-Lock"}</h3>
            <p>{isRunning ? "Monitoring for matches..." : "Begin monitoring for matches and auto-lock your agent"}</p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Home;