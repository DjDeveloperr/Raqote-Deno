import { Plug } from "https://deno.land/x/plug@0.2.10/mod.ts";
import { PathData, ISource, Spread, StrokeStyle, BlendMode } from "./types.ts";

const VERSION = "0.0.3";
const POLICY =
  Deno.env.get("PLUGIN_URL") === undefined
    ? Plug.CachePolicy.STORE
    : Plug.CachePolicy.NONE;
const PLUGIN_URL =
  Deno.env.get("PLUGIN_URL") ??
  `https://github.com/DjDeveloperr/Raqote-Deno/releases/download/${VERSION}/`;

await Plug.prepare({
  name: "raqote_deno",
  url: PLUGIN_URL,
  policy: POLICY,
});

export const {
  op_new_draw_target,
  op_dt_get_data,
  op_dt_fill_rect,
  op_dt_fill,
  op_dt_stroke,
  op_dt_write_png,
  op_dt_clear,
  op_dt_height,
  op_dt_width,
  op_dt_encode,
  op_dt_draw_image_at,
  op_dt_draw_image_with_size_at,
  op_dt_destroy,
  op_dt_set_transform,
  op_dt_push_layer,
  op_dt_pop_layer,
  op_dt_push_layer_with_blend,
  op_dt_pop_clip,
  op_dt_push_clip,
  op_dt_push_clip_rect,
} = (Deno as any).core.ops() as { [name: string]: number };

export const encoder = new TextEncoder();
export const decoder = new TextDecoder("utf-8");

function _fix_src(src: ISource) {
  if (!src.color) src.color = { a: 0, r: 0, g: 0, b: 0 };
  if (!src.start) src.start = [0, 0];
  if (!src.end) src.end = [0, 0];
  if (!src.center) src.center = [0, 0];
  if (!src.radius) src.radius = 0;
  if (!src.center2) src.center2 = [0, 0];
  if (!src.radius2) src.radius2 = 0;
  if (!src.spread) src.spread = Spread.Pad;
  if (!src.gradient) src.gradient = { stops: [] };
  return src;
}

function _fix_path(path: PathData): PathData {
  return {
    steps: path.steps.map((step) => {
      if (!step.linear) step.linear = [0, 0];
      if (!step.quad) step.quad = [0, 0, 0, 0];
      if (!step.cubic) step.cubic = [0, 0, 0, 0, 0, 0];
      if (!step.arc) step.arc = [0, 0, 0, 0, 0];
      return step;
    }),
  };
}

export function dispatch(id: number, ...args: any[]): any {
  return (Deno as any).core.dispatch(
    id,
    ...args.map((e) =>
      typeof e === "object" && e instanceof Uint8Array
        ? e
        : encoder.encode(`${typeof e == "object" ? JSON.stringify(e) : e}`)
    )
  );
}

export function dispatch_data(id: number, ...args: any[]): any {
  return decoder.decode(dispatch(id, ...args));
}

export function new_draw_target(
  id: number,
  width: number,
  height: number
): boolean {
  return dispatch_data(op_new_draw_target, id, width, height) == "0";
}

export function dt_get_data(id: number): Uint8Array | void {
  let data = dispatch(op_dt_get_data, id);
  if (data.length == 1 && decoder.decode(data) == "1") return;
  else return data;
}

export function dt_write_png(id: number, path: string): boolean {
  let data = dispatch_data(op_dt_write_png, id, path);
  return data == "0";
}

export function dt_fill_rect(
  id: number,
  x: number,
  y: number,
  w: number,
  h: number,
  src: ISource
) {
  let res = dispatch_data(op_dt_fill_rect, id, x, y, w, h, _fix_src(src));
  return res == "0";
}

export function dt_fill(id: number, path: PathData, src: ISource) {
  let res = dispatch_data(op_dt_fill, id, _fix_path(path), _fix_src(src));
  return res == "0";
}

export function dt_stroke(
  id: number,
  path: PathData,
  stroke: StrokeStyle,
  src: ISource
) {
  let res = dispatch_data(
    op_dt_stroke,
    id,
    _fix_path(path),
    _fix_src(src),
    stroke
  );
  return res == "0";
}

export function dt_clear(
  id: number,
  a: number,
  r: number,
  g: number,
  b: number
) {
  let res = dispatch_data(op_dt_clear, id, a, r, g, b);
  return res == "0";
}

export function dt_destroy(id: number) {
  let res = dispatch_data(op_dt_destroy, id);
  return res == "0";
}

export function dt_height(id: number): void | number {
  let res = dispatch_data(op_dt_height, id);
  if (res == "n") return;
  else return Number(res);
}

export function dt_encode(id: number): void | Uint8Array {
  let res = dispatch(op_dt_encode, id);
  if (res.length == 0 && decoder.decode(res) == "n") return;
  else return res;
}

export function dt_width(id: number): void | number {
  let res = dispatch_data(op_dt_width, id);
  if (res == "n") return;
  else return Number(res);
}

export function draw_image_at(
  id: number,
  img: Uint8Array,
  x: number,
  y: number
): boolean {
  return dispatch_data(op_dt_draw_image_at, id, img, x, y) == "0";
}

export function draw_image_with_size_at(
  id: number,
  img: Uint8Array,
  x: number,
  y: number,
  w: number,
  h: number
): boolean {
  return (
    dispatch_data(op_dt_draw_image_with_size_at, id, img, x, y, w, h) == "0"
  );
}

export function dt_set_transform(
  id: number,
  rc: number,
  m11: number,
  m21: number,
  m31: number,
  m12: number,
  m22: number,
  m32: number
) {
  return (
    dispatch_data(op_dt_set_transform, id, rc, m11, m21, m31, m12, m22, m32) ==
    "0"
  );
}

export function dt_push_layer(id: number, opacity: number) {
  return dispatch_data(op_dt_push_layer, id, opacity) == "0";
}

export function dt_push_layer_with_blend(
  id: number,
  opacity: number,
  blend: BlendMode
) {
  return dispatch_data(op_dt_push_layer_with_blend, id, opacity, blend) == "0";
}

export function dt_push_clip(id: number, path: PathData) {
  return dispatch_data(op_dt_pop_clip, id, _fix_path(path)) == "0";
}

export function dt_push_clip_rect(
  id: number,
  x1: number,
  y1: number,
  x2: number,
  y2: number
) {
  return dispatch_data(op_dt_push_clip_rect, id, x1, y1, x2, y2) == "0";
}

export function dt_pop_clip(id: number) {
  return dispatch_data(op_dt_pop_clip, id) == "0";
}

export function dt_pop_layer(id: number) {
  return dispatch_data(op_dt_pop_layer, id) == "0";
}
