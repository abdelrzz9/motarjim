import type { SVGProps } from 'react';

type IconProps = SVGProps<SVGSVGElement> & { size?: number };

const S = { w: 1.5, c: 'round' as const };

export const Icon = {
  Logo: ({ size = 22, ...props }: IconProps) => (
    <svg viewBox="0 0 22 22" fill="none" width={size} height={size} {...props}>
      <rect x="1" y="1" width="20" height="20" rx="4" fill="currentColor" />
      <path d="M7 15V7l4 4 4-4v8" stroke="white" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  ),
  Compile: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M3 1.5l8 5.5-8 5.5V1.5z" fill="currentColor" />
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
  Code: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M6 4L2 8l4 4M10 4l4 4-4 4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
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
  Search: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="7" cy="7" r="4.5" stroke="currentColor" strokeWidth={S.w} />
      <path d="M10.5 10.5L14 14" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),
  Node: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="4" r="2" stroke="currentColor" strokeWidth={S.w} />
      <circle cx="4" cy="12" r="2" stroke="currentColor" strokeWidth={S.w} />
      <circle cx="12" cy="12" r="2" stroke="currentColor" strokeWidth={S.w} />
      <path d="M6 5.5L5 10.5M10 5.5l1 5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),
  Zap: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M8 1L3 9h5l-1 6 6-8H8z" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),
  Play: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M5 3l8 5-8 5V3z" fill="currentColor" />
    </svg>
  ),
  Check: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M3 8l3 3 7-7" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  ),
  Copy: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <rect x="5.5" y="5.5" width="8" height="8" rx="1.5" stroke="currentColor" strokeWidth={S.w} />
      <path d="M10.5 5.5V4a1.5 1.5 0 00-1.5-1.5H4A1.5 1.5 0 002.5 4v5A1.5 1.5 0 004 10.5h1.5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),
  Download: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M8 2v9M4 7l4 4 4-4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
      <path d="M2 12v1.5A1.5 1.5 0 003.5 15h9a1.5 1.5 0 001.5-1.5V12" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
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
  Sample: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M2 4h12M5 4V3a1 1 0 011-1h4a1 1 0 011 1v1" stroke="currentColor" strokeWidth={S.w} />
      <path d="M3.5 6l.7 7.2a1 1 0 001 .8h5.6a1 1 0 001-.8l.7-7.2" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
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
  Expand: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M4 6l4 4 4-4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),
  Collapse: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M4 10l4-4 4 4" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
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
  Playground: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M6 2L2 8l4 6M10 2l4 6-4 6" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),
  ArrowRight: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <path d="M5 3l5 5-5 5" stroke="currentColor" strokeWidth={S.w} strokeLinecap={S.c} />
    </svg>
  ),
  Dot: ({ size = 16, ...props }: IconProps) => (
    <svg viewBox="0 0 16 16" fill="none" width={size} height={size} {...props}>
      <circle cx="8" cy="8" r="3" fill="currentColor" />
    </svg>
  ),
};
