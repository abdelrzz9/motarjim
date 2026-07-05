import { CodeSkeleton } from '../../../design-system';
import { usePlaygroundStore } from '../../../stores/playgroundStore';

const STATUS_MESSAGES: Record<string, string> = {
  parsing_html: 'Parsing HTML structure...',
  parsing_css: 'Processing CSS styles...',
  building_ast: 'Building abstract syntax tree...',
  processing_javascript: 'Analyzing JavaScript...',
  building_ir: 'Building intermediate representation...',
  optimizing: 'Optimizing layout...',
  generating_code: 'Generating native UI code...',
  idle: 'Initializing...',
  complete: 'Complete!',
  failed: 'Failed',
};

export function LoadingState() {
  const { pipelineStage } = usePlaygroundStore();
  const status = STATUS_MESSAGES[pipelineStage] || STATUS_MESSAGES.idle;

  return (
    <div style={{
      flex: 1,
      display: 'flex',
      flexDirection: 'column',
      animation: 'fade-in 150ms var(--ease-out)',
      overflow: 'hidden',
    }}>
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: 'var(--space-3) var(--space-4)',
        borderBottom: '1px solid var(--border-subtle)',
      }}>
        <span style={{
          width: 12,
          height: 12,
          border: '2px solid var(--accent)',
          borderTopColor: 'transparent',
          borderRadius: '50%',
          display: 'inline-block',
          animation: 'spin 0.6s linear infinite',
          flexShrink: 0,
        }} />
        <span style={{
          fontSize: 11,
          color: 'var(--text-secondary)',
          fontWeight: 500,
        }}>
          {status}
        </span>
      </div>
      <CodeSkeleton lines={8} />
    </div>
  );
}
