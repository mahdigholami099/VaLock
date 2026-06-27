import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

function Profile({ profiles, activeProfile, onRefresh }) {
  const [newName, setNewName] = useState("");

  const handleAdd = async () => {
    if (!newName.trim()) return;
    try {
      await invoke("add_profile", { name: newName.trim() });
      setNewName("");
      onRefresh();
    } catch (error) {
      alert(error);
    }
  };

  const handleDelete = async (name) => {
    if (!confirm(`Delete profile "${name}"?`)) return;
    try {
      await invoke("delete_profile", { name });
      onRefresh();
    } catch (error) {
      alert(error);
    }
  };

  const handleSetActive = async (name) => {
    try {
      await invoke("set_active_profile", { name });
      onRefresh();
    } catch (error) {
      alert(error);
    }
  };

  return (
    <div>
      <div className="content-header">
        <h1>PROFILES</h1>
        <p>Manage your agent configurations</p>
      </div>

      <div style={{ padding: "32px" }}>
        <div className="profile-layout">
          <div className="profile-section">
            <h3>Create New Profile</h3>
            <div className="input-group">
              <input
                type="text"
                placeholder="Enter profile name"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleAdd()}
              />
              <button className="btn btn-primary" onClick={handleAdd}>
                Add
              </button>
            </div>
          </div>

          <div className="profile-section" style={{ flex: 2 }}>
            <h3>Your Profiles</h3>
            <div className="profile-list">
              {profiles.length === 0 ? (
                <div
                  style={{
                    textAlign: "center",
                    color: "var(--text-muted)",
                    padding: "32px",
                  }}
                >
                  No profiles yet. Create one to get started.
                </div>
              ) : (
                profiles.map((profile) => (
                  <div
                    key={profile}
                    className={`profile-item ${
                      profile === activeProfile ? "active" : ""
                    }`}
                  >
                    <span className="profile-name">{profile}</span>
                    <div className="profile-actions">
                      {profile !== activeProfile && (
                        <button
                          className="btn btn-ghost"
                          onClick={() => handleSetActive(profile)}
                          style={{ padding: "6px 12px", fontSize: "12px" }}
                        >
                          Set Active
                        </button>
                      )}
                      {profile === activeProfile && (
                        <span
                          style={{
                            fontSize: "12px",
                            color: "var(--accent)",
                            padding: "6px 12px",
                          }}
                        >
                          Active
                        </span>
                      )}
                      <button
                        className="btn btn-ghost"
                        onClick={() => handleDelete(profile)}
                        style={{
                          padding: "6px 12px",
                          fontSize: "12px",
                          color: "var(--error)",
                        }}
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Profile;
