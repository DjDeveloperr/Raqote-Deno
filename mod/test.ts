import { Color, DrawTarget, Gradient, PathBuilder, Point, Source } from "./lib.ts";
import { LineCap, LineJoin, Spread } from "./types.ts";

const dt = new DrawTarget(400, 400);

const gradient = Source.createRadialGradient(
    new Gradient()
        .addStop(0.2, new Color(0xff, 0, 0xff, 0))
        .addStop(0.8, new Color(0xff, 0xff, 0xff, 0xff))
        .addStop(1, new Color(0xff, 0xff, 0, 0xff))
        .stops,
    new Point(150, 150),
    128,
    Spread.Pad
);

dt.fill(
    new PathBuilder()
        .moveTo(100, 10)
        .cubicTo(150, 40, 175, 0, 200, 10)
        .quadTo(120, 100, 80, 200)
        .quadTo(150, 180, 300, 300)
        .close(),
    gradient
);

dt.stroke(
    new PathBuilder()
        .moveTo(100, 100)
        .lineTo(300, 300)
        .lineTo(200, 300),
    {
        cap: LineCap.Round,
        join: LineJoin.Round,
        width: 10,
        miter_limit: 2,
        dash_array: [10, 18],
        dash_offset: 16,
    },
    Source.Solid(new Color(0x80, 0x0, 0x0, 0x80))
);

dt.writePNG("example.png");