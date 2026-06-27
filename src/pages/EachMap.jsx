import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";

function EachMap({ onNavigate }) {
  const [agents, setAgents] = useState([]);
  const [maps, setMaps] = useState([]);
  const [config, setConfig] = useState({});
  const [selectedMap, setSelectedMap] = useState(null);
  const [selectedAgent, setSelectedAgent] = useState(null);

  useEffect(() => {
    const loadData = async () => {
      try {
        const agentsData = await invoke("get_agents");
        const mapsData = await invoke("get_maps");
        const configData = await invoke("get_config");

        setAgents(agentsData);
        setMaps(mapsData);
        setConfig(configData);

        if (mapsData.length > 0) {
          setSelectedMap(mapsData[0]);
          const mapName = mapsData[0].name;
          if (configData[mapName]) {
            const agent = agentsData.find((a) => a.name === configData[mapName]);
            if (agent) setSelectedAgent(agent);
          }
        }
      } catch (error) {
        alert(error);
      }
    };

    loadData();
  }, []);

  const handleMapSelect = (map) => {
    setSelectedMap(map);
    const agentName = config[map.name];
    if (agentName) {
      const agent = agents.find((a) => a.name === agentName);
      if (agent) setSelectedAgent(agent);
    } else {
      setSelectedAgent(null);
    }
  };

  const handleAgentSelect = (agent) => {
    setSelectedAgent(agent);
  };

  const handleSave = async () => {
    if (!selectedAgent || !selectedMap) return;
    try {
      await invoke("set_agent_for_map", {
        agent: selectedAgent.name,
        map: selectedMap.name,
      });
      setConfig((prev) => ({ ...prev, [selectedMap.name]: selectedAgent.name }));
      alert("Saved!");
    } catch (error) {
      alert(error);
    }
  };

  return (
    <div>
      <div className="content-header">
        <h1>EACH MAP</h1>
        <p>Configure a different agent for each map</p>
      </div>

      <div style={{ padding: "32px" }}>
        <div className="config-panel" style={{ marginBottom: "24px" }}>
          <h3>Select Map</h3>
          <div className="map-grid">
            {maps.map((map, index) => (
              <div
                key={index}
                className={`map-card ${
                  selectedMap?.name === map.name ? "selected" : ""
                }`}
                onClick={() => handleMapSelect(map)}
              >
                {map.full_portrait && map.full_portrait !== "null" ? (
                  <img src={map.full_portrait} alt={map.name} />
                ) : (
                  <div
                    style={{
                      width: "100%",
                      aspectRatio: "16/9",
                      background: "var(--bg-tertiary)",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      color: "var(--text-muted)",
                    }}
                  >
                    No Image
                  </div>
                )}
                <div className="map-name">{map.name}</div>
              </div>
            ))}
          </div>
        </div>

        <div className="config-panel">
          <h3>
            Select Agent for {selectedMap?.name || "..."}
            {config[selectedMap?.name] && (
              <span
                style={{
                  fontSize: "14px",
                  color: "var(--accent)",
                  marginLeft: "12px",
                }}
              >
                Current: {config[selectedMap.name]}
              </span>
            )}
          </h3>
          <div className="agent-grid">
            {agents.map((agent, index) => (
              <div
                key={index}
                className={`agent-card ${
                  selectedAgent?.name === agent.name ? "selected" : ""
                }`}
                onClick={() => handleAgentSelect(agent)}
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
            disabled={!selectedAgent || !selectedMap}
          >
            Save Configuration
          </button>
        </div>
      </div>
    </div>
  );
}

export default EachMap;
