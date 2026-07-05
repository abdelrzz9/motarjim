import { useCallback, useMemo, useState } from 'react';
import { EditorPanel } from './components/EditorPanel';
import { OutputPanel } from './components/OutputPanel';
import { SplitPanel } from '../../components/SplitPanel';
import { CommandPalette } from '../../components/CommandPalette';
import { KeyboardShortcutModal } from '../../components/KeyboardShortcutModal';
import { useKeyboard } from '../../hooks/useKeyboard';
import { Icon } from '../../components/Icons';

export default function PlaygroundPage() {
  const [cmdPaletteOpen, setCmdPaletteOpen] = useState(false);
  const [shortcutModalOpen, setShortcutModalOpen] = useState(false);

  const compile = useCallback(() => {
    document.dispatchEvent(new CustomEvent('compile-trigger'));
  }, []);

  const commands = useMemo(() => [
    {
      id: 'compile',
      label: 'Compile code',
      description: 'Run the compiler pipeline',
      shortcut: 'Ctrl+Enter',
      icon: Icon.Play,
      action: compile,
    },
    {
      id: 'format',
      label: 'Format code',
      description: 'Format the active editor content',
      shortcut: 'Ctrl+Shift+F',
      icon: Icon.Format,
      action: () => {},
    },
    {
      id: 'download',
      label: 'Download output',
      description: 'Download generated code as a file',
      icon: Icon.Download,
      action: () => {
        document.dispatchEvent(new CustomEvent('download-output'));
      },
    },
    {
      id: 'shortcuts',
      label: 'Keyboard shortcuts',
      description: 'View all keyboard shortcuts',
      icon: Icon.Keyboard,
      action: () => setShortcutModalOpen(true),
    },
  ], [compile]);

  const onCmdPalette = useCallback(() => setCmdPaletteOpen(true), []);

  useKeyboard([
    { key: 'k', ctrl: true, handler: onCmdPalette },
    { key: 'Enter', ctrl: true, handler: compile },
    { key: 's', ctrl: true, handler: compile },
  ]);

  return (
    <>
      <SplitPanel
        left={<EditorPanel />}
        right={<OutputPanel />}
        defaultRatio={0.5}
        minLeftWidth={320}
        minRightWidth={320}
      />

      <CommandPalette
        open={cmdPaletteOpen}
        onClose={() => setCmdPaletteOpen(false)}
        commands={commands}
      />

      <KeyboardShortcutModal
        open={shortcutModalOpen}
        onClose={() => setShortcutModalOpen(false)}
      />
    </>
  );
}
