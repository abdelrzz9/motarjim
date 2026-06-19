// -- HTML AST Types --

export interface HtmlAttribute {
  name: string;
  value: string;
}

export interface HtmlNode {
  nodeId: string;
  tagName: string;
  attributes: HtmlAttribute[];
  children: HtmlNode[];
  value?: string;
  sourceLocation?: SourceLocation;
}

export interface SourceLocation {
  line: number;
  col: number;
}

// -- CSS Types --

export interface CssStyleDeclaration {
  property: string;
  value: string;
  important: boolean;
}

export interface CssRule {
  selectors: string[];
  declarations: CssStyleDeclaration[];
  sourceLocation?: SourceLocation;
}

export interface CssStylesheet {
  rules: CssRule[];
}

// -- Resolved Styles (HTML node -> computed styles) --

export interface ResolvedStyles {
  [property: string]: string;
}

export interface StyledNode {
  node: HtmlNode;
  styles: ResolvedStyles;
  children: StyledNode[];
}

// -- IR Types --

export type UiNodeType =
  | 'Container'
  | 'Row'
  | 'Column'
  | 'Stack'
  | 'ScrollView'
  | 'LazyList'
  | 'ListView'
  | 'Grid'
  | 'Text'
  | 'Image'
  | 'Icon'
  | 'Button'
  | 'TextField'
  | 'TextArea'
  | 'Form'
  | 'List'
  | 'NavigationBar'
  | 'AppBar'
  | 'Drawer'
  | 'Card'
  | 'HeroSection'
  | 'Footer'
  | 'Sidebar'
  | 'Dialog'
  | 'Modal'
  | 'Tabs'
  | 'Spacer'
  | 'Divider'
  | 'UnorderedList'
  | 'OrderedList'
  | 'ListItem'
  | 'Section'
  | 'Article'
  | 'Header'
  | 'Nav'
  | 'Link'
  | 'Svg'
  | 'Unknown';

export interface UiNode {
  type: UiNodeType;
  properties: Record<string, unknown>;
  children: UiNode[];
  styles?: ResolvedStyles;
  sourceHtmlTag?: string;
  value?: string;
}

// -- Platform target --

export type PlatformTarget = 'flutter' | 'compose' | 'swiftui';

// -- Generator result --

export interface GenerateResult {
  code: string;
  metadata: {
    platform: PlatformTarget;
    nodes: number;
    duration: number;
  };
}

// -- Semantic detection hints --

export interface SemanticHint {
  type: UiNodeType;
  confidence: number;
  node: HtmlNode;
  reason: string;
}

// -- CLI Options --

export interface CliOptions {
  input: string;
  css?: string;
  target: PlatformTarget;
  output?: string;
  watch?: boolean;
  pretty?: boolean;
}
