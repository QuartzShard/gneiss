use darling::Error as DarlingError;
use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use ra_ap_rustc_parse_format::{
	Alignment, Count, FormatSpec, ParseMode, Parser, Piece, Position, Sign,
};
use syn::{
	Expr, Ident, LitStr, Token,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

/// Where the formatted text goes
enum Target {
	/// Appended to a `Sink`, as `pbl_fmt_write!` does
	Write { dest: Expr },
	/// Logged by `app_log`
	Log {
		level: Box<Expr>,
		file:  Box<Expr>,
		line:  Box<Expr>,
	},
}

/// A target, then `"format", args...`
struct Input {
	target: Target,
	format: LitStr,
	args:   Vec<FmtArg>,
}

/// `pbl_fmt_write!`'s input: `dest, "format", args...`
struct WriteInput(Input);

/// `__pbl_log!`'s input: `level, file, line, "format", args...`
struct LogInput(Input);

/// A positional argument, or `name = expr`
struct FmtArg {
	name: Option<Ident>,
	expr: Expr,
}

impl Parse for FmtArg {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(Ident) && input.peek2(Token![=]) && !input.peek2(Token![==]) {
			let name = input.parse()?;
			input.parse::<Token![=]>()?;
			Ok(Self {
				name: Some(name),
				expr: input.parse()?,
			})
		} else {
			Ok(Self {
				name: None,
				expr: input.parse()?,
			})
		}
	}
}

impl Input {
	/// The `"format", args...` after `target` and its comma
	fn rest(target: Target, input: ParseStream) -> syn::Result<Self> {
		let format = input.parse()?;
		let args = if input.is_empty() {
			Vec::new()
		} else {
			input.parse::<Token![,]>()?;
			Punctuated::<FmtArg, Token![,]>::parse_terminated(input)?
				.into_iter()
				.collect()
		};
		Ok(Self {
			target,
			format,
			args,
		})
	}
}

impl Parse for WriteInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let dest = input.parse()?;
		input.parse::<Token![,]>()?;
		Input::rest(Target::Write { dest }, input).map(Self)
	}
}

impl Parse for LogInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let level = Box::new(input.parse()?);
		input.parse::<Token![,]>()?;
		let file = Box::new(input.parse()?);
		input.parse::<Token![,]>()?;
		let line = Box::new(input.parse()?);
		input.parse::<Token![,]>()?;
		Input::rest(Target::Log { level, file, line }, input).map(Self)
	}
}

// Room for a conversion's longest ending, `.*s` or `lld`, and its `%`
const CONVERSION_BOUND: usize = 4;
// Plain `{}` arguments at which pbl_fmt_write! warns, then refuses: each doubles the calls it writes
const WARN_AMBIGUOUS: usize = 5;
const MAX_AMBIGUOUS: usize = 9;

pub fn pbl_fmt_write(input: TokenStream) -> TokenStream {
	run(
		syn::parse2::<WriteInput>(input).map(|i| i.0),
		"pbl_fmt_write!",
		quote!(::core::result::Result::<(), ::core::fmt::Error>::Ok(())),
	)
}

pub fn pbl_log(input: TokenStream) -> TokenStream {
	run(
		syn::parse2::<LogInput>(input).map(|i| i.0),
		"log!",
		quote!(()),
	)
}

/// Expand `input`, or on errors, emit them and `placeholder`, a value of the macro's type: the
/// code `expand` made from a malformed format may not compile either
fn run(input: syn::Result<Input>, name: &str, placeholder: TokenStream) -> TokenStream {
	let mut errs = DarlingError::accumulator();
	let out = errs
		.handle(input.map_err(DarlingError::from))
		.map(|input| expand(input, name, &mut errs));
	match errs.finish() {
		Err(e) => {
			let errors = e.write_errors();
			quote! {{ #errors #placeholder }}
		}
		Ok(()) => out.unwrap_or_default(),
	}
}

/// What a value is formatted as
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
	Str,
	Int,
}

/// What a conversion asks for: `{:s}`, an integer conversion, or a plain `{}`, either
#[derive(Clone, Copy, Debug, PartialEq)]
enum Want {
	Str,
	Int(&'static str),
	Plain,
}

impl Want {
	/// What it fixes its value as, if anything
	fn kind(self) -> Option<Kind> {
		match self {
			Want::Str => Some(Kind::Str),
			Want::Int(_) => Some(Kind::Int),
			Want::Plain => None,
		}
	}
}

/// What a conversion's type letter asks for
fn want(ty: &str) -> Result<Want, String> {
	match ty {
		"" => Ok(Want::Plain),
		"s" => Ok(Want::Str),
		"d" => Ok(Want::Int("d")),
		"x" => Ok(Want::Int("x")),
		"X" => Ok(Want::Int("X")),
		"o" => Ok(Want::Int("o")),
		"?" => Err("pbl_fmt_write! has no `Debug` formatting".into()),
		other => Err(format!("pbl_fmt_write! can't format `{{:{other}}}`")),
	}
}

/// The values the format reads in order of first use: an argument, or a captured
/// variable
struct Values<'a> {
	args:        &'a [FmtArg],
	positional:  usize,
	// Where a captured variable's name points
	span:        Span,
	exprs:       Vec<Expr>,
	// How each value is named in errors
	labels:      Vec<String>,
	// What `{:s}` or an integer conversion fixes each value as
	kinds:       Vec<Option<Kind>>,
	slot_of_arg: Vec<Option<usize>>,
	captures:    Vec<(String, usize)>,
}

impl<'a> Values<'a> {
	fn new(args: &'a [FmtArg], span: Span) -> Self {
		Self {
			args,
			positional: args.iter().take_while(|a| a.name.is_none()).count(),
			span,
			exprs: Vec::new(),
			labels: Vec::new(),
			kinds: Vec::new(),
			slot_of_arg: vec![None; args.len()],
			captures: Vec::new(),
		}
	}

	/// A positional argument after a named one, which `format!` rejects too
	fn late_positional(&self) -> Option<&'a Expr> {
		self.args[self.positional..]
			.iter()
			.find(|a| a.name.is_none())
			.map(|a| &a.expr)
	}

	fn push(&mut self, expr: Expr, label: String) -> usize {
		self.exprs.push(expr);
		self.labels.push(label);
		self.kinds.push(None);
		self.exprs.len() - 1
	}

	/// The value of argument `i`, added on its first use
	fn arg(&mut self, i: usize, label: String) -> usize {
		if let Some(slot) = self.slot_of_arg[i] {
			return slot;
		}
		let slot = self.push(self.args[i].expr.clone(), label);
		self.slot_of_arg[i] = Some(slot);
		slot
	}

	/// The value `position` names
	fn slot(&mut self, position: &Position) -> Result<usize, String> {
		match position {
			Position::ArgumentImplicitlyIs(i) | Position::ArgumentIs(i) => {
				if *i < self.positional {
					Ok(self.arg(*i, format!("argument {i}")))
				} else {
					Err(format!(
						"argument {i} is missing: {} positional argument{} given",
						self.positional,
						if self.positional == 1 { "" } else { "s" }
					))
				}
			}
			Position::ArgumentNamed(name) => {
				if let Some(i) = self
					.args
					.iter()
					.position(|a| a.name.as_ref().is_some_and(|n| n == name))
				{
					Ok(self.arg(i, format!("`{name}`")))
				} else if let Some((_, slot)) = self.captures.iter().find(|(n, _)| n == name) {
					Ok(*slot)
				} else {
					let ident = Ident::new(name, self.span);
					let slot = self.push(syn::parse_quote!(#ident), format!("`{name}`"));
					self.captures.push(((*name).to_string(), slot));
					Ok(slot)
				}
			}
		}
	}

	/// Fix `slot` as `kind`, which an earlier conversion may have fixed otherwise
	fn fix(&mut self, slot: usize, kind: Kind) -> Result<(), String> {
		match self.kinds[slot] {
			Some(fixed) if fixed != kind => Err(format!(
				"pbl_fmt_write!: {} is formatted as both a string and an integer",
				self.labels[slot]
			)),
			_ => {
				self.kinds[slot] = Some(kind);
				Ok(())
			}
		}
	}

	/// Arguments no conversion reads
	fn unused(&self) -> impl Iterator<Item = &'a Expr> + '_ {
		self.args
			.iter()
			.zip(&self.slot_of_arg)
			.filter(|(_, slot)| slot.is_none())
			.map(|(arg, _)| &arg.expr)
	}
}

/// A conversion's `snprintf` flags and width, checked against what it asks for
#[derive(Debug, PartialEq)]
struct Spec {
	flags:        String,
	width:        String,
	// Pads on the right if the value turns out to be a string, as `format!` does strings
	left_default: bool,
	// Its flags need an integer, or a signed one
	needs_int:    bool,
	needs_signed: bool,
}

/// `spec` as `snprintf` flags and a width, or why `snprintf` can't do it
fn spec(spec: &FormatSpec<'_>, want: Want) -> Result<Spec, Vec<String>> {
	let mut errors = Vec::new();
	if spec.fill.is_some_and(|f| f != ' ') {
		errors.push("pbl_fmt_write! pads with spaces only: for zeros, use `{:05}`".into());
	}
	if spec.align == Alignment::AlignCenter {
		errors.push("pbl_fmt_write! can't centre: `snprintf` aligns left or right".into());
	}
	if spec.precision != Count::CountImplied {
		errors.push("pbl_fmt_write! has no precision".into());
	}
	if spec.debug_hex.is_some() {
		errors.push("pbl_fmt_write! has no `Debug` formatting".into());
	}
	let width = match spec.width {
		Count::CountImplied => String::new(),
		Count::CountIs(n) => n.to_string(),
		_ => {
			errors.push("pbl_fmt_write! takes widths as numbers, not from arguments".into());
			String::new()
		}
	};

	let mut flags = String::new();
	let mut needs_int = false;
	let mut needs_signed = false;
	if spec.alternate {
		match want {
			Want::Int("x") => {
				flags.push('#');
				needs_int = true;
			}
			// `#` changes nothing for these, as in `format!`
			Want::Plain | Want::Int("d") | Want::Str => {}
			Want::Int(_) => errors.push("pbl_fmt_write! has `#` only for `{:#x}`".into()),
		}
	}
	if spec.sign == Some(Sign::Plus) {
		match want {
			Want::Plain | Want::Int("d") => {
				flags.push('+');
				needs_int = true;
				needs_signed = true;
			}
			_ => errors.push("pbl_fmt_write! has `+` only for decimal integers".into()),
		}
	}
	// `0` overrides alignment, as in `format!`
	if spec.zero_pad {
		flags.push('0');
		needs_int = true;
	} else if spec.align == Alignment::AlignLeft {
		flags.push('-');
	}
	if needs_int && want == Want::Str {
		errors.push("pbl_fmt_write!: `+`, `#` and `0` are for integers, not `{:s}`".into());
	}

	if !errors.is_empty() {
		return Err(errors);
	}
	Ok(Spec {
		left_default: !width.is_empty() && spec.align == Alignment::AlignUnknown && !spec.zero_pad,
		flags,
		width,
		needs_int,
		needs_signed,
	})
}

/// The conversion's format pieces: fixed if `kind` is known, otherwise chosen in the format's
/// const block by `arg`'s type
fn parts(spec: &Spec, want: Want, kind: Option<Kind>, arg: &TokenStream) -> Vec<TokenStream> {
	let head = Literal::byte_string(format!("%{}", spec.flags).as_bytes());
	let width = Literal::byte_string(spec.width.as_bytes());
	let str_left = if spec.left_default {
		quote!(b"-")
	} else {
		quote!(b"")
	};
	let int_letter = match want {
		Want::Int(letter) if letter != "d" => {
			let letter = Literal::byte_string(letter.as_bytes());
			quote!(#letter)
		}
		_ => quote!(if #arg::SIGNED { b"d" } else { b"u" }),
	};
	match kind {
		Some(Kind::Str) => vec![quote!(#head), str_left, quote!(#width), quote!(b".*s")],
		Some(Kind::Int) => vec![
			quote!(#head),
			quote!(#width),
			quote!(#arg::MODIFIER),
			int_letter,
		],
		None => vec![
			quote!(#head),
			quote!(if #arg::IS_STR { #str_left } else { b"" }),
			quote!(#width),
			quote!(if #arg::IS_STR { b".*s" } else { #arg::MODIFIER }),
			quote!(if #arg::IS_STR { b"" } else { #int_letter }),
		],
	}
}

/// The format's length bound for a conversion with `spec`
fn part_bound(spec: &Spec) -> usize {
	1 + spec.flags.len() + 1 + spec.width.len() + CONVERSION_BOUND
}

/// The const checks that `arg`'s type takes the conversion
fn asserts(
	spec: &Spec,
	want: Want,
	label: &str,
	arg: &TokenStream,
	name: &str,
) -> Vec<TokenStream> {
	let check = |cond: TokenStream, msg: String| {
		let msg = escape(&msg.replace("pbl_fmt_write!", name));
		quote!(assert!(#cond, #msg);)
	};
	let mut out = Vec::new();
	match want {
		Want::Str => out.push(check(
			quote!(#arg::IS_STR),
			format!("pbl_fmt_write!: {label} isn't a string, for `{{:s}}`"),
		)),
		Want::Int(letter) => out.push(check(
			quote!(!#arg::IS_STR),
			format!("pbl_fmt_write!: {label} isn't an integer, for `{{:{letter}}}`"),
		)),
		Want::Plain if spec.needs_int => out.push(check(
			quote!(!#arg::IS_STR),
			format!("pbl_fmt_write!: {label} isn't an integer, for `+`, `#` or `0`"),
		)),
		Want::Plain => {}
	}
	if spec.needs_signed {
		out.push(check(
			quote!(#arg::SIGNED),
			format!("pbl_fmt_write!: {label} isn't a signed integer, for `+`"),
		));
	}
	out
}

/// Warn about, or refuse, `count` values only plain `{}`s read
fn check_ambiguous(
	count: usize,
	format: &LitStr,
	name: &str,
	errs: &mut darling::error::Accumulator,
) {
	if count >= MAX_AMBIGUOUS {
		errs.push(
			DarlingError::custom(format!(
				"{name}: {count} arguments only formatted with `{{}}`; at most {} can be, as \
				 each doubles the code compiled. Mark strings `{{:s}}` and integers `{{:d}}`",
				MAX_AMBIGUOUS - 1
			))
			.with_span(format),
		);
	} else if count >= WARN_AMBIGUOUS && proc_macro::is_available() {
		proc_macro::Diagnostic::spanned(
			format.span().unwrap(),
			proc_macro::Level::Warning,
			format!(
				"{name}: {count} arguments only formatted with `{{}}` double the code compiled \
				 for each; mark strings `{{:s}}` and integers `{{:d}}` to cut build time"
			),
		)
		.emit();
	}
}

fn expand(
	Input {
		target,
		format,
		args,
	}: Input,
	name: &str,
	errs: &mut darling::error::Accumulator,
) -> TokenStream {
	// Messages name pbl_fmt_write!, the macro the format language is for
	let err =
		|msg: String| DarlingError::custom(msg.replace("pbl_fmt_write!", name)).with_span(&format);
	let source = format.value();
	let mut parser = Parser::new(&source, None, None, false, ParseMode::Format);
	let pieces: Vec<Piece> = parser.by_ref().collect();
	for e in &parser.errors {
		errs.push(err(format!(
			"invalid format string: {}: {}",
			e.description, e.label
		)));
	}

	let mut values = Values::new(&args, format.span());
	if let Some(late) = values.late_positional() {
		errs.push(
			DarlingError::custom("positional arguments must come before named ones")
				.with_span(late),
		);
	}

	// Pass 1: each conversion's value and what it asks for, so every value's fixed kind is known
	// before pass 2 uses it
	let mut resolved: Vec<Option<(usize, Want)>> = Vec::new();
	for piece in &pieces {
		let conversion = match piece {
			Piece::Lit(_) => None,
			Piece::NextArgument(arg) => values
				.slot(&arg.position)
				.and_then(|slot| Ok((slot, want(arg.format.ty)?)))
				.and_then(|(slot, want)| {
					want.kind().map_or(Ok(()), |kind| values.fix(slot, kind))?;
					Ok((slot, want))
				})
				.map_err(|e| errs.push(err(e)))
				.ok(),
		};
		resolved.push(conversion);
	}
	for unused in values.unused() {
		errs.push(DarlingError::custom("argument never used").with_span(unused));
	}

	// Values only plain `{}`s read, whose type picks their conversion
	let ambiguous: Vec<usize> = (0..values.exprs.len())
		.filter(|&v| {
			values.kinds[v].is_none()
				&& resolved
					.iter()
					.flatten()
					.any(|&(i, w)| i == v && w == Want::Plain)
		})
		.collect();
	check_ambiguous(ambiguous.len(), &format, name, errs);

	// Pass 2: the format's parts and checks, and each conversion's value
	let mut parts: Vec<TokenStream> = Vec::new();
	let mut checks: Vec<TokenStream> = Vec::new();
	let mut uses: Vec<usize> = Vec::new();
	let mut bound = 1;
	for (piece, resolved) in pieces.iter().zip(&resolved) {
		match (piece, *resolved) {
			(Piece::Lit(lit), _) => {
				if lit.contains('\0') {
					errs.push(err("pbl_fmt_write! text can't contain NUL".into()));
				}
				let escaped = lit.replace('%', "%%");
				bound += escaped.len();
				let bytes = Literal::byte_string(escaped.as_bytes());
				parts.push(quote!(#bytes));
			}
			(Piece::NextArgument(arg), Some((slot, want))) => match spec(&arg.format, want) {
				Ok(spec) => {
					let ty = format_ident!("A{slot}");
					let arg_ty = quote!(<#ty as ::gneiss::fmt::Arg>);
					let kind = want.kind().or(values.kinds[slot]);
					parts.extend(self::parts(&spec, want, kind, &arg_ty));
					checks.extend(asserts(&spec, want, &values.labels[slot], &arg_ty, name));
					bound += part_bound(&spec);
					uses.push(slot);
				}
				Err(errors) => errors.into_iter().for_each(|e| errs.push(err(e))),
			},
			// Its value didn't resolve, already reported
			(Piece::NextArgument(_), None) => {}
		}
	}

	emit(&target, &values, &ambiguous, &uses, &parts, &checks, bound)
}

/// The macro's output: a function formatting the values to the target, called with each argument
/// evaluated once, up front, as `format!` evaluates them
fn emit(
	target: &Target,
	values: &Values<'_>,
	ambiguous: &[usize],
	uses: &[usize],
	parts: &[TokenStream],
	checks: &[TokenStream],
	bound: usize,
) -> TokenStream {
	let count = values.exprs.len();
	let exprs = &values.exprs;
	let generics: Vec<Ident> = (0..count).map(|i| format_ident!("A{i}")).collect();
	let params: Vec<Ident> = (0..count).map(|i| format_ident!("a{i}")).collect();
	let bound = Literal::usize_unsuffixed(bound);
	let site = Span::mixed_site();
	let go = Ident::new("__pbl_fmt", site);
	let bindings: Vec<Ident> = (0..count)
		.map(|i| Ident::new(&format!("arg{i}"), site))
		.collect();
	let format_fn = |head_params: TokenStream,
	                 ret: TokenStream,
	                 func: TokenStream,
	                 head: TokenStream| {
		let call = call_tree(
			ambiguous,
			&mut values.kinds.clone(),
			uses,
			&params,
			&generics,
			&func,
			&head,
		);
		quote! {
			#[inline(always)]
			#[allow(non_snake_case)]
			fn #go<#(#generics: ::gneiss::fmt::Arg + ?Sized),*>(#head_params #(, #params: &#generics)*) #ret {
				// Type checks, at build time: apps' lints against assert! are for runtime ones
				#[allow(clippy::disallowed_macros)]
				const { #(#checks)* };
				let fmt: &'static [u8; #bound] = &const { ::gneiss::fmt::__build::<#bound>(&[#(#parts),*]) };
				#call
			}
		}
	};

	match target {
		Target::Write { dest } => {
			let buf = Ident::new("buf", site);
			let format_fn = format_fn(
				quote!(#buf: &mut [u8]),
				quote!(-> i32),
				quote!(::gneiss::sys::snprintf),
				quote!(#buf.as_mut_ptr().cast(), #buf.len() as u32),
			);
			quote! {{
				#format_fn
				// Method syntax, as write! uses, reborrows a `&mut` destination without it being bound
				// `mut`
				use ::gneiss::fmt::Sink as _;
				// The fill can run twice
				match (#(&(#exprs),)*) {
					(#(#bindings,)*) => {
						if (#dest).append_formatted(|#buf| #go(#buf #(, #bindings)*)) {
							::core::result::Result::<(), ::core::fmt::Error>::Ok(())
						} else {
							::core::result::Result::<(), ::core::fmt::Error>::Err(::core::fmt::Error)
						}
					}
				}
			}}
		}
		Target::Log { level, file, line } => {
			let (lvl, fl, ln) = (
				Ident::new("level", site),
				Ident::new("file", site),
				Ident::new("line", site),
			);
			let format_fn = format_fn(
				quote!(#lvl: u8, #fl: *const ::core::ffi::c_char, #ln: i32),
				quote!(),
				quote!(::gneiss::sys::app_log),
				quote!(#lvl, #fl, #ln),
			);
			quote! {{
				#format_fn
				match (#(&(#exprs),)*) {
					(#(#bindings,)*) => #go(#level, (#file).as_ptr().cast(), #line #(, #bindings)*),
				}
			}}
		}
	}
}

/// The call to `func`, `snprintf` or `app_log`, after its `head` arguments, branching on each value in `ambiguous` being a string or an integer, so
/// each branch passes a fixed set of values. The branches not taken compile away.
fn call_tree(
	ambiguous: &[usize],
	kinds: &mut Vec<Option<Kind>>,
	uses: &[usize],
	params: &[Ident],
	generics: &[Ident],
	func: &TokenStream,
	head: &TokenStream,
) -> TokenStream {
	if let Some((&first, rest)) = ambiguous.split_first() {
		let ty = &generics[first];
		kinds[first] = Some(Kind::Str);
		let as_str = call_tree(rest, kinds, uses, params, generics, func, head);
		kinds[first] = Some(Kind::Int);
		let as_int = call_tree(rest, kinds, uses, params, generics, func, head);
		kinds[first] = None;
		return quote! {
			if <#ty as ::gneiss::fmt::Arg>::IS_STR { #as_str } else { #as_int }
		};
	}
	let mut stmts = Vec::new();
	let mut values = Vec::new();
	for (k, &index) in uses.iter().enumerate() {
		let param = &params[index];
		if kinds[index] == Some(Kind::Str) {
			let (len, ptr) = (format_ident!("v{k}_len"), format_ident!("v{k}_ptr"));
			stmts.push(quote!(let (#len, #ptr) = ::gneiss::fmt::Arg::str(#param);));
			values.push(quote!(#len));
			values.push(quote!(#ptr));
		} else {
			let v = format_ident!("v{k}");
			stmts.push(quote!(let #v = ::gneiss::fmt::Arg::int(#param);));
			values.push(quote!(#v));
		}
	}
	// SAFETY (of the generated code): `fmt` is __build's output, NUL-padded past its end, and each
	// value is what its conversion reads, which `Arg`, an unsafe trait, guarantees
	quote! {{
		#(#stmts)*
		unsafe { #func(#head, fmt.as_ptr().cast() #(, #values)*) }
	}}
}

/// `msg` as a panic message for `assert!`, which reads it as a format string
fn escape(msg: &str) -> String {
	msg.replace('{', "{{").replace('}', "}}")
}

#[cfg(test)]
mod tests {
	use super::*;

	/// The spec of the format's only conversion
	fn parsed(format: &str, check: impl FnOnce(&FormatSpec<'_>)) {
		let pieces: Vec<Piece> =
			Parser::new(format, None, None, false, ParseMode::Format).collect();
		let [Piece::NextArgument(arg)] = &pieces[..] else {
			panic!("{format} isn't one conversion");
		};
		check(&arg.format);
	}

	fn ok(format: &str, want: Want) -> Spec {
		let mut out = None;
		parsed(format, |s| out = Some(spec(s, want).expect(format)));
		out.unwrap()
	}

	fn rejected(format: &str, want: Want) {
		parsed(format, |s| {
			assert!(spec(s, want).is_err(), "{format} accepted")
		});
	}

	#[test]
	fn plain_width_pads_strings_left() {
		let s = ok("{:5}", Want::Plain);
		assert_eq!(
			(s.flags.as_str(), s.width.as_str(), s.left_default),
			("", "5", true)
		);
	}

	#[test]
	fn explicit_right_doesnt_pad_left() {
		let s = ok("{:>5}", Want::Plain);
		assert_eq!((s.flags.as_str(), s.left_default), ("", false));
	}

	#[test]
	fn zero_overrides_left() {
		let s = ok("{:<05}", Want::Plain);
		assert_eq!(
			(s.flags.as_str(), s.left_default, s.needs_int),
			("0", false, true)
		);
	}

	#[test]
	fn plus_needs_signed() {
		let s = ok("{:+}", Want::Plain);
		assert_eq!(
			(s.flags.as_str(), s.needs_int, s.needs_signed),
			("+", true, true)
		);
	}

	#[test]
	fn alternate_hex() {
		assert_eq!(ok("{:#x}", Want::Int("x")).flags, "#");
		rejected("{:#X}", Want::Int("X"));
		rejected("{:#o}", Want::Int("o"));
	}

	#[test]
	fn rejects_what_snprintf_cant_do() {
		rejected("{:^5}", Want::Plain);
		rejected("{:*>5}", Want::Plain);
		rejected("{:.2}", Want::Plain);
		rejected("{:1$}", Want::Plain);
		rejected("{:x?}", Want::Plain);
		rejected("{:+x}", Want::Int("x"));
		rejected("{:05s}", Want::Str);
	}

	#[test]
	fn type_letters() {
		assert_eq!(want(""), Ok(Want::Plain));
		assert_eq!(want("s"), Ok(Want::Str));
		assert_eq!(want("d"), Ok(Want::Int("d")));
		assert!(want("?").is_err());
		assert!(want("e").is_err());
	}
}
