import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { Icon } from '../../../components/Icons';
import styles from './PipelineVisualizer.module.css';

const STAGES = [
  { id: 'parse', label: 'Parse', icon: Icon.Code },
  { id: 'style', label: 'Style', icon: Icon.Palette },
  { id: 'analyze', label: 'Analyze', icon: Icon.Search },
  { id: 'ir', label: 'IR', icon: Icon.Node },
  { id: 'optimize', label: 'Optimize', icon: Icon.Zap },
  { id: 'generate', label: 'Generate', icon: Icon.Play },
];

export default function PipelineVisualizer() {
  const pipelineStage = usePlaygroundStore((s) => s.pipelineStage);

  return (
    <div className={styles.pipeline} role="progressbar" aria-valuemin={0} aria-valuemax={STAGES.length - 1} aria-valuenow={Math.max(0, pipelineStage)}>
      {STAGES.map((stage, i) => {
        const StageIcon = stage.icon;
        const isActive = i === pipelineStage;
        const isDone = i < pipelineStage;
        const stageClass = isDone ? styles.stageDone : isActive ? styles.stageActive : '';
        return (
          <div
            key={stage.id}
            className={`${styles.stage} ${stageClass}`}
            title={stage.label}
          >
            <div className={`${styles.stageIcon} ${isActive ? styles.pulse : ''}`}>
              <StageIcon size={12} />
            </div>
            <span className={styles.stageLabel}>{stage.label}</span>
            {i < STAGES.length - 1 && (
              <div className={`${styles.link} ${isDone ? styles.linkDone : isActive ? styles.linkActive : ''}`}>
                <div className={styles.linkInner} />
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
