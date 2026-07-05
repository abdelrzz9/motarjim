import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { Icon } from '../../../components/Icons';

export default function StatusBar() {
  const { stats, isCompiling, diagnostics, backendOnline } = usePlaygroundStore();
  const errors = diagnostics.filter((d) => d.severity === 'error').length;
  const warnings = diagnostics.filter((d) => d.severity === 'warning').length;

  return (
    <div className="statusBar">
      <div className="statusBarLeft">
        {!stats && !isCompiling && (
          <span className="statusBarItem">Ready</span>
        )}
        {isCompiling && (
          <span className="statusBarItem">Compiling...</span>
        )}
        {stats && (
          <>
            <span className="statusBarItem">
              <Icon.Zap size={11} />
              <span>{stats.timeMs < 1000 ? `${stats.timeMs.toFixed(0)}ms` : `${(stats.timeMs / 1000).toFixed(2)}s`}</span>
            </span>
            <span className="statusBarItem">
              <Icon.Code size={11} />
              <span>{stats.nodesParsed} nodes</span>
            </span>
            <span className="statusBarItem">
              <Icon.Palette size={11} />
              <span>{stats.cssRules} rules</span>
            </span>
          </>
        )}
      </div>

      <div className="statusBarRight">
        {warnings > 0 && (
          <span className="statusBarItem warning">
            <Icon.Warning size={11} /> {warnings}
          </span>
        )}
        {errors > 0 && (
          <span className="statusBarItem error">
            <Icon.Error size={11} /> {errors}
          </span>
        )}
        <span className="statusBarItem">
          <span className={`statusBarDot ${backendOnline ? 'online' : ''}`} />
          Engine
        </span>
      </div>
    </div>
  );
}
