import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { PLATFORMS } from '../../../utils/constants';
import styles from './Toolbar.module.css';

interface ToolbarProps {
  onCompile: () => void;
  isCompiling: boolean;
}

export default function Toolbar({ onCompile, isCompiling }: ToolbarProps) {
  const { platform, minify, setPlatform, setMinify } = usePlaygroundStore();

  return (
    <div className={styles.toolbar}>
      <div className={styles.group}>
        <label className={styles.label}>Platform:</label>
        <select
          className={styles.select}
          value={platform}
          onChange={(e) => setPlatform(e.target.value as typeof platform)}
        >
          {PLATFORMS.map((p) => (
            <option key={p.value} value={p.value}>{p.label}</option>
          ))}
        </select>
      </div>

      <div className={styles.group}>
        <label className={styles.checkbox}>
          <input
            type="checkbox"
            checked={minify}
            onChange={(e) => setMinify(e.target.checked)}
          />
          Minify
        </label>
      </div>

      <div className={styles.spacer} />

      <button
        className={styles.compileBtn}
        onClick={onCompile}
        disabled={isCompiling}
      >
        {isCompiling ? 'Compiling...' : 'Compile'}
      </button>
    </div>
  );
}
