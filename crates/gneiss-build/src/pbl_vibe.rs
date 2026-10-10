//! Compile vibration scores
use std::path::Path;

use color_eyre::{Result, eyre::bail};
use serde::Deserialize;
use serde_json::Value;

const VERSION: u16 = 1;
// Keep in sync with VIBE_SCORE_MAX_REPEAT_DELAY_MS in vibe_score.c; the
// firmware refuses to load scores exceeding it
const MAX_REPEAT_DELAY_MS: u16 = 60000;
const ATTR_NOTE_LIST: u8 = 0x01;
const ATTR_PATTERN: u8 = 0x02;
const ATTR_REPEAT_DELAY: u8 = 0x03;

#[derive(Debug, Deserialize)]
struct VibeScore {
	notes:           Vec<VibeNote>,
	pattern:         Vec<Value>,
	#[serde(default)]
	repeat_delay_ms: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct VibeNote {
	id:                Value,
	vibe_duration_ms:  u16,
	brake_duration_ms: u8,
	strength:          i8,
}

pub(crate) fn process_vibe(file: &Path) -> Result<Vec<u8>> {
	serialise(&serde_json::from_str(&std::fs::read_to_string(file)?)?)
}

fn serialise(score: &VibeScore) -> Result<Vec<u8>> {
	if score.pattern.is_empty() {
		bail!("Vibe score has no pattern");
	}
	if let Some(delay) = score.repeat_delay_ms
		&& delay > MAX_REPEAT_DELAY_MS
	{
		bail!("repeat_delay_ms {delay} out of bounds, max {MAX_REPEAT_DELAY_MS}");
	}
	for note in &score.notes {
		if !(-100..=100).contains(&note.strength) {
			bail!("strength {} out of bounds, -100 to 100 only", note.strength);
		}
	}
	// Duplicate note ids resolve to the last occurrence, matching upstream
	let indices: Vec<u8> = score
		.pattern
		.iter()
		.map(|id| {
			score
				.notes
				.iter()
				.rposition(|note| note.id == *id)
				.and_then(|i| u8::try_from(i).ok())
				.ok_or_else(|| color_eyre::eyre::eyre!("Pattern references unknown note id {id}"))
		})
		.collect::<Result<_>>()?;

	let mut attrs = vec![2 + u8::from(score.repeat_delay_ms.is_some())];
	let mut push_attr = |id: u8, payload: &[u8]| {
		attrs.push(id);
		attrs.extend((payload.len() as u16).to_le_bytes());
		attrs.extend_from_slice(payload);
	};
	let notes: Vec<u8> = score
		.notes
		.iter()
		.flat_map(|note| {
			let mut b = note.vibe_duration_ms.to_le_bytes().to_vec();
			b.push(note.brake_duration_ms);
			b.push(note.strength as u8);
			b
		})
		.collect();
	push_attr(ATTR_NOTE_LIST, &notes);
	push_attr(ATTR_PATTERN, &indices);
	if let Some(delay) = score.repeat_delay_ms {
		push_attr(ATTR_REPEAT_DELAY, &delay.to_le_bytes());
	}

	let mut vibe = b"VIBE".to_vec();
	vibe.extend(VERSION.to_le_bytes());
	vibe.extend([0u8; 4]);
	vibe.extend((attrs.len() as u16).to_le_bytes());
	vibe.extend(attrs);
	Ok(vibe)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn score(json: &str) -> Vec<u8> {
		let score: VibeScore = serde_json::from_str(json).unwrap();
		serialise(&score).unwrap()
	}

	// Expected bytes generated with upstream tools/json2vibe.py
	#[test]
	fn parity_with_upstream_json2vibe() {
		let double_pulse = r#"{"notes": [
			{"id": "short_pulse", "vibe_duration_ms": 15, "brake_duration_ms": 9, "strength": 100},
			{"id": "delay_100", "vibe_duration_ms": 100, "brake_duration_ms": 0, "strength": 0}],
			"pattern": ["short_pulse", "delay_100", "short_pulse"]}"#;
		assert_eq!(
			score(double_pulse),
			hex("564942450100000000001200020108000f00096464000000020300000100")
		);
		let numeric_ids = r#"{"notes": [
			{"id": 17, "vibe_duration_ms": 15, "brake_duration_ms": 9, "strength": 100},
			{"id": "delay_100", "vibe_duration_ms": 100, "brake_duration_ms": 0, "strength": 0}],
			"pattern": [17, "delay_100", 17]}"#;
		assert_eq!(
			score(numeric_ids),
			hex("564942450100000000001200020108000f00096464000000020300000100")
		);
		let repeating = r#"{"notes": [
			{"id": "short_pulse", "vibe_duration_ms": 15, "brake_duration_ms": 9, "strength": 100},
			{"id": "delay_100", "vibe_duration_ms": 100, "brake_duration_ms": 0, "strength": 0}],
			"pattern": ["short_pulse", "delay_100", "short_pulse"], "repeat_delay_ms": 1092}"#;
		assert_eq!(
			score(repeating),
			hex("564942450100000000001700030108000f000964640000000203000001000302004404")
		);
	}

	fn hex(s: &str) -> Vec<u8> {
		(0..s.len())
			.step_by(2)
			.map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
			.collect()
	}
}
