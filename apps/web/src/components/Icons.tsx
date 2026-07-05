import type { SVGProps } from 'react';

type IconProps = SVGProps<SVGSVGElement> & { size?: number };
const S = { w: 1.5, c: 'round' as const };

export const Icon = {
  Logo: ({ size = 20, ...props }: IconProps) => (
    <svg viewBox="0 0 20 20" fill="none" width={size} height={size} {...props}>
      <rect x="0.5" y="0.5" width="19" height="19" rx="4" fill="var(--accent)" />
      <path d="M6.5 14V6l4 4 4-4v8" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  ),

  Compile: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M2.5 2.5l11 5.5-11 5.5V2.5z" fill="currentColor" />
    </svg>
  ),

  Play: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M4 2.5l10 5.5L4 13.5V2.5z" fill="currentColor" />
    </svg>
  ),

  Stop: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <rect x="3" y="3" width="10" height="10" rx="2" fill="currentColor" />
    </svg>
  ),

  Check: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M3 8l3.5 3.5L13 4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  ),

  Copy: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <rect x="5.5" y="5.5" width="8" height="8" rx="1.5" stroke="currentColor" strokeWidth={S.w} />
      <path d="M10.5 5.5V4a1.5 1.5 0 00-1.5-1.5H4A1.5 1.5 0 002.5 4v5A1.5 1.5 0 004 10.5h1.5" stroke="currentColor" strokeWidth={S.w} />
    </svg>
  ),

  Download: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M8 2v8.5M4.5 7L8 10.5 11.5 7" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
      <path d="M2 12v1a1 1 0 001 1h10a1 1 0 001-1v-1" stroke="currentColor" strokeWidth={S.w} />
    </svg>
  ),

  Format: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M3 4h10M3 8h6M3 12h8" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Fullscreen: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M2 6V2h4M14 6V2h-4M2 10v4h4M14 10v4h-4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Paste: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M6 3.5A1.5 1.5 0 017.5 2h1A1.5 1.5 0 0110 3.5v.5H6v-.5z" stroke="currentColor" strokeWidth={S.w} />
      <rect x="2.5" y="4" width="11" height="10" rx="1.5" stroke="currentColor" strokeWidth={S.w} />
      <path d="M5 8h6M5 11h4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Upload: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M8 2v8M4 6l4-4 4 4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
      <path d="M2 11v1.5A1.5 1.5 0 003.5 14h9a1.5 1.5 0 001.5-1.5V11" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Clear: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Folder: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M1.5 4.5V12a1.5 1.5 0 001.5 1.5h10A1.5 1.5 0 0014.5 12V6a1.5 1.5 0 00-1.5-1.5H8l-1.5-2H3a1.5 1.5 0 00-1.5 1.5z" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Code: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M6 4L2 8l4 4M10 4l4 4-4 4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Settings: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="2.5" stroke="currentColor" strokeWidth={S.w} />
      <path d="M8 1v2M8 13v2M1 8h2M13 8h2M2.5 2.5l1.5 1.5M12 12l1.5 1.5M2.5 13.5l1.5-1.5M12 4l1.5-1.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Theme: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="3" stroke="currentColor" strokeWidth={S.w} />
      <path d="M8 0v2M8 14v2M0 8h2M14 8h2" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Sun: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="3" fill="currentColor" />
      <path d="M8 1v1M8 14v1M1 8h1M14 8h1M3.05 3.05l.7.7M12.25 12.25l.7.7M3.05 12.95l.7-.7M12.25 3.75l.7-.7" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Moon: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M13 10.3A6 6 0 015.7 3 6 6 0 1013 10.3z" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Search: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="7" cy="7" r="4.5" stroke="currentColor" strokeWidth={S.w} />
      <path d="M10.5 10.5L14 14" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Error: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth={S.w} />
      <path d="M8 5v3M8 10.5v.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Warning: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M8 2L1 14h14L8 2z" stroke="currentColor" strokeWidth={S.w} />
      <path d="M8 6v3M8 10.5v.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Info: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth={S.w} />
      <path d="M8 7v4M8 5v.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Close: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  ChevronDown: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M4 6l4 4 4-4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  ChevronRight: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M6 4l4 4-4 4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Keyboard: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <rect x="1.5" y="4" width="13" height="8" rx="1.5" stroke="currentColor" strokeWidth={S.w} />
      <path d="M4 7h1M7.5 7h1M11 7h1M4 9.5h8" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Sparkles: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M8 1l1.5 3.5L13 6l-3.5 1.5L8 11l-1.5-3.5L3 6l3.5-1.5L8 1z" stroke="currentColor" strokeWidth="1.2" />
      <path d="M12 10l.8 1.2L14 12l-1.2.8L12 14l-.8-1.2L10 12l1.2-.8L12 10z" stroke="currentColor" strokeWidth="1.2" strokeLinecap={S.c} />
    </svg>
  ),

  Python: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <rect x="3" y="1" width="10" height="14" rx="2" stroke="currentColor" strokeWidth={S.w} />
      <path d="M6 5h4M6 8h4M6 11h2" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Palette: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth={S.w} />
      <circle cx="6" cy="6" r="1" fill="currentColor" />
      <circle cx="10" cy="6" r="1" fill="currentColor" />
      <circle cx="8" cy="10" r="1" fill="currentColor" />
    </svg>
  ),

  External: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M12 8.5V12a1.5 1.5 0 01-1.5 1.5h-7A1.5 1.5 0 012 12V5a1.5 1.5 0 011.5-1.5h3.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
      <path d="M9 2h5v5M8 8l6-6" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Html: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M3 2l1 12 4 1.5 4-1.5L13 2H3z" stroke="currentColor" strokeWidth={S.w} strokeLinejoin="round" />
      <path d="M6 6l-1 2.5 1 2M10 6l1 2.5-1 2" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Css: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M2 2l1.2 12L8 15l4.8-1L14 2H2z" stroke="currentColor" strokeWidth={S.w} strokeLinejoin="round" />
      <path d="M5 5h6l-.5 4.5L8 11l-2.5-1.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} strokeLinejoin="round" />
    </svg>
  ),

  Flutter: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 24 24" fill="none" width={size} height={size} {...props}>
      <polygon points="14.314,0 2.3,12 6,15.7 21.684,0" fill="#47C5FB" />
      <polygon points="14.328,11.072 9.028,16.372 14.328,21.686 21.698,21.686 16.398,16.372 21.698,11.072" fill="#47C5FB" />
      <polygon points="9.028,16.372 11.128,18.472 14.328,21.686" fill="#00569E" />
      <polygon points="9.028,16.372 11.128,18.472 6,13.344" fill="#00B5F8" />
    </svg>
  ),

  Kotlin: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 24 24" fill="none" width={size} height={size} {...props}>
      <path d="M12 2L3 12v7l9-9 9-8z" fill="url(#kotlin-a)" />
      <path d="M3 19l9-9 9 8.9z" fill="url(#kotlin-b)" />
      <path d="M3 2h9L3 12z" fill="url(#kotlin-c)" />
      <defs>
        <linearGradient id="kotlin-a" x1="2.039" y1="11.659" x2="9.95" y2="3.748" gradientUnits="userSpaceOnUse">
          <stop offset=".108" stopColor="#c757bc"/>
          <stop offset=".173" stopColor="#cd5ca9"/>
          <stop offset=".492" stopColor="#e8744f"/>
          <stop offset=".716" stopColor="#f88316"/>
          <stop offset=".823" stopColor="#ff8900"/>
        </linearGradient>
        <linearGradient id="kotlin-b" gradientUnits="userSpaceOnUse">
          <stop offset=".296" stopColor="#00afff"/>
          <stop offset=".694" stopColor="#5282ff"/>
          <stop offset="1" stopColor="#945dff"/>
        </linearGradient>
        <linearGradient id="kotlin-c" x1="3.369" y1="6.189" x2="6.073" y2="3.484" href="#kotlin-b"/>
      </defs>
    </svg>
  ),

  Swift: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 24 24" fill="none" width={size} height={size} {...props}>
      <path d="M21.73 6.42a6.72 6.72 0 00-.15-1.42c-.19-.96-.69-1.84-1.43-2.52a4.53 4.53 0 00-1.3-.85c-.47-.22-.97-.37-1.48-.46a10.4 10.4 0 00-1.58-.14H4.92a9.27 9.27 0 00-1.17.1c-.39.04-.77.13-1.13.27l-.36.14a6.56 6.56 0 00-.97.6c-.1.09-.2.14-.3.22a4.1 4.1 0 00-.94 1.15c-.25.42-.47.87-.56 1.37A13.43 13.43 0 000 6.5v10.96c0 .48.02.96.15 1.42.19.96.69 1.84 1.43 2.52.4.35.84.64 1.3.85.47.22.97.37 1.48.46.51.1 1.07.1 1.58.14h12.2c.5 0 1.05-.04 1.58-.14.5-.08 1-.23 1.45-.45.46-.24.88-.54 1.27-.88.38-.35.7-.75.96-1.19.25-.43.42-.89.5-1.38.06-.47.1-.95.15-1.42V6.5z" fill="#F05138"/>
      <path d="M16.06 17.94c-2.14 1.11-5.08 1.22-8.04.09-2.3-1.01-4.28-2.67-5.9-4.72.65.46 1.35.84 2.1 1.13 3.05 1.26 6.11 1.2 8.26 0-2.83-2.05-5.38-4.43-7.6-7.09a7.96 7.96 0 01-1.02-1.25c2.32 1.85 4.79 3.53 7.39 5.02-1.89-1.82-3.63-3.78-5.2-5.86 1.79 1.6 3.62 3.05 5.57 4.4l.33.18c.1-.2.17-.41.23-.63.72-2.34-.1-5-.89-7.2 4.13 2.25 6.58 6.46 5.56 10-.01.09-.05.19-.09.28l.03.04c2.05 2.3 1.48 4.72 1.23 4.26-1.12-1.91-3.19-1.33-4.22-.93z" fill="#fff"/></svg>
  ),

  DragHandle: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 4 16" fill="none" width={size} height={size} {...props}>
      <circle cx="2" cy="3" r="1" fill="currentColor" />
      <circle cx="2" cy="8" r="1" fill="currentColor" />
      <circle cx="2" cy="13" r="1" fill="currentColor" />
    </svg>
  ),

  Clock: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth={S.w} />
      <path d="M8 5v3l2 1.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  File: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M10 1.5H4.5A1.5 1.5 0 003 3v10a1.5 1.5 0 001.5 1.5h7A1.5 1.5 0 0013 13V5l-3-3.5z" stroke="currentColor" strokeWidth={S.w} strokeLinejoin="round" />
      <path d="M10 1.5V5h3.5" stroke="currentColor" strokeWidth={S.w} strokeLinejoin="round" />
    </svg>
  ),

  Layers: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M8 2L1 6l7 4 7-4-7-4z" stroke="currentColor" strokeWidth={S.w} strokeLinejoin="round" />
      <path d="M1 10l7 4 7-4" stroke="currentColor" strokeWidth={S.w} strokeLinejoin="round" />
    </svg>
  ),

  Command: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M4.5 4.5v7M11.5 4.5v7M4.5 4.5h7M4.5 11.5h7" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
      <path d="M4.5 4.5A1.5 1.5 0 113 6h1.5v-1.5zM4.5 11.5A1.5 1.5 0 113 10h1.5v1.5zM11.5 4.5A1.5 1.5 0 1113 6h-1.5V4.5zM11.5 11.5A1.5 1.5 0 1113 10h-1.5v1.5z" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Zap: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M8 1L3 9h5l-1 6 6-8H8z" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),

  Bug: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="5" stroke="currentColor" strokeWidth={S.w} />
      <path d="M8 6v3M8 10.5v.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),
};
