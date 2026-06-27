import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import Sidebar from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import Home from "./pages/Home";
import AllMap from "./pages/AllMap";
import EachMap from "./pages/EachMap";
import Profile from "./pages/Profile";

function App() {
  const [page, setPage] = useState("home");
  const [profiles, setProfiles] = useState([]);
  const [activeProfile, setActiveProfile] = useState("default");
  const [instaLockStatus, setInstaLockStatus] = useState({
    running: false,
    message: "Ready",
  });

  useEffect(() => {
    loadData();
    setupEventListeners();
  }, []);

  const loadData = async () => {
    try {
      const allProfiles = await invoke("get_profiles");
      const active = await invoke("get_active_profile");
      setProfiles(allProfiles);
      setActiveProfile(active);
    } catch (error) {
      console.error("Failed to load data:", error);
    }
  };

  const setupEventListeners = async () => {
    await listen("instalock-status", (event) => {
      setInstaLockStatus({ running: true, message: event.payload });
    });

    await listen("instalock-stopped", () => {
      setInstaLockStatus({ running: false, message: "Ready" });
    });
  };

  const handleProfileChange = async (profileName) => {
    try {
      await invoke("set_active_profile", { name: profileName });
      setActiveProfile(profileName);
    } catch (error) {
      console.error("Failed to set profile:", error);
    }
  };

  const renderPage = () => {
    switch (page) {
      case "allmap":
        return <AllMap onNavigate={setPage} />;
      case "eachmap":
        return <EachMap onNavigate={setPage} />;
      case "profile":
        return (
          <Profile
            profiles={profiles}
            activeProfile={activeProfile}
            onRefresh={loadData}
          />
        );
      default:
        return <Home onNavigate={setPage} />;
    }
  };

  return (
    <div className="app-layout">
      <Sidebar
        page={page}
        onNavigate={setPage}
        profiles={profiles}
        activeProfile={activeProfile}
        onProfileChange={handleProfileChange}
      />
      <div className="main-content">
        <div className="content-body">{renderPage()}</div>
        <StatusBar status={instaLockStatus} />
      </div>
    </div>
  );
}

export default App;
