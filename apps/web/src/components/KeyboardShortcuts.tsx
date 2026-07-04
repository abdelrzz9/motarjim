const SHORTCUTS = [
  { keys: 'Ctrl+Enter', action: 'Compile' },
  { keys: 'Ctrl+S', action: 'Save / Compile' },
  { keys: 'Ctrl+Shift+P', action: 'Toggle platform' },
  { keys: 'Escape', action: 'Clear diagnostics' },
];

export default function KeyboardShortcuts() {
  return (
    <div>
      <h3>Keyboard Shortcuts</h3>
      <table>
        <thead>
          <tr>
            <th>Keys</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {SHORTCUTS.map((shortcut) => (
            <tr key={shortcut.keys}>
              <td><kbd>{shortcut.keys}</kbd></td>
              <td>{shortcut.action}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
