//! Rewrites the SDK headers' Doxygen comments as rustdoc Markdown, as bindgen generates them.
use bindgen::callbacks::ParseCallbacks;

#[derive(Debug)]
pub struct DoxygenComments;

impl ParseCallbacks for DoxygenComments {
	fn process_comment(&self, comment: &str) -> Option<String> {
		Some(to_markdown(comment))
	}
}

/// Commands that open a paragraph of their own, and what they become.
const SECTIONS: &[(&str, &str)] = &[
	("@return", "Returns"),
	("@returns", "Returns"),
	("\\return", "Returns"),
	("@note", "Note:"),
	("\\note", "Note:"),
	("@warning", "Warning:"),
	("@see", "See"),
	("\\sa", "See"),
	("@deprecated", "Deprecated:"),
];

/// Lines that only carry Doxygen structure.
const DROPPED_LINES: &[&str] = &["@addtogroup", "@{", "@}", "@file", "@generated"];

fn to_markdown(comment: &str) -> String {
	let mut out: Vec<String> = Vec::new();
	let mut in_code = false;
	for line in comment.lines() {
		// bindgen trims the `//` of a `//!` comment, not the `!`
		let line = line.strip_prefix('!').unwrap_or(line);
		if !in_code && let Some((prose, _)) = line.split_once("\\code")
			&& !prose.trim().is_empty()
		{
			out.push(format!(" {}", rewrite_words(prose)));
			out.push(" ```c".into());
			in_code = true;
			continue;
		}
		let first = line.split_whitespace().next().unwrap_or("");
		if first.starts_with("\\code") {
			in_code = true;
			out.push(" ```c".into());
			continue;
		}
		if first == "\\endcode" {
			in_code = false;
			out.push(" ```".into());
			continue;
		}
		if in_code {
			out.push(line.into());
			continue;
		}
		if DROPPED_LINES.contains(&first) || first.starts_with("![](") {
			continue;
		}

		if let Some(heading) = html_heading(line) {
			out.push(heading);
			continue;
		}

		let is_param = first.starts_with("@param") || first.starts_with("\\param");
		let is_section = SECTIONS.iter().any(|(cmd, _)| *cmd == first);
		let after_list = out.last().is_some_and(|l| l.starts_with(" * "));
		let after_blank = out.last().is_none_or(|l| l.trim().is_empty());
		if (is_section || (is_param && !after_list)) && !after_blank {
			out.push(String::new());
		}
		out.push(format!(" {}", rewrite_words(line)));
	}

	let blank = |l: &String| l.trim().is_empty();
	while out.last().is_some_and(blank) {
		out.pop();
	}
	let start = out.iter().position(|l| !blank(l)).unwrap_or(out.len());
	out[start..].join("\n")
}

/// A `<h3>`/`<h4>` line as a Markdown heading.
fn html_heading(line: &str) -> Option<String> {
	let line = line.trim();
	["h3", "h4"].iter().zip(["###", "####"]).find_map(|(tag, hashes)| {
		let text = line
			.strip_prefix(&format!("<{tag}>"))?
			.strip_suffix(&format!("</{tag}>"))?;
		Some(format!(" {hashes} {}", rewrite_words(text)))
	})
}

/// One prose line, with each inline command replaced. Whitespace is collapsed, so a continuation
/// line can't turn into an indented code block.
fn rewrite_words(line: &str) -> String {
	let mut out: Vec<String> = Vec::new();
	let mut words = line.split_whitespace();
	while let Some(word) = words.next() {
		let mut next = || words.next().unwrap_or("");
		let (open, word) = word.split_at(word.len() - word.trim_start_matches(['(', '[']).len());
		let word = match word {
			_ if word.starts_with("@param") || word.starts_with("\\param") => {
				format!("* `{}`:", next())
			}
			"\\ref" | "@ref" | "@c" | "\\c" | "@p" => code(next()),
			"{@link" => {
				let (name, rest) = next().split_once('}').unwrap_or(("", ""));
				format!("`{name}`{rest}")
			}
			"\\a" => emphasis(next(), "*"),
			"\\b" => emphasis(next(), "**"),
			"\\htmlinclude" => {
				let page = next();
				let (name, rest) = page.split_once(".html").unwrap_or((page, ""));
				format!("*{name}*{rest}")
			}
			"\\li" => "*".into(),
			"@brief" | "\\brief" | "@internal" | "@{" | "@}" => continue,
			_ => match SECTIONS.iter().find(|(cmd, _)| *cmd == word) {
				Some((_, text)) => (*text).into(),
				None => word.into(),
			},
		};
		out.push(format!("{open}{word}"));
	}
	out.join(" ")
}

/// `name` as inline code, leaving trailing punctuation outside it.
fn code(word: &str) -> String {
	let (name, rest) = split_name(word);
	format!("`{name}`{rest}")
}

fn emphasis(word: &str, marker: &str) -> String {
	let (name, rest) = split_name(word);
	format!("{marker}{name}{marker}{rest}")
}

fn split_name(word: &str) -> (&str, &str) {
	let name = word.trim_end_matches(['.', ',', ';', ':', ')']);
	let name = if word[name.len()..].starts_with(')') && name.ends_with('(') {
		&word[..=name.len()]
	} else {
		name
	};
	word.split_at(name.len())
}
