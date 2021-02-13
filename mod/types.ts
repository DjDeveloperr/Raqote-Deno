export type PathType =
  | "Move"
  | "Quad"
  | "Cubic"
  | "Arc"
  | "Rect"
  | "Line"
  | "Close";

export type SourceType =
  | "Solid"
  | "LinearGradient"
  | "RadialGradient"
  | "TwoCircleRadialGradient";

export enum Spread {
  Pad = "Pad",
  Reflect = "Reflect",
  Repeat = "Repeat",
}

export enum LineCap {
  Round = "Round",
  Butt = "Butt",
  Square = "Square",
}

export enum LineJoin {
  Round = "Round",
  Miter = "Miter",
  Bevel = "Bevel",
}

export interface Color {
  r: number;
  g: number;
  b: number;
  a: number;
}

export interface GradientStop {
  position: number;
  color: Color;
}

export interface Gradient {
  stops: GradientStop[];
}

export interface ISource {
  src_type: SourceType;
  color?: Color | null;
  start?: number[] | null;
  end?: number[] | null;
  center?: number[] | null;
  radius?: number | null;
  center2?: number[] | null;
  radius2?: number | null;
  spread?: Spread | null;
  gradient?: Gradient | null;
}

export interface Path {
  path_type: PathType;
  linear?: number[] | null;
  quad?: number[] | null;
  cubic?: number[] | null;
  arc?: number[] | null;
}

export interface PathData {
  steps: Path[];
}

export interface StrokeStyle {
  width: number;
  cap: LineCap;
  join: LineJoin;
  miter_limit: number;
  dash_array: number[];
  dash_offset: number;
}
