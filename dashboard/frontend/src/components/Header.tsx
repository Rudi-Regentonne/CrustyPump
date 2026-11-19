interface HeaderProps {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
}

export function Header({ sidebarOpen, onToggleSidebar }: HeaderProps) {
  return (
    <header>
      <div className="header-title">
        <h1>CrustyPump Dashboard</h1>
        <span className="subtitle">Connected to TimescaleDB</span>
      </div>
      <button
        id="sidebar-toggle"
        className="btn secondary mobile-only"
        type="button"
        aria-expanded={sidebarOpen}
        aria-controls="sidebar"
        onClick={onToggleSidebar}
      >
        Channels &amp; Layout
      </button>
    </header>
  );
}
