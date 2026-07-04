import {
  createConnection,
  TextDocuments,
  ProposedFeatures,
  InitializeParams,
  InitializeResult,
  CompletionItem,
  CompletionItemKind,
  TextDocumentPositionParams,
  HoverParams,
  Hover,
  Diagnostic,
  DiagnosticSeverity,
  MarkupKind,
} from 'vscode-languageserver/node';
import { TextDocument } from 'vscode-languageserver-textdocument';

const connection = createConnection(ProposedFeatures.all);
const documents = new TextDocuments(TextDocument);

connection.onInitialize((_params: InitializeParams): InitializeResult => {
  return {
    capabilities: {
      textDocumentSync: {
        openClose: true,
        change: 1,
      },
      completionProvider: {
        resolveProvider: true,
        triggerCharacters: ['<', ' ', '.', '/'],
      },
      hoverProvider: true,
      diagnosticProvider: {
        interFileDependencies: false,
        workspaceDiagnostics: false,
      },
    },
  };
});

connection.onInitialized(() => {
  connection.console.log('Motarjim LSP server initialized');
});

function getHTMLCompletions(docText: string, line: number, character: number): CompletionItem[] {
  const lines = docText.split('\n');
  const currentLine = lines[line] || '';
  const beforeCursor = currentLine.slice(0, character);

  const completions: CompletionItem[] = [];

  if (beforeCursor.endsWith('<')) {
    completions.push(
      {
        label: 'html',
        kind: CompletionItemKind.Keyword,
        detail: 'HTML root element',
        insertText: 'html>\n  <head>\n    <title></title>\n  </head>\n  <body>\n    \n  </body>\n</html>',
      },
      {
        label: 'div',
        kind: CompletionItemKind.Keyword,
        detail: 'Container element',
        insertText: 'div>\n  \n</div>',
      },
      {
        label: 'text',
        kind: CompletionItemKind.Keyword,
        detail: 'Text element',
        insertText: 'text>\n  \n</text>',
      },
      {
        label: 'image',
        kind: CompletionItemKind.Keyword,
        detail: 'Image element',
        insertText: 'image src="" />',
      },
      {
        label: 'button',
        kind: CompletionItemKind.Keyword,
        detail: 'Button element',
        insertText: 'button>\n  \n</button>',
      },
      {
        label: 'input',
        kind: CompletionItemKind.Keyword,
        detail: 'Input element',
        insertText: 'input type="" />',
      },
      {
        label: 'column',
        kind: CompletionItemKind.Keyword,
        detail: 'Column layout element',
        insertText: 'column>\n  \n</column>',
      },
      {
        label: 'row',
        kind: CompletionItemKind.Keyword,
        detail: 'Row layout element',
        insertText: 'row>\n  \n</row>',
      },
      {
        label: 'list',
        kind: CompletionItemKind.Keyword,
        detail: 'List element',
        insertText: 'list>\n  \n</list>',
      },
      {
        label: 'scaffold',
        kind: CompletionItemKind.Keyword,
        detail: 'Scaffold layout element',
        insertText: 'scaffold>\n  \n</scaffold>',
      }
    );
  }

  if (beforeCursor.includes(' ')) {
    const tagMatch = beforeCursor.match(/<(\w+)[^>]*$/);
    if (tagMatch) {
      completions.push(
        { label: 'id', kind: CompletionItemKind.Property, detail: 'Element identifier', insertText: 'id=""' },
        { label: 'class', kind: CompletionItemKind.Property, detail: 'CSS class name', insertText: 'class=""' },
        { label: 'style', kind: CompletionItemKind.Property, detail: 'Inline styles', insertText: 'style=""' },
        { label: 'src', kind: CompletionItemKind.Property, detail: 'Source URL', insertText: 'src=""' },
        { label: 'href', kind: CompletionItemKind.Property, detail: 'Hyperlink reference', insertText: 'href=""' },
        { label: 'type', kind: CompletionItemKind.Property, detail: 'Input type', insertText: 'type=""' },
        { label: 'value', kind: CompletionItemKind.Property, detail: 'Element value', insertText: 'value=""' },
        { label: 'placeholder', kind: CompletionItemKind.Property, detail: 'Placeholder text', insertText: 'placeholder=""' },
        { label: 'disabled', kind: CompletionItemKind.Property, detail: 'Disabled state', insertText: 'disabled' },
        { label: 'hidden', kind: CompletionItemKind.Property, detail: 'Visibility control', insertText: 'hidden' }
      );
    }
  }

  return completions;
}

function getCSSCompletions(docText: string, line: number, character: number): CompletionItem[] {
  const lines = docText.split('\n');
  const currentLine = lines[line] || '';
  const beforeCursor = currentLine.slice(0, character);

  const completions: CompletionItem[] = [];

  if (beforeCursor.endsWith(':') || beforeCursor.match(/:\s*$/)) {
    completions.push(
      { label: 'red', kind: CompletionItemKind.Color, detail: 'Color value' },
      { label: 'blue', kind: CompletionItemKind.Color, detail: 'Color value' },
      { label: 'green', kind: CompletionItemKind.Color, detail: 'Color value' },
      { label: 'white', kind: CompletionItemKind.Color, detail: 'Color value' },
      { label: 'black', kind: CompletionItemKind.Color, detail: 'Color value' },
      { label: 'transparent', kind: CompletionItemKind.Color, detail: 'Color value' },
      { label: 'flex-start', kind: CompletionItemKind.Value, detail: 'Alignment value' },
      { label: 'flex-end', kind: CompletionItemKind.Value, detail: 'Alignment value' },
      { label: 'center', kind: CompletionItemKind.Value, detail: 'Alignment value' },
      { label: 'stretch', kind: CompletionItemKind.Value, detail: 'Alignment value' }
    );
  }

  if (beforeCursor.match(/^[a-zA-Z]/) || beforeCursor.match(/^\s*[a-zA-Z]/)) {
    completions.push(
      { label: 'color', kind: CompletionItemKind.Property, detail: 'Text color', insertText: 'color: ;' },
      { label: 'background-color', kind: CompletionItemKind.Property, detail: 'Background color', insertText: 'background-color: ;' },
      { label: 'font-size', kind: CompletionItemKind.Property, detail: 'Font size', insertText: 'font-size: ;' },
      { label: 'font-family', kind: CompletionItemKind.Property, detail: 'Font family', insertText: 'font-family: ;' },
      { label: 'margin', kind: CompletionItemKind.Property, detail: 'Margin', insertText: 'margin: ;' },
      { label: 'padding', kind: CompletionItemKind.Property, detail: 'Padding', insertText: 'padding: ;' },
      { label: 'width', kind: CompletionItemKind.Property, detail: 'Element width', insertText: 'width: ;' },
      { label: 'height', kind: CompletionItemKind.Property, detail: 'Element height', insertText: 'height: ;' },
      { label: 'display', kind: CompletionItemKind.Property, detail: 'Display mode', insertText: 'display: ;' },
      { label: 'flex-direction', kind: CompletionItemKind.Property, detail: 'Flex direction', insertText: 'flex-direction: ;' },
      { label: 'justify-content', kind: CompletionItemKind.Property, detail: 'Justify content', insertText: 'justify-content: ;' },
      { label: 'align-items', kind: CompletionItemKind.Property, detail: 'Align items', insertText: 'align-items: ;' },
      { label: 'gap', kind: CompletionItemKind.Property, detail: 'Gap between items', insertText: 'gap: ;' },
      { label: 'border-radius', kind: CompletionItemKind.Property, detail: 'Border radius', insertText: 'border-radius: ;' },
      { label: 'box-shadow', kind: CompletionItemKind.Property, detail: 'Box shadow', insertText: 'box-shadow: ;' }
    );
  }

  return completions;
}

function getHoverInfo(word: string): string | null {
  const elementDocs: Record<string, string> = {
    html: 'Root element of a Motarjim document. Contains `<head>` and `<body>` sections.',
    div: 'A generic container element. Renders as a layout block.',
    text: 'Displays a text string. Supports inline formatting.',
    image: 'Displays an image. Requires `src` attribute with a valid URL.',
    button: 'A clickable button element. Can contain text or child elements.',
    input: 'A form input field. Use `type` attribute to specify input type.',
    column: 'A flex column layout container. Children are arranged vertically.',
    row: 'A flex row layout container. Children are arranged horizontally.',
    list: 'Renders a scrollable list of items.',
    scaffold: 'A top-level layout structure providing app bar, body, and fab slots.',
    head: 'Document metadata container. Not rendered visually.',
    body: 'Document body container. Contains all visible content.',
    title: 'Sets the document title (used in head section).',
  };

  const cssDocs: Record<string, string> = {
    color: 'Sets the text color. Accepts named colors, hex, rgb, or hsl values.',
    'background-color': 'Sets the background color of an element.',
    'font-size': 'Sets the size of the font. Accepts px, em, rem, or percentage values.',
    'font-family': 'Specifies the font family for text.',
    margin: 'Sets the outer spacing of an element. Accepts 1-4 values (top, right, bottom, left).',
    padding: 'Sets the inner spacing of an element. Accepts 1-4 values.',
    width: 'Sets the width of an element.',
    height: 'Sets the height of an element.',
    display: 'Controls the display behavior of an element. Common values: flex, block, inline, none.',
    'flex-direction': 'Defines the direction of flex items. Values: row, column, row-reverse, column-reverse.',
    'justify-content': 'Aligns flex items along the main axis. Values: flex-start, flex-end, center, space-between, space-around.',
    'align-items': 'Aligns flex items along the cross axis. Values: flex-start, flex-end, center, stretch, baseline.',
    gap: 'Sets the spacing between flex items or grid cells.',
    'border-radius': 'Rounds the corners of an element\'s border.',
    'box-shadow': 'Adds shadow effects around an element\'s frame.',
  };

  return elementDocs[word] || cssDocs[word] || null;
}

function validateDocument(text: string): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];
  const lines = text.split('\n');
  const openTags: { tag: string; line: number }[] = [];
  const voidElements = new Set(['image', 'input', 'br', 'hr', 'meta', 'link']);

  for (let i = 0; i < lines.length; i++) {
    const lineText = lines[i];

    const tagRegex = /<\/?(\w+)[^>]*>/g;
    let match: RegExpExecArray | null;

    while ((match = tagRegex.exec(lineText)) !== null) {
      const fullMatch = match[0];
      const tagName = match[1];
      const isClosingTag = fullMatch.startsWith('</');
      const isSelfClosing = fullMatch.endsWith('/>') || voidElements.has(tagName);

      if (isClosingTag) {
        if (openTags.length > 0 && openTags[openTags.length - 1].tag === tagName) {
          openTags.pop();
        } else {
          diagnostics.push({
            severity: DiagnosticSeverity.Error,
            range: {
              start: { line: i, character: match.index },
              end: { line: i, character: match.index + fullMatch.length },
            },
            message: `Unexpected closing tag </${tagName}>. Expected closing tag for "${openTags.length > 0 ? openTags[openTags.length - 1].tag : 'nothing'}".`,
            source: 'motarjim',
          });
        }
      } else if (!isSelfClosing) {
        openTags.push({ tag: tagName, line: i });
      }
    }
  }

  for (const openTag of openTags) {
    diagnostics.push({
      severity: DiagnosticSeverity.Error,
      range: {
        start: { line: openTag.line, character: 0 },
        end: { line: openTag.line, character: lines[openTag.line].length },
      },
      message: `Tag <${openTag.tag}> was never closed.`,
      source: 'motarjim',
    });
  }

  const knownElements = new Set([
    'html', 'head', 'body', 'title', 'div', 'text', 'image', 'button',
    'input', 'column', 'row', 'list', 'scaffold', 'style', 'link', 'meta',
    'br', 'hr', 'span', 'header', 'footer', 'section', 'nav', 'main',
    'aside', 'article', 'form', 'label', 'select', 'option', 'textarea',
  ]);

  const elementRegex = /<(\w+)[^>]*>/g;
  for (let i = 0; i < lines.length; i++) {
    const lineText = lines[i];
    let match: RegExpExecArray | null;

    while ((match = elementRegex.exec(lineText)) !== null) {
      const tagName = match[1];
      if (!knownElements.has(tagName) && tagName !== 'style') {
        diagnostics.push({
          severity: DiagnosticSeverity.Warning,
          range: {
            start: { line: i, character: match.index },
            end: { line: i, character: match.index + match[0].length },
          },
          message: `Unknown element <${tagName}>. This may not be supported by all platforms.`,
          source: 'motarjim',
        });
      }
    }
  }

  return diagnostics;
}

documents.onDidChangeContent((change) => {
  const text = change.document.getText();
  const diagnostics = validateDocument(text);
  connection.sendDiagnostics({ uri: change.document.uri, diagnostics });
});

connection.onCompletion(
  (params: TextDocumentPositionParams): CompletionItem[] => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return [];

    const languageId = document.languageId;
    const text = document.getText();
    const { line, character } = params.position;

    if (languageId === 'css' || languageId === 'motarjim-css') {
      return getCSSCompletions(text, line, character);
    }

    return getHTMLCompletions(text, line, character);
  }
);

connection.onCompletionResolve(
  (item: CompletionItem): CompletionItem => {
    if (!item.data) return item;

    item.detail = `Motarjim: ${item.detail || item.label}`;
    return item;
  }
);

connection.onHover((params: HoverParams): Hover | null => {
  const document = documents.get(params.textDocument.uri);
  if (!document) return null;

  const text = document.getText();
  const lines = text.split('\n');
  const { line, character } = params.position;

  if (line < 0 || line >= lines.length) return null;

  const currentLine = lines[line];
  const wordRegex = /[\w-]+/g;
  let match: RegExpExecArray | null;

  while ((match = wordRegex.exec(currentLine)) !== null) {
    const start = match.index;
    const end = match.index + match[0].length;

    if (character >= start && character <= end) {
      const word = match[0];
      const info = getHoverInfo(word);

      if (info) {
        return {
          contents: {
            kind: MarkupKind.Markdown,
            value: `**${word}**\n\n${info}`,
          },
          range: {
            start: { line, character: start },
            end: { line, character: end },
          },
        };
      }
    }
  }

  return null;
});

documents.listen(connection);
connection.listen();
