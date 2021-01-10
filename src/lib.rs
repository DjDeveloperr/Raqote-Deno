use raqote::{DrawTarget, Source, DrawOptions, SolidSource, Color, Image, Path, PathBuilder, Gradient, Spread, Point, GradientStop, StrokeStyle, LineCap, LineJoin};
use std::collections::HashMap;
use deno_core::plugin_api::Interface;
use deno_core::{ZeroCopyBuf, Op};
use std::str::FromStr;
use std::cell::RefCell;
use deno_core::serde::Deserialize;

thread_local! {
    static TARGETS: RefCell<HashMap<u32, DrawTarget>> = RefCell::new(HashMap::new());
}

#[derive(Deserialize)]
enum JsonPathType {
    Move,
    Quad,
    Cubic,
    Arc,
    Rect,
    Line,
    Close
}

#[derive(Deserialize)]
enum JsonLineCap {
    Round,
    Butt,
    Square
}

fn line_cap_from_json(json: JsonLineCap) -> LineCap {
    match json {
        JsonLineCap::Butt => { LineCap::Butt }
        JsonLineCap::Round => { LineCap::Round }
        JsonLineCap::Square => { LineCap::Square }
    }
}

#[derive(Deserialize)]
enum JsonLineJoin {
    Round,
    Miter,
    Bevel
}

fn line_join_from_json(json: JsonLineJoin) -> LineJoin {
    match json {
        JsonLineJoin::Miter => { LineJoin::Miter }
        JsonLineJoin::Round => { LineJoin::Round }
        JsonLineJoin::Bevel => { LineJoin::Bevel }
    }
}

#[derive(Deserialize)]
struct JsonStrokeStyle {
    width: f32,
    cap: JsonLineCap,
    join: JsonLineJoin,
    miter_limit: f32,
    dash_array: Vec<f32>,
    dash_offset: f32
}

fn stroke_style_from_json(json: JsonStrokeStyle) -> StrokeStyle {
    StrokeStyle {
        width: json.width,
        cap: line_cap_from_json(json.cap),
        join: line_join_from_json(json.join),
        miter_limit: json.miter_limit,
        dash_array: json.dash_array,
        dash_offset: json.dash_offset
    }
}

#[derive(Deserialize)]
struct JsonPath {
    path_type: JsonPathType,
    linear: Option<[f32; 2]>,
    quad: Option<[f32; 4]>,
    cubic: Option<[f32; 6]>,
    arc: Option<[f32; 5]>,
}

#[derive(Deserialize)]
struct JsonPathData {
    steps: Vec<JsonPath>
}

#[derive(Deserialize)]
struct JsonColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

fn color_from_json(json: JsonColor) -> Color {
    Color::new(json.a, json.r, json.g, json.b)
}

#[derive(Deserialize)]
struct JsonGradientStop {
    position: f32,
    color: JsonColor
}

#[derive(Deserialize)]
enum JsonSpread {
    Pad,
    Reflect,
    Repeat
}

fn spread_from_json(json: JsonSpread) -> Spread {
    match json {
        JsonSpread::Pad => { Spread::Pad }
        JsonSpread::Reflect => { Spread::Reflect }
        JsonSpread::Repeat => { Spread::Repeat }
    }
}

#[derive(Deserialize)]
struct JsonGradient {
    stops: Vec<JsonGradientStop>,
}

fn gradient_from_json(json: JsonGradient) -> Gradient {
    let mut stops = Vec::<GradientStop>::new();
    for stop in json.stops {
        stops.push(GradientStop { 
            position: stop.position, color: color_from_json(stop.color)
        })
    }
    Gradient { stops }
}

#[derive(Deserialize)]
enum JsonSourceType {
    Solid,
    LinearGradient,
    RadialGradient,
    TwoCircleRadialGradient
}

#[derive(Deserialize)]
struct JsonSource {
    src_type: JsonSourceType,
    color: Option<JsonColor>,
    start: Option<[f32; 2]>,
    end: Option<[f32; 2]>,
    center: Option<[f32; 2]>,
    radius: Option<f32>,
    center2: Option<[f32; 2]>,
    radius2: Option<f32>,
    spread: Option<JsonSpread>,
    gradient: Option<JsonGradient>
}

fn point_from_json(v: [f32; 2]) -> Point {
    Point::new(v[0], v[1])
}

#[no_mangle]
pub fn deno_plugin_init(interface: &mut dyn Interface) {
    interface.register_op("op_new_draw_target", op_new_draw_target);
    interface.register_op("op_dt_get_data", op_dt_get_data);
    interface.register_op("op_dt_write_png", op_dt_write_png);
    interface.register_op("op_dt_fill_rect", op_dt_fill_rect);
    interface.register_op("op_dt_clear", op_dt_clear);
    interface.register_op("op_dt_height", op_dt_height);
    interface.register_op("op_dt_width", op_dt_width);
    interface.register_op("op_dt_fill", op_dt_fill);
    interface.register_op("op_dt_stroke", op_dt_stroke);
    interface.register_op("op_dt_draw_image_at", op_dt_draw_image_at);
    interface.register_op("op_dt_draw_image_with_size_at", op_dt_draw_image_with_size_at);
}

fn get_arg_str(args: &mut [ZeroCopyBuf], idx: usize) -> Result<&str, &str> {
    let res = args.get(idx);
    if res.is_none() {
        Err("not found")
    } else {
        let vec = res.unwrap();
        Ok(std::str::from_utf8(&vec[..]).unwrap())
    }
}

fn get_arg_img(args: &mut [ZeroCopyBuf], idx: usize) -> Result<Box<[u32]>, &str> {
    let res = args.get(idx);
    if res.is_none() {
        Err("not found")
    } else {
        let vec = res.unwrap();
        let mut res = Vec::<u32>::new();
        for e in &vec[..] {
            res.push(*e as u32);
        }
        Ok(res.into_boxed_slice())
    }
}

fn get_arg_path(args: &mut [ZeroCopyBuf], idx: usize) -> Result<Path, &str> {
    let res = get_arg_str(args, idx);
    if res.is_err() {
        Err("not found")
    } else {
        let json_path: JsonPathData = deno_core::serde_json::from_str(res.unwrap()).unwrap();
        let mut pb = PathBuilder::new();
        for step in json_path.steps {
            match step.path_type {
                JsonPathType::Move => {
                    let data = step.linear.unwrap();
                    pb.move_to(data[0], data[1]);
                }
                JsonPathType::Line => {
                    let data = step.linear.unwrap();
                    pb.line_to(data[0], data[1]);
                }
                JsonPathType::Quad => {
                    let data = step.quad.unwrap();
                    pb.quad_to(data[0], data[1], data[2], data[3]);
                }
                JsonPathType::Rect => {
                    let data = step.quad.unwrap();
                    pb.rect(data[0], data[1], data[2], data[3]);
                }
                JsonPathType::Cubic => {
                    let data = step.cubic.unwrap();
                    pb.cubic_to(data[0], data[1], data[2], data[3], data[4], data[5]);
                }
                JsonPathType::Arc => {
                    let data = step.arc.unwrap();
                    pb.arc(data[0], data[1], data[2], data[3], data[4]);
                }
                JsonPathType::Close => {
                    pb.close();
                }
            }
        }
        Ok(pb.finish())
    }
}

fn get_arg_src(args: &mut [ZeroCopyBuf], idx: usize) -> Result<Source, &str> {
    let res = get_arg_str(args, idx);
    if res.is_err() {
        Err("not found")
    } else {
        let val = res.unwrap();
        let json_res = deno_core::serde_json::from_str(val);
        if json_res.is_err() {
            println!("src_json: {}", val);
            eprintln!(" err_msg: {}", json_res.err().unwrap().to_string());
            Err("failed to parse json")
        } else {
            let json: JsonSource = json_res.unwrap();
            match json.src_type {
                JsonSourceType::Solid => {
                    let v = json.color.unwrap();
                    Ok(Source::from(color_from_json(v)))
                }
                JsonSourceType::LinearGradient => {
                    let v = json.gradient.unwrap();
                    let start = point_from_json(json.start.unwrap());
                    let end = point_from_json(json.end.unwrap());
                    let spread = json.spread.unwrap();
                    Ok(Source::new_linear_gradient(gradient_from_json(v), start, end, spread_from_json(spread)))
                }
                JsonSourceType::RadialGradient => {
                    let v = gradient_from_json(json.gradient.unwrap());
                    let center = point_from_json(json.center.unwrap());
                    let radius = json.radius.unwrap();
                    let spread = spread_from_json(json.spread.unwrap());
                    Ok(Source::new_radial_gradient(v, center, radius, spread))
                }
                JsonSourceType::TwoCircleRadialGradient => {
                    let v = gradient_from_json(json.gradient.unwrap());
                    let center = point_from_json(json.center.unwrap());
                    let radius = json.radius.unwrap();
                    let center2 = point_from_json(json.center2.unwrap());
                    let radius2 = json.radius2.unwrap();
                    let spread = spread_from_json(json.spread.unwrap());
                    Ok(Source::new_two_circle_radial_gradient(v, center, radius, center2, radius2, spread))
                }
            }
        }
    }
}

fn get_arg_i32(args: &mut [ZeroCopyBuf], idx: usize) -> Result<i32, &str> {
    let res = get_arg_str(args, idx);
    if res.is_err() {
        Err("not found")
    } else {
        Ok(i32::from_str(res.unwrap()).unwrap())
    }
}

fn get_arg_f32(args: &mut [ZeroCopyBuf], idx: usize) -> Result<f32, &str> {
    let res = get_arg_str(args, idx);
    if res.is_err() {
        Err("not found")
    } else {
        Ok(f32::from_str(res.unwrap()).unwrap())
    }
}

fn get_arg_u32(args: &mut [ZeroCopyBuf], idx: usize) -> Result<u32, &str> {
    let res = get_arg_str(args, idx);
    if res.is_err() {
        Err("not found")
    } else {
        Ok(u32::from_str(res.unwrap()).unwrap())
    }
}

fn get_arg_u8(args: &mut [ZeroCopyBuf], idx: usize) -> Result<u8, &str> {
    let res = get_arg_str(args, idx);
    if res.is_err() {
        Err("not found")
    } else {
        Ok(u8::from_str(res.unwrap()).unwrap())
    }
}

fn op_new_draw_target(
    _interface: &mut dyn Interface, 
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let width = get_arg_i32(_args, 1).unwrap();
    let height = get_arg_i32(_args, 2).unwrap();
    TARGETS.with(|map| {
        let mut targets = map.borrow_mut();
        if targets.contains_key(&id) {
            let res = b"1";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else {
            let dt = DrawTarget::new(width, height);
            targets.insert(id, dt);
            let res = b"0";
            Op::Sync(res.to_vec().into_boxed_slice())   
        }
    })
}

fn op_dt_get_data(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id: u32 = get_arg_u32(_args, 0).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            Op::Sync(target.get_data_u8().to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_width(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id: u32 = get_arg_u32(_args, 0).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            Op::Sync(target.width().to_string().as_bytes().to_vec().into_boxed_slice())
        } else { let res= b"n"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_height(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id: u32 = get_arg_u32(_args, 0).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            Op::Sync(target.height().to_string().as_bytes().to_vec().into_boxed_slice())
        } else { let res= b"n"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_write_png(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id: u32 = get_arg_u32(_args, 0).unwrap();
    let path = get_arg_str(_args, 1).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            let written = target.write_png(path);
            let mut res= b"0";
            if written.is_err() { res = b"1"; }
            Op::Sync(res.to_vec().into_boxed_slice())
        } else {
            let res= b"1";
            Op::Sync(res.to_vec().into_boxed_slice()) 
        }
    })
}

fn op_dt_fill_rect(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let x = get_arg_f32(_args, 1).unwrap();
    let y = get_arg_f32(_args, 2).unwrap();
    let w = get_arg_f32(_args, 3).unwrap();
    let h = get_arg_f32(_args, 4).unwrap();
    let src = get_arg_src(_args, 5).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.fill_rect(x, y, w, h, &src, &DrawOptions::new());
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_clear(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let a = get_arg_u8(_args, 1).unwrap();
    let r = get_arg_u8(_args, 2).unwrap();
    let g = get_arg_u8(_args, 3).unwrap();
    let b = get_arg_u8(_args, 4).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.clear(SolidSource::from(Color::new(a, r, g, b)));
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_fill(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let path = get_arg_path(_args, 1).unwrap();
    let src = get_arg_src(_args, 2).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.fill(&path, &src, &DrawOptions::new());
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_stroke(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let path = get_arg_path(_args, 1).unwrap();
    let stroke = stroke_style_from_json(deno_core::serde_json::from_str(get_arg_str(_args, 3).unwrap()).unwrap());
    let src = get_arg_src(_args, 2).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.stroke(&path, &src, &stroke, &DrawOptions::new());
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_draw_image_at(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let x = get_arg_f32(_args, 2).unwrap();
    let y = get_arg_f32(_args, 3).unwrap();
    let w = get_arg_i32(_args, 4).unwrap();
    let h = get_arg_i32(_args, 5).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.draw_image_at(x, y, &Image {
                width: w,
                height: h,
                data: &*get_arg_img(_args, 1).unwrap()
            }, &DrawOptions::new());
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_draw_image_with_size_at(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let x = get_arg_f32(_args, 2).unwrap();
    let y = get_arg_f32(_args, 3).unwrap();
    let sw = get_arg_f32(_args, 4).unwrap();
    let sh = get_arg_f32(_args, 5).unwrap();
    let w = get_arg_i32(_args, 6).unwrap();
    let h = get_arg_i32(_args, 7).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.draw_image_with_size_at(x, y, sw, sh, &Image {
                width: w,
                height: h,
                data: &*get_arg_img(_args, 1).unwrap()
            }, &DrawOptions::new());
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}