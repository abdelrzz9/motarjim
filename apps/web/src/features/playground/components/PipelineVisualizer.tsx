import { usePlaygroundStore } from '../../../stores/playgroundStore';
import { Icon } from '../../../components/Icons';

interface StageDef {
  id: string;
  label: string;
  pipelineStage: string;
  icon: typeof Icon.Code;
}

const STAGES: StageDef[] = [
  { id: 'parse', label: 'Parse', pipelineStage: 'parsing_html', icon: Icon.Code },
  { id: 'style', label: 'Style', pipelineStage: 'parsing_css', icon: Icon.Palette },
  { id: 'analyze', label: 'Analyze', pipelineStage: 'building_ast', icon: Icon.Search },
  { id: 'ir', label: 'IR', pipelineStage: 'building_ir', icon: Icon.Layers },
  { id: 'optimize', label: 'Optimize', pipelineStage: 'optimizing', icon: Icon.Zap },
  { id: 'generate', label: 'Generate', pipelineStage: 'generating_code', icon: Icon.Sparkles },
];

const stageOrder = [
  'parsing_html', 'parsing_css', 'building_ast',
  'processing_javascript', 'building_ir', 'optimizing',
  'generating_code', 'complete', 'failed',
];

function getStageStatus(stage: StageDef, currentStage: string): 'idle' | 'active' | 'completed' {
  const stageIdx = stageOrder.indexOf(stage.pipelineStage);
  const currentIdx = stageOrder.indexOf(currentStage);
  if (currentStage === 'complete' || currentStage === 'idle') {
    return currentStage === 'complete' ? 'completed' : 'idle';
  }
  if (currentIdx < 0) return 'idle';
  if (stageIdx < currentIdx) return 'completed';
  if (stageIdx === currentIdx) return 'active';
  return 'idle';
}

export function PipelineVisualizer() {
  const { pipelineStage, isCompiling } = usePlaygroundStore();

  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 0,
      flex: 1,
      maxWidth: 560,
      margin: '0 auto',
    }}>
      {STAGES.map((stage, i) => {
        const status = getStageStatus(stage, pipelineStage);
        const StageIcon = stage.icon;
        const isLast = i === STAGES.length - 1;

        return (
          <div key={stage.id} style={{
            display: 'flex',
            alignItems: 'center',
            flex: isLast ? 0 : 1,
          }}>
            <div style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '4px 8px',
              borderRadius: 6,
              transition: 'all 200ms var(--ease-out)',
              background: status === 'active' ? 'var(--accent-soft)' : 'transparent',
            }}>
              <span style={{
                display: 'flex',
                width: 16,
                height: 16,
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                color: status === 'completed' ? 'var(--success)'
                  : status === 'active' ? 'var(--accent)'
                  : 'var(--text-tertiary)',
                animation: status === 'active' ? 'pulse-glow 2s ease-in-out infinite' : undefined,
              }}>
                {status === 'completed' ? (
                  <span style={{ animation: 'check-pop 0.3s var(--ease-spring)' }}>
                    <Icon.Check size={14} />
                  </span>
                ) : status === 'active' && isCompiling ? (
                  <span style={{
                    width: 14, height: 14,
                    border: '2px solid var(--accent)',
                    borderTopColor: 'transparent',
                    borderRadius: '50%',
                    display: 'inline-block',
                    animation: 'spin 0.6s linear infinite',
                  }} />
                ) : (
                  <StageIcon size={14} />
                )}
              </span>
              <span style={{
                fontSize: 10,
                fontWeight: 600,
                color: status === 'completed' ? 'var(--success)'
                  : status === 'active' ? 'var(--accent)'
                  : 'var(--text-tertiary)',
                textTransform: 'uppercase',
                letterSpacing: '0.04em',
                whiteSpace: 'nowrap',
                transition: 'color 200ms var(--ease-out)',
              }}>
                {stage.label}
              </span>
            </div>

            {!isLast && (
              <div style={{
                flex: 1,
                height: 1,
                margin: '0 4px',
                background: status === 'completed'
                  ? 'var(--success)'
                  : pipelineStage !== 'idle' && stageOrder.indexOf(stage.pipelineStage) < stageOrder.indexOf(pipelineStage)
                    ? 'var(--success)'
                    : 'var(--border-subtle)',
                transition: 'background 300ms var(--ease-out)',
                opacity: status === 'idle' ? 0.3 : 1,
              }} />
            )}
          </div>
        );
      })}
    </div>
  );
}
