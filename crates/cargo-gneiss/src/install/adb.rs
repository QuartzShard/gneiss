//! Reach the Pebble app on an Android device through the adb server, without running `adb`.
//!
//! The server listens on 127.0.0.1:5037. Each request is its length in 4 hex digits, then its
//! text, answered by `OKAY`, or `FAIL` and a message. Once `host:transport:<serial>` picks a
//! device, the connection becomes the stream of the next service it opens: `shell:` for the
//! broadcast that starts the app's developer connection, `tcp:<port>` for the connection itself.
use std::{
	io::{ErrorKind, Read, Write},
	net::{Ipv4Addr, SocketAddr, TcpStream},
	time::Duration,
};

use color_eyre::{
	Result,
	eyre::{Context, bail, eyre},
};

const SERVER: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 5037);
const BROADCAST: &str = "shell:am broadcast -a coredevices.coreapp.DEV_CONNECTION \
                         -n coredevices.coreapp/coredevices.coreapp.debug.DevConnectionReceiver";
/// The app gives a restart of its developer connection 10 s
const BROADCAST_TIMEOUT: Duration = Duration::from_secs(20);

/// A stream to the developer connection of the Pebble app on `device`, as `adb devices` names it,
/// or on the one device attached. Returns the stream and the device it chose.
pub fn open(device: Option<&str>) -> Result<(TcpStream, String)> {
	let serial = pick_device(device)?;
	let port = start_dev_connection(&serial)?;
	let mut stream = transport(&serial)?;
	request(&mut stream, &format!("tcp:{port}"))
		.wrap_err_with(|| format!("Can't reach port {port} on {serial}"))?;
	Ok((stream, serial))
}

fn server() -> Result<TcpStream> {
	TcpStream::connect(SERVER).map_err(|e| {
		if e.kind() == ErrorKind::ConnectionRefused {
			eyre!("No adb server on {SERVER}: start it with `adb start-server`")
		} else {
			eyre!("Can't reach the adb server on {SERVER}: {e}")
		}
	})
}

fn request(stream: &mut TcpStream, request: &str) -> Result<()> {
	stream.write_all(format!("{:04x}{request}", request.len()).as_bytes())?;
	let mut status = [0; 4];
	stream
		.read_exact(&mut status)
		.wrap_err("The adb server hung up")?;
	match &status {
		b"OKAY" => Ok(()),
		b"FAIL" => bail!("adb: {}", read_string(stream)?),
		_ => bail!(
			"The adb server answered {:?}",
			String::from_utf8_lossy(&status)
		),
	}
}

/// A length-prefixed string from the server.
fn read_string(stream: &mut TcpStream) -> Result<String> {
	let mut len = [0; 4];
	stream.read_exact(&mut len)?;
	let len = std::str::from_utf8(&len)
		.ok()
		.and_then(|len| usize::from_str_radix(len, 16).ok())
		.ok_or_else(|| eyre!("The adb server sent a bad length: {len:?}"))?;
	let mut text = vec![0; len];
	stream.read_exact(&mut text)?;
	Ok(String::from_utf8_lossy(&text).into_owned())
}

fn transport(serial: &str) -> Result<TcpStream> {
	let mut stream = server()?;
	request(&mut stream, &format!("host:transport:{serial}"))?;
	Ok(stream)
}

fn pick_device(device: Option<&str>) -> Result<String> {
	let mut stream = server()?;
	request(&mut stream, "host:devices")?;
	let list = read_string(&mut stream)?;
	let devices = parse_devices(&list);
	if let Some(device) = device {
		return match devices.iter().find(|(serial, _)| *serial == device) {
			Some((_, "device")) => Ok(device.to_owned()),
			Some((_, state)) => bail!("{device} is {state}, not ready for adb"),
			None => bail!("No device {device} attached to adb"),
		};
	}
	let ready: Vec<&str> = devices
		.iter()
		.filter(|(_, state)| *state == "device")
		.map(|(serial, _)| *serial)
		.collect();
	match ready[..] {
		[serial] => Ok(serial.to_owned()),
		[] if devices.is_empty() => bail!("No devices attached to adb"),
		[] => bail!(
			"No device ready for adb: {}",
			devices
				.iter()
				.map(|(serial, state)| format!("{serial} is {state}"))
				.collect::<Vec<_>>()
				.join(", ")
		),
		_ => bail!(
			"Several devices attached to adb ({}): pick one with --adb=DEVICE",
			ready.join(", ")
		),
	}
}

/// `(serial, state)` for each line of a `host:devices` reply.
fn parse_devices(list: &str) -> Vec<(&str, &str)> {
	list.lines()
		.filter_map(|line| line.split_once('\t'))
		.collect()
}

/// Ask the Pebble app to start its developer connection, and return the port it's on.
fn start_dev_connection(serial: &str) -> Result<u16> {
	let mut stream = transport(serial)?;
	stream.set_read_timeout(Some(BROADCAST_TIMEOUT))?;
	request(&mut stream, BROADCAST)?;
	let mut output = String::new();
	stream
		.read_to_string(&mut output)
		.wrap_err("No answer from the Pebble app's developer connection broadcast")?;
	parse_broadcast(&output)
}

/// The port from `am broadcast`'s `result=N, data="…"`, or the app's reason it failed.
fn parse_broadcast(output: &str) -> Result<u16> {
	let Some((_, rest)) = output.split_once("result=") else {
		bail!(
			"Can't make sense of the Pebble app's answer: {}",
			output.trim()
		);
	};
	let code = rest
		.split(|c: char| !(c == '-' || c.is_ascii_digit()))
		.next()
		.unwrap_or_default();
	let data = rest
		.split_once("data=\"")
		.and_then(|(_, data)| data.rfind('"').map(|end| &data[..end]));
	match (code, data) {
		("0", Some(port)) => port
			.parse()
			.wrap_err_with(|| format!("The Pebble app gave a bad port: {port:?}")),
		("0", None) => bail!(
			"Nothing answered the developer connection broadcast: is the Pebble app installed and \
			 up to date?"
		),
		(code, data) => bail!(
			"The Pebble app couldn't start its developer connection: {}",
			data.map_or_else(|| format!("error {code}"), str::to_owned)
		),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn devices() {
		assert_eq!(
			parse_devices("R58M12ABCDE\tdevice\n192.168.1.5:5555\tunauthorized\n"),
			[
				("R58M12ABCDE", "device"),
				("192.168.1.5:5555", "unauthorized")
			]
		);
		assert!(parse_devices("").is_empty());
	}

	#[test]
	fn broadcast() {
		let ok = "Broadcasting: Intent { act=coredevices.coreapp.DEV_CONNECTION flg=0x400000 }\r\n\
		          Broadcast completed: result=0, data=\"9000\"\r\n";
		assert_eq!(parse_broadcast(ok).unwrap(), 9000);
		let failed = "Broadcast completed: result=1, data=\"no watch connected\"\n";
		assert!(
			parse_broadcast(failed)
				.unwrap_err()
				.to_string()
				.ends_with("no watch connected")
		);
		let unhandled = "Broadcast completed: result=0\n";
		assert!(
			parse_broadcast(unhandled)
				.unwrap_err()
				.to_string()
				.starts_with("Nothing answered")
		);
		assert!(parse_broadcast("Error: unknown").is_err());
	}
}
