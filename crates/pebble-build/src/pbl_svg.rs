//! Compile SVGs into Pebble Draw Commands
use std::{
	path::{Path, PathBuf},
	str::FromStr,
};

use color_eyre::{
	Result,
	eyre::{Context, OptionExt, bail},
};
use serde::{Deserialize, Serialize};
use svgtypes::{
	Color, Length, LengthUnit, Paint, PointsParser, SimplePathSegment, SimplifyingPathParser,
	Transform, ViewBox,
};

const PDC_VERSION: u8 = 1;
const SVG_NS: &str = "http://www.w3.org/2000/svg";
/// `Fixed_S16_3` is 13 integer bits over 3 fractional ones (`math_fixed.h`),
/// so a precise path's coordinates step by 1/8th of a pixel over
/// [`PRECISE_RANGE`]. A plain `GPoint` is two `i16` pixels.
const PRECISE_FACTOR: f64 = 8.0;
const PRECISE_RANGE: (f64, f64) = (-4096.0, 4095.875);
const COORD_RANGE: (f64, f64) = (i16::MIN as f64, i16::MAX as f64);
const DEFAULT_TOLERANCE: f64 = 0.125;
const DEFAULT_FRAME_DURATION_MS: u16 = 33;
const DEFAULT_PLAY_COUNT: u16 = 1;
const SUBDIVISION_LIMIT: u32 = 16;
/// Cubic control point offset for a quarter turn, as a fraction of the radius
const KAPPA: f64 = 0.551_915_024_494;
const EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgSettings {
	/// Emit 1/8th-pixel `precise` paths. Defaults to `true` for images and
	/// `false` for sequences, as upstream's resource generator does.
	#[serde(default)]
	precise:           Option<bool>,
	/// Largest deviation, in pixels, permitted when subdividing a curve into
	/// straight segments.
	#[serde(default)]
	tolerance:         Option<f64>,
	#[serde(default)]
	frame_duration_ms: Option<u16>,
	#[serde(default)]
	play_count:        Option<u16>,
}

type Point = (f64, f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Size<T> {
	width:  T,
	height: T,
}

/// Shapes in `viewBox` space: every ancestor `transform` applied, every
/// `opacity` folded into the paint's alpha. Curves still curved.
struct Drawing<G> {
	size:   Size<f64>,
	shapes: Vec<Shape<G>>,
}

struct Shape<G> {
	stroke:       Option<Color>,
	stroke_width: f64,
	fill:         Option<Color>,
	geometry:     G,
}

enum Curved {
	Path {
		open:     bool,
		start:    Point,
		segments: Vec<Segment>,
	},
	Circle {
		center: Point,
		radius: f64,
	},
}

enum Segment {
	Line(Point),
	Quad(Point, Point),
	Cubic(Point, Point, Point),
}

/// Curves subdivided to the configured tolerance, so every path is a vertex
/// list. Still `viewBox` space.
enum Flat {
	Path { open: bool, points: Vec<Point> },
	Circle { center: Point, radius: f64 },
}

/// Pebble coordinate space and `ARGB2`: the lossy decisions are made, the wire
/// layout is not.
struct DrawCommand {
	stroke:       u8,
	stroke_width: u8,
	fill:         u8,
	geometry:     Encoded,
}

enum Encoded {
	Path {
		open:    bool,
		precise: bool,
		points:  Vec<(i16, i16)>,
	},
	Circle {
		center: (i16, i16),
		radius: u16,
	},
}

enum Pdc {
	Image {
		size:     Size<i16>,
		commands: Vec<DrawCommand>,
	},
	Sequence {
		size:       Size<i16>,
		play_count: u16,
		frames:     Vec<Frame>,
	},
}

struct Frame {
	duration_ms: u16,
	commands:    Vec<DrawCommand>,
}

#[derive(Clone, Copy)]
struct Style {
	stroke:         Option<Color>,
	stroke_width:   f64,
	fill:           Option<Color>,
	stroke_opacity: f64,
	fill_opacity:   f64,
	opacity:        f64,
}

impl Default for Style {
	fn default() -> Self {
		Style {
			stroke:         None,
			stroke_width:   1.0,
			fill:           Some(Color::new_rgb(0, 0, 0)),
			stroke_opacity: 1.0,
			fill_opacity:   1.0,
			opacity:        1.0,
		}
	}
}

pub(crate) fn process_svg(file: &Path, settings: &SvgSettings) -> Result<Vec<u8>> {
	let tolerance = settings.tolerance.unwrap_or(DEFAULT_TOLERANCE);
	if tolerance <= 0.0 || !tolerance.is_finite() {
		bail!("tolerance must be a positive number of pixels, got {tolerance}");
	}
	let pdc = if file.is_dir() {
		let precise = settings.precise.unwrap_or(false);
		let duration_ms = settings
			.frame_duration_ms
			.unwrap_or(DEFAULT_FRAME_DURATION_MS);
		let mut size = None;
		let mut frames = Vec::new();
		for path in frame_paths(file)? {
			let drawing = read_drawing(&path)?;
			let frame_size = pdc_size(drawing.size)?;
			match size {
				None => size = Some(frame_size),
				Some(first) if first != frame_size => bail!(
					"Frame {} is {}x{}, but the first frame is {}x{}; every frame of a sequence \
					 shares one viewBox",
					path.display(),
					frame_size.width,
					frame_size.height,
					first.width,
					first.height
				),
				Some(_) => (),
			}
			frames.push(Frame {
				duration_ms,
				commands: encode(flatten(drawing, tolerance).shapes, precise)?,
			});
		}
		Pdc::Sequence {
			size: size.ok_or_eyre("Sequence has no frames")?,
			play_count: settings.play_count.unwrap_or(DEFAULT_PLAY_COUNT),
			frames,
		}
	} else {
		let drawing = read_drawing(file)?;
		Pdc::Image {
			size:     pdc_size(drawing.size)?,
			commands: encode(
				flatten(drawing, tolerance).shapes,
				settings.precise.unwrap_or(true),
			)?,
		}
	};
	pdc.serialise()
}

fn read_drawing(file: &Path) -> Result<Drawing<Curved>> {
	let svg = std::fs::read_to_string(file)
		.wrap_err_with(|| format!("Resource Missing! {}", file.display()))?;
	parse(&svg).wrap_err_with(|| format!("In {}", file.display()))
}

/// Animation frames, in filename order.
fn frame_paths(dir: &Path) -> Result<Vec<PathBuf>> {
	let mut paths = Vec::new();
	for entry in std::fs::read_dir(dir)? {
		let path = entry?.path();
		if path
			.extension()
			.is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
		{
			paths.push(path);
		}
	}
	if paths.is_empty() {
		bail!("{} contains no .svg frames", dir.display());
	}
	paths.sort();
	Ok(paths)
}

fn parse(svg: &str) -> Result<Drawing<Curved>> {
	let doc = roxmltree::Document::parse_with_options(svg, roxmltree::ParsingOptions {
		allow_dtd: true,
		..roxmltree::ParsingOptions::default()
	})?;
	let root = doc.root_element();
	let name = root.tag_name().name();
	if name != "svg" {
		bail!("Root element is <{name}>, expected <svg>");
	}
	let view_box = ViewBox::from_str(
		root.attribute("viewBox")
			.ok_or_eyre("<svg> has no viewBox; a draw command image takes its size from it")?,
	)?;
	if view_box.w <= 0.0 || view_box.h <= 0.0 {
		bail!("viewBox is {}x{}", view_box.w, view_box.h);
	}
	let mut shapes = Vec::new();
	walk(
		root,
		Transform::new(1.0, 0.0, 0.0, 1.0, -view_box.x, -view_box.y),
		Style::default(),
		&mut shapes,
	)?;
	Ok(Drawing {
		size: Size {
			width:  view_box.w,
			height: view_box.h,
		},
		shapes,
	})
}

fn walk(
	node: roxmltree::Node<'_, '_>,
	transform: Transform,
	style: Style,
	shapes: &mut Vec<Shape<Curved>>,
) -> Result<()> {
	for child in node.children().filter(roxmltree::Node::is_element) {
		let tag = child.tag_name();
		if !matches!(tag.namespace(), None | Some(SVG_NS)) {
			continue;
		}
		let name = tag.name();
		if property(child, "display") == Some("none")
			|| matches!(name, "title" | "desc" | "metadata" | "defs")
		{
			continue;
		}
		for attr in ["clip-path", "mask", "filter"] {
			if child.attribute(attr).is_some() {
				bail!("<{name}> carries `{attr}`, which has no draw command equivalent");
			}
		}
		let transform = child
			.attribute("transform")
			.map(Transform::from_str)
			.transpose()
			.wrap_err_with(|| format!("Bad transform on <{name}>"))?
			.map_or(transform, |local| compose(transform, local));
		let style = inherit(child, style)?;
		if matches!(name, "g" | "svg" | "a" | "layer") {
			walk(child, transform, style, shapes)?;
			continue;
		}
		let stroke_scale = (transform.a * transform.d - transform.b * transform.c)
			.abs()
			.sqrt();
		for geometry in geometry(child, name, transform)? {
			shapes.push(Shape {
				stroke: fade(style.stroke, style.stroke_opacity * style.opacity),
				stroke_width: style.stroke_width * stroke_scale,
				fill: fade(style.fill, style.fill_opacity * style.opacity),
				geometry,
			});
		}
	}
	Ok(())
}

/// A presentation attribute, taking the `style` declaration over the attribute
/// of the same name.
fn property<'a>(node: roxmltree::Node<'a, '_>, name: &str) -> Option<&'a str> {
	node.attribute("style")
		.and_then(|style| {
			style.split(';').find_map(|decl| {
				let (key, value) = decl.split_once(':')?;
				(key.trim() == name).then(|| value.trim())
			})
		})
		.or_else(|| node.attribute(name))
}

fn inherit(node: roxmltree::Node<'_, '_>, mut style: Style) -> Result<Style> {
	if let Some(raw) = property(node, "stroke") {
		style.stroke = paint(raw, style.stroke)?;
	}
	if let Some(raw) = property(node, "fill") {
		style.fill = paint(raw, style.fill)?;
	}
	if let Some(raw) = property(node, "stroke-width") {
		style.stroke_width = number(raw, "stroke-width")?;
	}
	if let Some(raw) = property(node, "stroke-opacity") {
		style.stroke_opacity = opacity(raw)?;
	}
	if let Some(raw) = property(node, "fill-opacity") {
		style.fill_opacity = opacity(raw)?;
	}
	// `opacity` is group compositing in SVG; a flat command list can only
	// approximate it by compounding into the paint, as upstream does
	if let Some(raw) = property(node, "opacity") {
		style.opacity *= opacity(raw)?;
	}
	Ok(style)
}

fn paint(raw: &str, inherited: Option<Color>) -> Result<Option<Color>> {
	match Paint::from_str(raw).wrap_err_with(|| format!("Bad paint value {raw:?}"))? {
		Paint::None => Ok(None),
		Paint::Inherit => Ok(inherited),
		Paint::Color(colour) => Ok(Some(colour)),
		_ => bail!("Paint {raw:?} has no draw command equivalent; colours and `none` only"),
	}
}

fn opacity(raw: &str) -> Result<f64> {
	let raw = raw.trim();
	let value = match raw.strip_suffix('%') {
		Some(percent) => percent.trim().parse::<f64>().map(|value| value / 100.0),
		None => raw.parse::<f64>(),
	}
	.wrap_err_with(|| format!("Bad opacity value {raw:?}"))?;
	Ok(value.clamp(0.0, 1.0))
}

fn number(raw: &str, name: &str) -> Result<f64> {
	let length = Length::from_str(raw).wrap_err_with(|| format!("Bad `{name}` value {raw:?}"))?;
	match length.unit {
		LengthUnit::None | LengthUnit::Px => Ok(length.number),
		unit => {
			bail!("`{name}` is {raw:?}; only unitless and `px` lengths are supported, not {unit:?}")
		}
	}
}

fn attr_number(node: roxmltree::Node<'_, '_>, name: &str, default: f64) -> Result<f64> {
	node.attribute(name)
		.map_or(Ok(default), |raw| number(raw, name))
}

/// Alpha scaled by the resolved `opacity` chain.
fn fade(colour: Option<Color>, opacity: f64) -> Option<Color> {
	colour.map(|colour| Color {
		alpha: (f64::from(colour.alpha) * opacity) as u8,
		..colour
	})
}

fn geometry(
	node: roxmltree::Node<'_, '_>,
	name: &str,
	transform: Transform,
) -> Result<Vec<Curved>> {
	match name {
		"path" => path_data(
			node.attribute("d")
				.ok_or_eyre("<path> has no `d` attribute")?,
			transform,
		),
		"rect" => rect(node, transform),
		"circle" => {
			let radius = attr_number(node, "r", 0.0)?;
			Ok(ellipse(
				(attr_number(node, "cx", 0.0)?, attr_number(node, "cy", 0.0)?),
				(radius, radius),
				transform,
			))
		}
		"ellipse" => Ok(ellipse(
			(attr_number(node, "cx", 0.0)?, attr_number(node, "cy", 0.0)?),
			(attr_number(node, "rx", 0.0)?, attr_number(node, "ry", 0.0)?),
			transform,
		)),
		"line" => Ok(polyline(
			[
				(attr_number(node, "x1", 0.0)?, attr_number(node, "y1", 0.0)?),
				(attr_number(node, "x2", 0.0)?, attr_number(node, "y2", 0.0)?),
			],
			true,
			transform,
		)),
		"polyline" | "polygon" => Ok(polyline(
			PointsParser::from(
				node.attribute("points")
					.ok_or_eyre("<{name}> has no `points` attribute")?,
			),
			name == "polyline",
			transform,
		)),
		_ => bail!("<{name}> has no draw command equivalent"),
	}
}

fn polyline(
	points: impl IntoIterator<Item = Point>,
	open: bool,
	transform: Transform,
) -> Vec<Curved> {
	let mut points = points.into_iter().map(|point| apply(transform, point));
	let Some(start) = points.next() else {
		return Vec::new();
	};
	vec![Curved::Path {
		open,
		start,
		segments: points.map(Segment::Line).collect(),
	}]
}

fn rect(node: roxmltree::Node<'_, '_>, transform: Transform) -> Result<Vec<Curved>> {
	let (x, y) = (attr_number(node, "x", 0.0)?, attr_number(node, "y", 0.0)?);
	let (w, h) = (
		attr_number(node, "width", 0.0)?,
		attr_number(node, "height", 0.0)?,
	);
	if w <= 0.0 || h <= 0.0 {
		return Ok(Vec::new());
	}
	let (rx, ry) = match (node.attribute("rx"), node.attribute("ry")) {
		(None, None) => (0.0, 0.0),
		(rx, ry) => {
			let rx = rx.map(|raw| number(raw, "rx")).transpose()?;
			let ry = ry.map(|raw| number(raw, "ry")).transpose()?;
			(
				rx.or(ry).unwrap_or(0.0).clamp(0.0, w / 2.0),
				ry.or(rx).unwrap_or(0.0).clamp(0.0, h / 2.0),
			)
		}
	};
	if rx <= 0.0 || ry <= 0.0 {
		return Ok(polyline(
			[(x, y), (x + w, y), (x + w, y + h), (x, y + h)],
			false,
			transform,
		));
	}
	let (kx, ky) = (rx * (1.0 - KAPPA), ry * (1.0 - KAPPA));
	let (right, bottom) = (x + w, y + h);
	let at = |point: Point| apply(transform, point);
	Ok(vec![Curved::Path {
		open:     false,
		start:    at((x + rx, y)),
		segments: vec![
			Segment::Line(at((right - rx, y))),
			Segment::Cubic(
				at((right - kx, y)),
				at((right, y + ky)),
				at((right, y + ry)),
			),
			Segment::Line(at((right, bottom - ry))),
			Segment::Cubic(
				at((right, bottom - ky)),
				at((right - kx, bottom)),
				at((right - rx, bottom)),
			),
			Segment::Line(at((x + rx, bottom))),
			Segment::Cubic(
				at((x + kx, bottom)),
				at((x, bottom - ky)),
				at((x, bottom - ry)),
			),
			Segment::Line(at((x, y + ry))),
			Segment::Cubic(at((x, y + ky)), at((x + kx, y)), at((x + rx, y))),
		],
	}])
}

/// A circle command where the shape survives the transform as a circle, four
/// cubics otherwise.
fn ellipse(center: Point, radii: Point, transform: Transform) -> Vec<Curved> {
	let (rx, ry) = radii;
	if rx <= 0.0 || ry <= 0.0 {
		return Vec::new();
	}
	if (rx - ry).abs() <= EPSILON * rx.max(ry)
		&& let Some(scale) = similarity_scale(transform)
	{
		return vec![Curved::Circle {
			center: apply(transform, center),
			radius: rx * scale,
		}];
	}
	let (cx, cy) = center;
	let (kx, ky) = (rx * KAPPA, ry * KAPPA);
	let at = |point: Point| apply(transform, point);
	vec![Curved::Path {
		open:     false,
		start:    at((cx + rx, cy)),
		segments: vec![
			Segment::Cubic(
				at((cx + rx, cy + ky)),
				at((cx + kx, cy + ry)),
				at((cx, cy + ry)),
			),
			Segment::Cubic(
				at((cx - kx, cy + ry)),
				at((cx - rx, cy + ky)),
				at((cx - rx, cy)),
			),
			Segment::Cubic(
				at((cx - rx, cy - ky)),
				at((cx - kx, cy - ry)),
				at((cx, cy - ry)),
			),
			Segment::Cubic(
				at((cx + kx, cy - ry)),
				at((cx + rx, cy - ky)),
				at((cx + rx, cy)),
			),
		],
	}]
}

/// Each subpath becomes its own command. Upstream concatenates them into one,
/// which draws a spurious edge between them.
fn path_data(d: &str, transform: Transform) -> Result<Vec<Curved>> {
	let mut paths = Vec::new();
	let mut start = None;
	let mut segments: Vec<Segment> = Vec::new();
	let mut closed = false;
	let mut flush = |start: Option<Point>, segments: Vec<Segment>, closed: bool| {
		if let Some(start) = start
			&& !segments.is_empty()
		{
			let end = match segments.last() {
				Some(Segment::Line(to) | Segment::Quad(_, to) | Segment::Cubic(_, _, to)) => *to,
				None => start,
			};
			paths.push(Curved::Path {
				open: !closed && distance(start, end) > EPSILON,
				start,
				segments,
			});
		}
	};
	let at = |point: Point| apply(transform, point);
	for segment in SimplifyingPathParser::from(d) {
		match segment.wrap_err("Bad path data")? {
			SimplePathSegment::MoveTo { x, y } => {
				flush(start, std::mem::take(&mut segments), closed);
				start = Some(at((x, y)));
				closed = false;
			}
			SimplePathSegment::LineTo { x, y } => segments.push(Segment::Line(at((x, y)))),
			SimplePathSegment::Quadratic { x1, y1, x, y } => {
				segments.push(Segment::Quad(at((x1, y1)), at((x, y))));
			}
			SimplePathSegment::CurveTo {
				x1,
				y1,
				x2,
				y2,
				x,
				y,
			} => segments.push(Segment::Cubic(at((x1, y1)), at((x2, y2)), at((x, y)))),
			SimplePathSegment::ClosePath => {
				if let Some(start) = start {
					segments.push(Segment::Line(start));
				}
				closed = true;
			}
		}
	}
	flush(start, segments, closed);
	Ok(paths)
}

fn compose(outer: Transform, inner: Transform) -> Transform {
	Transform::new(
		outer.a * inner.a + outer.c * inner.b,
		outer.b * inner.a + outer.d * inner.b,
		outer.a * inner.c + outer.c * inner.d,
		outer.b * inner.c + outer.d * inner.d,
		outer.a * inner.e + outer.c * inner.f + outer.e,
		outer.b * inner.e + outer.d * inner.f + outer.f,
	)
}

fn apply(ts: Transform, point: Point) -> Point {
	(
		ts.a * point.0 + ts.c * point.1 + ts.e,
		ts.b * point.0 + ts.d * point.1 + ts.f,
	)
}

/// `Some(scale)` when the linear part is a rotation and a single uniform
/// scale, the only case where a circle stays a circle.
fn similarity_scale(ts: Transform) -> Option<f64> {
	let (columns, skew) = (
		(ts.a.hypot(ts.b), ts.c.hypot(ts.d)),
		ts.a * ts.c + ts.b * ts.d,
	);
	let bound = EPSILON * columns.0.max(columns.1).max(1.0);
	((columns.0 - columns.1).abs() <= bound && skew.abs() <= bound).then_some(columns.0)
}

fn distance(from: Point, to: Point) -> f64 {
	(to.0 - from.0).hypot(to.1 - from.1)
}

fn flatten(drawing: Drawing<Curved>, tolerance: f64) -> Drawing<Flat> {
	Drawing {
		size:   drawing.size,
		shapes: drawing
			.shapes
			.into_iter()
			.map(|shape| Shape {
				stroke:       shape.stroke,
				stroke_width: shape.stroke_width,
				fill:         shape.fill,
				geometry:     match shape.geometry {
					Curved::Circle { center, radius } => Flat::Circle { center, radius },
					Curved::Path {
						open,
						start,
						segments,
					} => Flat::Path {
						open,
						points: vertices(start, segments, tolerance),
					},
				},
			})
			.collect(),
	}
}

/// The polyline through `segments`, dropping a final vertex that repeats the
/// first as upstream does.
fn vertices(start: Point, segments: Vec<Segment>, tolerance: f64) -> Vec<Point> {
	let mut points = vec![start];
	let mut from = start;
	for segment in segments {
		from = match segment {
			Segment::Line(to) => {
				points.push(to);
				to
			}
			Segment::Quad(control, to) => {
				flatten_quad(from, control, to, tolerance, 0, &mut points);
				to
			}
			Segment::Cubic(first, second, to) => {
				flatten_cubic(from, first, second, to, tolerance, 0, &mut points);
				to
			}
		};
	}
	while points.len() > 1 && distance(start, points[points.len() - 1]) <= EPSILON {
		points.pop();
	}
	points
}

fn midpoint(from: Point, to: Point) -> Point {
	(f64::midpoint(from.0, to.0), f64::midpoint(from.1, to.1))
}

/// Perpendicular distance from `point` to the chord `from`-`to`.
fn deviation(from: Point, to: Point, point: Point) -> f64 {
	let (dx, dy) = (to.0 - from.0, to.1 - from.1);
	let length = dx.hypot(dy);
	if length <= EPSILON {
		return distance(from, point);
	}
	((point.0 - from.0) * dy - (point.1 - from.1) * dx).abs() / length
}

fn flatten_quad(
	from: Point,
	control: Point,
	to: Point,
	tolerance: f64,
	depth: u32,
	out: &mut Vec<Point>,
) {
	if depth == SUBDIVISION_LIMIT || deviation(from, to, control) <= tolerance {
		out.push(to);
		return;
	}
	let (first, second) = (midpoint(from, control), midpoint(control, to));
	let split = midpoint(first, second);
	flatten_quad(from, first, split, tolerance, depth + 1, out);
	flatten_quad(split, second, to, tolerance, depth + 1, out);
}

fn flatten_cubic(
	from: Point,
	first: Point,
	second: Point,
	to: Point,
	tolerance: f64,
	depth: u32,
	out: &mut Vec<Point>,
) {
	if depth == SUBDIVISION_LIMIT
		|| deviation(from, to, first).max(deviation(from, to, second)) <= tolerance
	{
		out.push(to);
		return;
	}
	let (left, middle, right) = (
		midpoint(from, first),
		midpoint(first, second),
		midpoint(second, to),
	);
	let (left_end, right_start) = (midpoint(left, middle), midpoint(middle, right));
	let split = midpoint(left_end, right_start);
	flatten_cubic(from, left, left_end, split, tolerance, depth + 1, out);
	flatten_cubic(split, right_start, right, to, tolerance, depth + 1, out);
}

fn encode(shapes: Vec<Shape<Flat>>, precise: bool) -> Result<Vec<DrawCommand>> {
	let mut commands = Vec::with_capacity(shapes.len());
	for shape in shapes {
		let fill = argb2(shape.fill);
		let mut stroke = argb2(shape.stroke);
		if stroke == 0 && fill == 0 {
			continue;
		}
		let mut stroke_width = shape.stroke_width.round().clamp(0.0, f64::from(u8::MAX)) as u8;
		if stroke == 0 {
			stroke_width = 0;
		} else if stroke_width == 0 {
			stroke = 0;
		}
		let geometry = match shape.geometry {
			Flat::Circle { center, radius } => Encoded::Circle {
				center: encode_point(center, false)?,
				radius: u16::try_from(radius.max(0.0) as u64)
					.wrap_err_with(|| format!("Circle radius {radius} over 65535px"))?,
			},
			Flat::Path { open, points } => {
				let mut encoded: Vec<(i16, i16)> = Vec::with_capacity(points.len());
				for point in points {
					let point = encode_point(point, precise)?;
					if encoded.last() != Some(&point) {
						encoded.push(point);
					}
				}
				if encoded.is_empty() {
					continue;
				}
				Encoded::Path {
					open,
					precise,
					points: encoded,
				}
			}
		};
		commands.push(DrawCommand {
			stroke,
			stroke_width,
			fill,
			geometry,
		});
	}
	Ok(commands)
}

/// Pebble's pixel grid sits half a pixel off the SVG one.
fn encode_point(point: Point, precise: bool) -> Result<(i16, i16)> {
	let (scale, (low, high)) = if precise {
		(PRECISE_FACTOR, PRECISE_RANGE)
	} else {
		(1.0, COORD_RANGE)
	};
	let fit = |value: f64| -> Result<i16> {
		if !value.is_finite() {
			bail!("Coordinate {value} is not a finite number");
		}
		let shifted = value - 0.5;
		if shifted < low || shifted > high {
			bail!(
				"Coordinate {value} is outside the {low}..={high}px range of a{} path",
				if precise { " precise" } else { "" }
			);
		}
		Ok((shifted * scale).round() as i16)
	};
	Ok((fit(point.0)?, fit(point.1)?))
}

/// Two bits per channel, to the nearest of the four levels. Zero alpha zeroes
/// the whole colour, and is how a command list spells "absent".
fn argb2(colour: Option<Color>) -> u8 {
	let Some(colour) = colour else {
		return 0;
	};
	let level = |channel: u8| ((u16::from(channel) + 42) / 85) as u8;
	let alpha = level(colour.alpha);
	if alpha == 0 {
		0
	} else {
		alpha << 6 | level(colour.red) << 4 | level(colour.green) << 2 | level(colour.blue)
	}
}

fn pdc_size(size: Size<f64>) -> Result<Size<i16>> {
	let fit = |value: f64| -> Result<i16> {
		i16::try_from(value.round() as i64)
			.wrap_err_with(|| format!("viewBox dimension {value} does not fit in an i16"))
	};
	Ok(Size {
		width:  fit(size.width)?,
		height: fit(size.height)?,
	})
}

impl Encoded {
	fn type_byte(&self) -> u8 {
		match self {
			Encoded::Path { precise: false, .. } => 1,
			Encoded::Circle { .. } => 2,
			Encoded::Path { precise: true, .. } => 3,
		}
	}
}

impl DrawCommand {
	fn serialise(&self, out: &mut Vec<u8>) -> Result<()> {
		out.push(self.geometry.type_byte());
		// GDrawCommand's second byte is a `hidden:1` flag plus padding
		out.push(0);
		out.push(self.stroke);
		out.push(self.stroke_width);
		out.push(self.fill);
		let points = match &self.geometry {
			Encoded::Path { open, points, .. } => {
				out.push(u8::from(*open));
				out.push(0);
				points.as_slice()
			}
			Encoded::Circle { center, radius } => {
				out.extend(radius.to_le_bytes());
				std::slice::from_ref(center)
			}
		};
		out.extend(
			u16::try_from(points.len())
				.wrap_err("Path has more than 65535 points")?
				.to_le_bytes(),
		);
		for (x, y) in points {
			out.extend(x.to_le_bytes());
			out.extend(y.to_le_bytes());
		}
		Ok(())
	}
}

fn serialise_list(commands: &[DrawCommand], out: &mut Vec<u8>) -> Result<()> {
	out.extend(
		u16::try_from(commands.len())
			.wrap_err("More than 65535 draw commands")?
			.to_le_bytes(),
	);
	for command in commands {
		command.serialise(out)?;
	}
	Ok(())
}

impl Pdc {
	fn serialise(&self) -> Result<Vec<u8>> {
		let (magic, size) = match self {
			Pdc::Image { size, .. } => (b"PDCI", size),
			Pdc::Sequence { size, .. } => (b"PDCS", size),
		};
		let mut payload = vec![PDC_VERSION, 0];
		payload.extend(size.width.to_le_bytes());
		payload.extend(size.height.to_le_bytes());
		match self {
			Pdc::Image { commands, .. } => serialise_list(commands, &mut payload)?,
			Pdc::Sequence {
				play_count, frames, ..
			} => {
				payload.extend(play_count.to_le_bytes());
				payload.extend(
					u16::try_from(frames.len())
						.wrap_err("More than 65535 frames")?
						.to_le_bytes(),
				);
				for frame in frames {
					payload.extend(frame.duration_ms.to_le_bytes());
					serialise_list(&frame.commands, &mut payload)?;
				}
			}
		}
		let mut pdc = magic.to_vec();
		pdc.extend((payload.len() as u32).to_le_bytes());
		pdc.extend(payload);
		Ok(pdc)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const IMAGE_SVG: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48">
  <rect x="2" y="3" width="20" height="14" fill="#ff0000" stroke="#00ff00" stroke-width="2"/>
  <circle cx="30" cy="12" r="7" fill="#0000ff" stroke="none" stroke-width="0"/>
  <polygon points="4,24 20,24 12,40" fill="#ffffff" stroke="#000000" stroke-width="1"/>
  <polyline points="26,26 34,30 40,26 44,34" fill="none" stroke="#ff00ff" stroke-width="3"/>
  <line x1="2" y1="45" x2="46" y2="45" fill="none" stroke="#00ffff" stroke-width="1"/>
  <path d="M 26 40 L 34 40 L 34 46 Z" fill="#aaaaaa" stroke="none" stroke-width="0"/>
</svg>
"##;

	/// `create_pdc_data_from_path(IMAGE_SVG, duration=0, play_count=0, precise=True)`.
	/// Every colour is an exact palette level, so upstream's truncating
	/// quantiser and this one's nearest agree on the bytes.
	const IMAGE_PDCI: &str = "504443498200000001003000300006000300cc02f0000004000c001400ac0014
	                          00ac0084000c00840002000000c3070001001e000c000300c001ff000003001c
	                          00bc009c00bc005c003c010300f3030001000400cc00cc000c01ec003c01cc00
	                          5c010c010300cf0100010002000c0064016c01640103000000ea00000300cc00
	                          3c010c013c010c016c01";

	/// `create_pdc_data_from_path(<dir of frame(1..=3)>, duration=33, play_count=1,
	/// precise=False)`
	const SEQUENCE_PDCS: &str = "50444353a000000001001800180001000300210002000100c001f00000040001
	                             0001000900010009000900010009000100ff02cc000003000c000200140002
	                             0010000a00210002000100c001f000000400020002000a0002000a000a0002
	                             000a000100ff02cc000003000c0002001400020010000a00210002000100c0
	                             01f000000400030003000b0003000b000b0003000b000100ff02cc00000300
	                             0c0002001400020010000a00";

	fn frame(offset: u8) -> String {
		format!(
			r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <rect x="{offset}" y="{offset}" width="8" height="8" fill="#ff0000" stroke="#000000" stroke-width="1"/>
  <polygon points="12,2 20,2 16,10" fill="#00ff00" stroke="#ffffff" stroke-width="2"/>
</svg>
"##
		)
	}

	fn unhex(hex: &str) -> Vec<u8> {
		hex.split_whitespace()
			.collect::<String>()
			.as_bytes()
			.chunks(2)
			.map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
			.collect()
	}

	fn commands(svg: &str, precise: bool) -> (Size<f64>, Vec<DrawCommand>) {
		let drawing = parse(svg).unwrap();
		let size = drawing.size;
		(
			size,
			encode(flatten(drawing, DEFAULT_TOLERANCE).shapes, precise).unwrap(),
		)
	}

	#[test]
	fn parity_with_upstream_image() {
		let (size, commands) = commands(IMAGE_SVG, true);
		let pdc = Pdc::Image {
			size: pdc_size(size).unwrap(),
			commands,
		};
		assert_eq!(pdc.serialise().unwrap(), unhex(IMAGE_PDCI));
	}

	#[test]
	fn parity_with_upstream_sequence() {
		let frames = (1..=3)
			.map(|offset| {
				let (_, commands) = commands(&frame(offset), false);
				Frame {
					duration_ms: DEFAULT_FRAME_DURATION_MS,
					commands,
				}
			})
			.collect();
		let pdc = Pdc::Sequence {
			size: Size {
				width:  24,
				height: 24,
			},
			play_count: DEFAULT_PLAY_COUNT,
			frames,
		};
		assert_eq!(pdc.serialise().unwrap(), unhex(SEQUENCE_PDCS));
	}

	#[test]
	fn transformed_circle_stays_a_circle() {
		let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40">
			<g transform="translate(4 4) rotate(20)">
				<circle cx="10" cy="10" r="5" fill="#ff0000" transform="scale(1.5)"/>
			</g>
		</svg>"##;
		let (_, commands) = commands(svg, true);
		assert!(matches!(commands.as_slice(), [DrawCommand {
			geometry: Encoded::Circle { radius: 7, .. },
			..
		}]));
	}

	#[test]
	fn squashed_circle_becomes_a_path() {
		let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40">
			<circle cx="10" cy="10" r="5" fill="#ff0000" transform="scale(2 1)"/>
		</svg>"##;
		let (_, commands) = commands(svg, true);
		assert!(matches!(commands.as_slice(), [DrawCommand {
			geometry: Encoded::Path { open: false, .. },
			..
		}]));
	}

	#[test]
	fn precise_range_is_enforced() {
		let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 5000 100">
			<path d="M 10 10 L 4990 90 Z" fill="#ff0000"/>
		</svg>"##;
		let drawing = parse(svg).unwrap();
		let shapes = flatten(drawing, DEFAULT_TOLERANCE).shapes;
		assert!(encode(shapes, true).is_err());

		let drawing = parse(svg).unwrap();
		let shapes = flatten(drawing, DEFAULT_TOLERANCE).shapes;
		assert!(encode(shapes, false).is_ok());
	}

	#[test]
	fn unrepresentable_input_bails() {
		for svg in [
			r#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1"/></svg>"#,
			r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4">
				<text x="0" y="0">nope</text></svg>"#,
			r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4">
				<rect width="1" height="1" fill="url(#grad)"/></svg>"#,
			r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4">
				<rect width="1" height="1" clip-path="url(#c)"/></svg>"#,
			r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4">
				<rect width="1mm" height="1"/></svg>"#,
		] {
			assert!(parse(svg).is_err(), "expected a bail for {svg}");
		}
	}
}
