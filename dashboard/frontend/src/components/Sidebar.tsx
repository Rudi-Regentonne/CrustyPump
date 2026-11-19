import type { Channel } from '../types';

interface SidebarProps {
  channels: Channel[];
  selectedChannel: string;
  onSelectChannel: (value: string) => void;
  onAddTile: () => void;
  onSaveConfig: () => void;
  onReloadConfig: () => void;
  statusMsg: { text: string; isError: boolean } | null;
  isOpen: boolean;
}

export function Sidebar({
  channels,
  selectedChannel,
  onSelectChannel,
  onAddTile,
  onSaveConfig,
  onReloadConfig,
  statusMsg,
  isOpen,
}: SidebarProps) {
  return (
    <aside id="sidebar" className={isOpen ? 'open' : ''}>
      <h3>Available Channels</h3>
      <div className="input-group">
        <select
          id="channel-selector"
          value={selectedChannel}
          onChange={(e) => onSelectChannel(e.target.value)}
        >
          <option value="">Select a channel...</option>
          {channels.map((c) => {
            const val = `${c.measurement}:${c.channel}`;
            return (
              <option key={val} value={val}>
                {c.channel}
              </option>
            );
          })}
        </select>
        <button id="add-btn" className="btn success w-100" type="button" onClick={onAddTile}>
          Add to Dashboard
        </button>
      </div>

      <hr />

      <h4>Layout Actions</h4>
      <button id="save-btn" className="btn warning w-100 mb-10" type="button" onClick={onSaveConfig}>
        Save Current Layout
      </button>
      <button id="load-btn" className="btn secondary w-100" type="button" onClick={onReloadConfig}>
        Reload Saved Layout
      </button>
      <div
        id="status-msg"
        style={{ color: statusMsg?.isError ? '#d32f2f' : '#4caf50' }}
      >
        {statusMsg?.text ?? ''}
      </div>
    </aside>
  );
}
