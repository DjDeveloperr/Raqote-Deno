use raqote::{DrawTarget, Source, DrawOptions, SolidSource, Color, Image, Path, PathBuilder, Gradient, Spread, Point, GradientStop, StrokeStyle, LineCap, LineJoin, Transform, IntRect, BlendMode};
use std::collections::HashMap;
use deno_core::plugin_api::Interface;
use deno_core::{ZeroCopyBuf, Op};
use std::str::FromStr;
use std::cell::RefCell;
use deno_core::serde::Deserialize;
use image::{GenericImageView};
use std::io::Read;
use std::env::temp_dir;
use euclid::{Point2D, UnknownUnit};

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

#[derive(Deserialize)]
enum JsonBlendMode {
    Dst,
    Src,
    Clear,
    SrcOver,
    DstOver,
    SrcIn,
    DstIn,
    SrcOut,
    DstOut,
    SrcAtop,
    DstAtop,
    Xor,
    Add,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Multiply,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

fn blend_from_json(json: JsonBlendMode) -> BlendMode {
    match json {
        JsonBlendMode::Dst => { BlendMode::Dst }
        JsonBlendMode::Src => { BlendMode::Src }
        JsonBlendMode::Clear => { BlendMode::Clear }
        JsonBlendMode::SrcOver => { BlendMode::SrcOver }
        JsonBlendMode::DstOver => { BlendMode::DstOver }
        JsonBlendMode::SrcIn => { BlendMode::SrcIn }
        JsonBlendMode::DstIn => { BlendMode::DstIn }
        JsonBlendMode::SrcOut => { BlendMode::SrcOut }
        JsonBlendMode::DstOut => { BlendMode::DstOut }
        JsonBlendMode::SrcAtop => { BlendMode::SrcAtop }
        JsonBlendMode::DstAtop => { BlendMode::DstAtop }
        JsonBlendMode::Xor => { BlendMode::Xor }
        JsonBlendMode::Add => { BlendMode::Add }
        JsonBlendMode::Screen => { BlendMode::Screen }
        JsonBlendMode::Overlay => { BlendMode::Overlay }
        JsonBlendMode::Darken => { BlendMode::Darken }
        JsonBlendMode::Lighten => { BlendMode::Lighten }
        JsonBlendMode::ColorDodge => { BlendMode::ColorDodge }
        JsonBlendMode::ColorBurn => { BlendMode::ColorBurn }
        JsonBlendMode::HardLight => { BlendMode::HardLight }
        JsonBlendMode::SoftLight => { BlendMode::SoftLight }
        JsonBlendMode::Difference => { BlendMode::Difference }
        JsonBlendMode::Exclusion => { BlendMode::Exclusion }
        JsonBlendMode::Multiply => { BlendMode::Multiply }
        JsonBlendMode::Hue => { BlendMode::Hue }
        JsonBlendMode::Saturation => { BlendMode::Saturation }
        JsonBlendMode::Color => { BlendMode::Color }
        JsonBlendMode::Luminosity => { BlendMode::Luminosity }
    }
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
    interface.register_op("op_dt_encode", op_dt_encode);
    interface.register_op("op_dt_destroy", op_dt_destroy);
    interface.register_op("op_dt_draw_image_at", op_dt_draw_image_at);
    interface.register_op("op_dt_set_transform", op_dt_set_transform);
    interface.register_op("op_dt_draw_image_with_size_at", op_dt_draw_image_with_size_at);
    interface.register_op("op_dt_push_clip_rect", op_dt_push_clip_rect);
    interface.register_op("op_dt_push_clip", op_dt_push_clip);
    interface.register_op("op_dt_pop_clip", op_dt_pop_clip);
    interface.register_op("op_dt_pop_layer", op_dt_pop_layer);
    interface.register_op("op_dt_push_layer", op_dt_push_layer);
    interface.register_op("op_dt_push_layer_with_blend", op_dt_push_layer_with_blend);
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

struct JsonImage {
    width: u32,
    height: u32,
    data: Box<[u32]>
}

fn get_arg_img(args: &mut [ZeroCopyBuf], idx: usize) -> Result<JsonImage, &str> {
    let res = args.get(idx);
    if res.is_none() {
        Err("not found")
    } else {
        let vec = res.unwrap();
        let mut res = Vec::<u32>::new();
        let dec = image::load_from_memory(vec.as_ref()).unwrap();
        for c in dec.as_rgba8().unwrap().as_raw().chunks(4) {
            // (A << 24) | (R << 16) | (G << 8) | B
            res.push((((c[3] as u32) << 24) | ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32)) as u32);
        }
        Ok(JsonImage { data: res.into_boxed_slice(), width: dec.width(), height: dec.height() })
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

fn op_dt_destroy(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id: u32 = get_arg_u32(_args, 0).unwrap();
    TARGETS.with(|map| {
        if let Some(_target) = map.borrow_mut().get_mut(&id) {
            map.borrow_mut().remove(&id);
            let res= b"0"; 
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
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

fn op_dt_encode(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id: u32 = get_arg_u32(_args, 0).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            let tmp = temp_dir().to_str().unwrap().to_owned();
            let file_name = format!("{}.png", uuid::Uuid::new_v4());
            let path = tmp + &file_name;
            target.write_png(&path).unwrap();
            let mut buf = Vec::<u8>::new();
            let mut file = std::fs::File::open(&path).unwrap();
            file.read_to_end(&mut buf).unwrap();
            std::fs::remove_file(&path).unwrap();
            Op::Sync(buf.into_boxed_slice())
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
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            let img = get_arg_img(_args, 1).unwrap();
            target.draw_image_at(x, y, &Image {
                width: img.width as i32,
                height: img.height as i32,
                data: &*img.data
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
    let w = get_arg_f32(_args, 4).unwrap();
    let h = get_arg_f32(_args, 5).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            let img = get_arg_img(_args, 1).unwrap();
            target.draw_image_with_size_at(x, y, w, h, &Image {
                width: img.width as i32,
                height: img.height as i32,
                data: &*img.data
            }, &DrawOptions::new());
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_set_transform(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let rc = get_arg_u8(_args, 1).unwrap();
    let m11 = get_arg_f32(_args, 2).unwrap();
    let m21 = get_arg_f32(_args, 3).unwrap();
    let m31 = get_arg_f32(_args, 4).unwrap();
    let m12 = get_arg_f32(_args, 5).unwrap();
    let m22 = get_arg_f32(_args, 6).unwrap();
    let m32 = get_arg_f32(_args, 7).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            let mut res= b"0";
            if rc == 0 { target.set_transform(&Transform::column_major(m11, m21, m31, m12, m22, m32)); }
            else if rc == 1 { target.set_transform(&Transform::row_major(m11, m21, m31, m12, m22, m32)); }
            else if rc == 2 { target.set_transform(&Transform::create_scale(m11, m21)); }
            else if rc == 3 { target.set_transform(&Transform::create_translation(m11, m21)); }
            else if rc == 4 {
                let angle = if m11 == 0.0 { euclid::Angle::degrees(m21) } else { euclid::Angle::radians(m21) };
                target.set_transform(&Transform::create_rotation(angle));
            } else { res = b"1"; }
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_push_clip_rect(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let x1 = get_arg_i32(_args, 1).unwrap();
    let y1 = get_arg_i32(_args, 2).unwrap();
    let x2 = get_arg_i32(_args, 3).unwrap();
    let y2 = get_arg_i32(_args, 4).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.push_clip_rect(IntRect::new(Point2D::<i32, UnknownUnit>::new(x1, y1), Point2D::<i32, UnknownUnit>::new(x2, y2)));
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_push_clip(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let path = get_arg_path(_args, 1).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.push_clip(&path);
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_pop_clip(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.pop_clip();
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_pop_layer(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.pop_layer();
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_push_layer(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let opacity = get_arg_f32(_args, 1).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.push_layer(opacity);
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}

fn op_dt_push_layer_with_blend(
    _interface: &mut dyn Interface,
    _args: &mut [ZeroCopyBuf],
) -> Op {
    let id = get_arg_u32(_args, 0).unwrap();
    let opacity = get_arg_f32(_args, 1).unwrap();
    let blend: JsonBlendMode = deno_core::serde_json::from_str(get_arg_str(_args, 2).unwrap()).unwrap();
    TARGETS.with(|map| {
        if let Some(target) = map.borrow_mut().get_mut(&id) {
            target.push_layer_with_blend(opacity, blend_from_json(blend));
            let res= b"0";
            Op::Sync(res.to_vec().into_boxed_slice())
        } else { let res= b"1"; Op::Sync(res.to_vec().into_boxed_slice()) }
    })
}