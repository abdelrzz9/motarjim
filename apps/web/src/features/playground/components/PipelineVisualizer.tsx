import { PIPELINE_STAGES } from '../../../utils/constants';
import styles from './PipelineVisualizer.module.css';

export default function PipelineVisualizer() {
  return (
    <div className={styles.pipeline}>
      {PIPELINE_STAGES.map((stage, i) => (
        <div key={stage.id} className={styles.stage}>
          <div className={styles.node}>
            <div className={styles.label}>{stage.label}</div>
            <div className={styles.description}>{stage.description}</div>
          </div>
          {i < PIPELINE_STAGES.length - 1 && (
            <div className={styles.arrow}>&rarr;</div>
          )}
        </div>
      ))}
    </div>
  );
}
