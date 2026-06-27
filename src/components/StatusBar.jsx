function StatusBar({ status }) {
  const getStatusClass = () => {
    if (status.running) return "running";
    if (status.message.includes("Error")) return "error";
    return "";
  };

  return (
    <div className="status-bar">
      <div className="status-indicator">
        <div className={`status-dot ${getStatusClass()}`} />
        <span>Insta-Lock</span>
      </div>
      <div className="status-message">{status.message}</div>
    </div>
  );
}

export default StatusBar;
