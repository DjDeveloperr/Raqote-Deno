import { PathData, ISource, Spread, StrokeStyle } from "./types.ts";

export const pluginID = Deno.openPlugin("../target/debug/raqote_deno.dll");
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
} = (Deno as any).core.ops() as { [name: string]: number };

export const encoder = new TextEncoder();
export const decoder = new TextDecoder('utf-8');

function _fix_src(src: ISource) {
    if (!src.color) src.color = {a:0,r:0,g:0,b:0};
    if (!src.start) src.start = [0,0];
    if (!src.end) src.end = [0,0];
    if (!src.center) src.center = [0,0];
    if (!src.radius) src.radius = 0;
    if (!src.center2) src.center2 = [0,0];
    if (!src.radius2) src.radius2 = 0;
    if (!src.spread) src.spread = Spread.Pad;
    if (!src.gradient) src.gradient = {stops:[]};
    return src;
}

function _fix_path(path: PathData): PathData {
    return { steps: path.steps.map(step => {
        if (!step.linear) step.linear = [0,0];
        if (!step.quad) step.quad = [0,0,0,0];
        if (!step.cubic) step.cubic = [0,0,0,0,0,0];
        if (!step.arc) step.arc = [0,0,0,0,0];
        return step;
    }) }
}

export function dispatch(id: number, ...args: any[]): any {
    return (Deno as any).core.dispatch(
        id, 
        ...args.map(e => typeof e === 'object' && e instanceof Uint8Array ? e :
            encoder.encode(`${typeof e == 'object' ? JSON.stringify(e) : e}`)
        )
    )
}

export function dispatch_data(id: number, ...args: any[]): any {
    return decoder.decode(dispatch(id, ...args));
}

export function new_draw_target(id: number, width: number, height: number): boolean {
    return dispatch_data(op_new_draw_target, id, width, height) == "0";
}

export function dt_get_data(id: number): Uint8Array | void {
    let data = dispatch(op_dt_get_data, id);
    if (data.length == 1 && decoder.decode(data) == '1') return;
    else return data;
}

export function dt_write_png(id: number, path: string): boolean {
    let data = dispatch_data(op_dt_write_png, id, path);
    return data == "0";
}

export function dt_fill_rect(id: number, x: number, y: number, w: number, h: number, src: ISource) {
    let res = dispatch_data(op_dt_fill_rect, id, x, y, w, h, _fix_src(src));
    return res == "0";
}

export function dt_fill(id: number, path: PathData, src: ISource) {
    let res = dispatch_data(op_dt_fill, id, _fix_path(path), _fix_src(src));
    return res == "0";
}

export function dt_stroke(id: number, path: PathData, stroke: StrokeStyle, src: ISource) {
    let res = dispatch_data(op_dt_stroke, id, _fix_path(path), _fix_src(src), stroke);
    return res == "0";
}

export function dt_clear(id: number, a: number, r: number, g: number, b: number) {
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

export function draw_image_at(id: number, img: Uint8Array, x: number, y: number): boolean {
    return dispatch_data(op_dt_draw_image_at, id, img, x, y) == "0";
}

export function draw_image_with_size_at(id: number, img: Uint8Array, x: number, y: number, w: number, h: number): boolean {
    return dispatch_data(op_dt_draw_image_with_size_at, id, img, x, y, w, h) == "0";
}