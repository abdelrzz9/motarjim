import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { Icon } from '../../../components/Icons';
import type { PipelineStage } from '../../../services/types';
import styles from './PipelineVisualizer.module.css';

const STAGE_ORDER: PipelineStage[] = [
  'idle',
  'parsing_html',
  'parsing_css',
  'building_ast',
  'building_ir',
  'optimizing',
  'generating_code',
  'complete',
];

const STAGE_INFO: { id: PipelineStage; label: string; icon: typeof Icon.Code }[] = [
  { id: 'parsing_html', label: 'Parse', icon: Icon.Code },
  { id: 'parsing_css', label: 'Style', icon: Icon.Palette },
  { id: 'building_ast', label: 'AST', icon: Icon.Node },
  { id: 'building_ir', label: 'IR', icon: Icon.Node },
  { id: 'optimizing', label: 'Optimize', icon: Icon.Zap },
  { id: 'generating_code', label: 'Generate', icon: Icon.Play },
];

function getStageIndex(stage: PipelineStage): number {
  const idx = STAGE_ORDER.indexOf(stage);
  return idx >= 0 ? idx : -1;
}

export default function PipelineVisualizer() {
  const pipelineStage = usePlaygroundStore((s) => s.pipelineStage);
  const isCompiling = usePlaygroundStore((s) => s.isCompiling);
  const currentIdx = getStageIndex(pipelineStage);

  return (
    <div className={styles.pipeline} role="progressbar" aria-valuemin={0} aria-valuemax={STAGE_INFO.length - 1} aria-valuenow={Math.max(0, currentIdx)}>
      {STAGE_INFO.map((stage, i) => {
        const StageIcon = stage.icon;
        const isActive = currentIdx === i && isCompiling;
        const isDone = currentIdx > i;
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
            {i < STAGE_INFO.length - 1 && (
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
