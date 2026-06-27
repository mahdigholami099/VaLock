import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";

function AllMap({ onNavigate }) {
  const [agents, setAgents] = useState([]);
  const [selectedAgent, setSelectedAgent] = useState(null);

  useEffect(() => {
    invoke("get_agents")
      .then((data) => setAgents(data))
      .catch((error) => alert(error));
  }, []);

  const handleSelect = (agent) => {
    setSelectedAgent(agent);
  };

  const handleSave = async () => {
    if (!selectedAgent) return;
    try {
      await invoke("set_agent_for_all_maps", { name: selectedAgent.name });
      alert("Saved!");
      onNavigate("home");
    } catch (error) {
      alert(error);
    }
  };

  return (
    <div>
      <div className="content-header">
        <h1>ALL MAPS</h1>
        <p>Select one agent for all maps</p>
      </div>

      <div style={{ padding: "32px" }}>
        <div className="config-panel">
          <h3>Select Agent</h3>
          <div className="agent-grid">
            {agents.map((agent, index) => (
              <div
                key={index}
                className={`agent-card ${
                  selectedAgent?.name === agent.name ? "selected" : ""
                }`}
                onClick={() => handleSelect(agent)}
              >
                <img src={agent.icon} alt={agent.name} />
                <div className="agent-name">{agent.name}</div>
              </div>
            ))}
          </div>
        </div>

        <div style={{ marginTop: "24px", display: "flex", gap: "12px" }}>
          <button className="btn btn-secondary" onClick={() => onNavigate("home")}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSave}
            disabled={!selectedAgent}
          >
            Save Configuration
          </button>
        </div>
      </div>
    </div>
  );
}

export default AllMap;
