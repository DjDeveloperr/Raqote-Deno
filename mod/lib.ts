import { draw_image_at, draw_image_with_size_at, dt_destroy, dt_encode, dt_fill, dt_fill_rect, dt_get_data, dt_stroke, dt_write_png, new_draw_target } from "./ops.ts";
import { ISource, PathData, StrokeStyle, Path, GradientStop, Spread } from "./types.ts";

const DRAW_TARGETS = new Set<number>();

function getNewID() {
    let id = 0;
    while (DRAW_TARGETS.has(id)) {
        id++;
    }
    return id;
}

export class DrawTarget {
    id: number
    height: number
    width: number

    constructor(width: number, height: number) {
        this.id = getNewID();
        this.width = width;
        this.height = height;
        const created = new_draw_target(this.id, this.width, this.height);
        if (!created) throw new Error("Failed to create DrawTarget");
        DRAW_TARGETS.add(this.id);
    }

    getData(): Uint8Array {
        return dt_get_data(this.id) as Uint8Array;
    }

    fillRect(x: number, y: number, w: number, h: number, src: ISource): DrawTarget {
        if(!dt_fill_rect(this.id, x, y, w, h, src)) throw new Error("Failed to fillRect");
        return this;
    }

    fill(path: PathData | PathBuilder, src: ISource): DrawTarget {
        if (path instanceof PathBuilder) path = path.finish();
        if(!dt_fill(this.id, path, src)) throw new Error("Failed to fill");
        return this;
    }

    stroke(path: PathData | PathBuilder, stroke: StrokeStyle, src: ISource): DrawTarget {
        if (path instanceof PathBuilder) path = path.finish();
        if(!dt_stroke(this.id, path, stroke, src)) throw new Error("Failed to stroke");
        return this;
    }

    drawImageAt(x: number, y: number, img: Image | Uint8Array): DrawTarget {
        if(!draw_image_at(this.id, img instanceof Uint8Array ? img : img.data, x, y)) throw new Error("Failed to drawImageAt");
        return this;
    }

    drawImageWithSizeAt(x: number, y: number, w: number, h: number, img: Image | Uint8Array): DrawTarget {
        if(!draw_image_with_size_at(this.id, img instanceof Uint8Array ? img : img.data, w, h, x, y)) throw new Error("Failed to drawImageWithSizeAt");
        return this;
    }

    writePNG(path: string): DrawTarget {
        if(!dt_write_png(this.id, path)) throw new Error("Failed to writePNG");
        return this;
    }

    encodePNG(): Uint8Array {
        const res = dt_encode(this.id);
        if(!res) throw new Error("Failed to encodePNG");
        return res;
    }

    destroy(): boolean {
        const done = dt_destroy(this.id);
        if (done) DRAW_TARGETS.delete(this.id);
        return done;
    }
}

export class Image {
    data: Uint8Array

    constructor(data: Uint8Array) {
        this.data = data;
    }

    static open(path: string) {
        return new Image(Deno.readFileSync(path));
    }
}

export class PathBuilder {
    steps: Path[] = []

    moveTo(x: number, y: number) {
        this.steps.push({
            path_type: "Move",
            linear: [x, y]
        });
        return this;
    }

    lineTo(x: number, y: number) {
        this.steps.push({
            path_type: "Line",
            linear: [x, y]
        });
        return this;
    }

    quadTo(cx: number, cy: number, x: number, y: number) {
        this.steps.push({
            path_type: "Quad",
            quad: [cx, cy, x, y]
        });
        return this;
    }

    rect(x: number, y: number, w: number, h: number) {
        this.steps.push({
            path_type: "Rect",
            quad: [x, y, w, h]
        });
        return this;
    }

    cubicTo(cx: number, cy: number, cx2: number, cy2: number, x: number, y: number) {
        this.steps.push({
            path_type: "Cubic",
            cubic: [cx, cy, cx2, cy2, x, y]
        });
        return this;
    }

    arc(x: number, y: number, r: number, startAngle: number, sweepAngle: number) {
        this.steps.push({
            path_type: "Arc",
            arc: [x, y, r, startAngle, sweepAngle]
        });
        return this;
    }

    close() {
        this.steps.push({
            path_type: "Close"
        });
        return this;
    }

    finish(): PathData {
        return { steps: this.steps };
    }
}

export class Point {
    x: number = 0;
    y: number = 0;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }

    toArray() {
        return [this.x, this.y];
    }
}

export class Source {
    static Solid(color: Color): ISource {
        return {
            src_type: "Solid",
            color
        }
    }

    static createRadialGradient(stops: GradientStop[], center: Point, radius: number, spread: Spread): ISource {
        return {
            src_type: "RadialGradient",
            gradient: {
                stops
            },
            center: center.toArray(),
            radius,
            spread
        }
    }
}

export class Color {
    constructor(public a: number, public r: number, public g: number, public b: number) {}
}

export class Gradient {
    stops: GradientStop[] = []

    addStop(position: number, color: Color) {
        this.stops.push({ position, color });
        return this;
    }
}