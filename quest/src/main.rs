#![allow(unused)]
#![allow(clippy::all, clippy::nursery, clippy::pedantic)]

#[macro_use]
use quest;
use quest::parse::ast::Compile;
use quest::parse::{token::*, *};
use quest::value::ty::*;
use quest::value::*;
use quest::vm::*;
use quest::Result;
use std::path::Path;

fn run_code(code: &str, filename: Option<&Path>) -> Result<Value> {
	let mut parser = Parser::new(code, filename);
	let mut builder = quest::vm::block::Builder::default();
	let scratch = quest::vm::block::Local::Scratch;

	ast::Group::parse_all(&mut parser).expect("bad parse").compile(&mut builder, scratch);

	builder.build().run(Default::default())
}

fn setup_tracing() {
	use tracing::level_filters::LevelFilter;
	use tracing_subscriber::{layer::SubscriberExt, registry::Registry};

	let loglevel = std::env::var("QUEST_LOGGING");
	let filter = match loglevel.as_ref().map(|x| x.as_ref()) {
		Ok("T") | Ok("TRACE") => LevelFilter::TRACE,
		Ok("D") | Ok("DEBUG") => LevelFilter::DEBUG,
		Ok("I") | Ok("INFO") => LevelFilter::INFO,
		Ok("W") | Ok("WARN") => LevelFilter::WARN,
		Ok("E") | Ok("ERROR") => LevelFilter::ERROR,
		Ok("O") | Ok("OFF") => LevelFilter::OFF,
		_ => LevelFilter::WARN,
	};

	tracing_subscriber::fmt()
		.with_max_level(filter)
		.with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
		.init();
}

fn main() {
	setup_tracing();

	if false {
		run_code(
			r#"fib=n->{(n<=1).then(n.return);fib(n-1)+fib(n-2)};print(fib(30))"#,
			// r#"i=0; while({ i < 5_000_000 }, { :1.i = i + 1 })"#,
			None,
		)
		.unwrap();

		return;
	}

	const USAGE: &str = "usage: -e <expr> | -f <file>";

	let mut args = std::env::args().skip(1);
	let (contents, filename) = match &*args.next().expect(USAGE) {
		"-f" => {
			let name = args.next().expect(USAGE);
			(std::fs::read_to_string(&name).expect("cant open file"), Some(name))
		}
		"-e" => (args.next().expect(USAGE), None),
		_ => panic!("{USAGE}"),
	};

	match run_code(&contents, filename.as_deref().map(Path::new)) {
		Err(err) => {
			eprintln!("error: {err:#}");
			std::process::exit(0)
		}
		Ok(num) => {
			if let Some(exit_code) = num.downcast::<Integer>() {
				std::process::exit(exit_code.get() as i32)
			}
		}
	}
}
