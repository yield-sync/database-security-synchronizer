use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::fmt::Arguments;
use std::sync::RwLock;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel
{
	Error,
	Warn,
	Info,
	Debug,
	Superdebug,
	Ultradebug,
}


impl LogLevel
{
	fn color_code(self) -> &'static str
	{
		match self
		{
			LogLevel::Error => "\x1b[31m", // Red
			LogLevel::Warn => "\x1b[33m", // Yellow
			LogLevel::Info => "\x1b[34m", // Blue
			LogLevel::Debug => "\x1b[36m", // Cyan
			LogLevel::Superdebug => "\x1b[36m", // Cyan
			LogLevel::Ultradebug => "\x1b[36m", // Cyan
		}
	}

	fn weight(self) -> u8
	{
		match self
		{
			LogLevel::Error => 0,
			LogLevel::Warn => 1,
			LogLevel::Info => 2,
			LogLevel::Debug => 3,
			LogLevel::Superdebug => 4,
			LogLevel::Ultradebug => 5,
		}
	}

	fn from_str(s: &str) -> Option<Self> {
		match s.to_ascii_lowercase().as_str() {
			"error" => Some(LogLevel::Error),
			"warn" => Some(LogLevel::Warn),
			"info" => Some(LogLevel::Info),
			"debug" => Some(LogLevel::Debug),
			"superdebug" => Some(LogLevel::Superdebug),
			"ultradebug" => Some(LogLevel::Ultradebug),
			_ => None,
		}
	}
}


static LOG_LEVEL: Lazy<RwLock<LogLevel>> = Lazy::new(
	||
	{
		dotenv().ok();

		let level = std::env::var("LOG_LEVEL").ok().as_deref().and_then(LogLevel::from_str).unwrap_or(LogLevel::Info);

		println!("LOG_LEVEL: {:?}", level);

		RwLock::new(level)
	}
);


	fn log(prefix: &str, level: LogLevel, args: Arguments)
	{
		let current = *LOG_LEVEL.read().unwrap();

		if current.weight() >= level.weight()
		{
			let color = level.color_code();
			let reset = "\x1b[0m";

			// This colors the prefix only.
			// Move {reset} to the end if you want the whole line colored.
			println!("{}{} {} {}", color, prefix, reset, args);
		}
	}


// ---------- public helpers ----------

pub fn ultradebug(args: Arguments)
{
	log("🔧 [UDBG]", LogLevel::Ultradebug, args);
}

pub fn superdebug(args: Arguments)
{
	log("🔧 [SDBG]", LogLevel::Superdebug, args);
}

pub fn debug(args: Arguments)
{
	log("🔧 [DBG]", LogLevel::Debug, args);
}

pub fn info(args: Arguments)
{
	log("🔵 [INF]", LogLevel::Info, args);
}

pub fn warn(args: Arguments)
{
	log("🚨 [WRN]", LogLevel::Warn, args);
}

pub fn error(args: Arguments)
{
	log("❌ [ERR]", LogLevel::Error, args);
}

// ---------- macros for ergonomic usage ----------

#[macro_export]
macro_rules! log_ultradebug
{
	($($arg:tt)*) =>
	{
		$crate::logger::ultradebug(format_args!($($arg)*))
	};
}

#[macro_export]
macro_rules! log_superdebug
{
	($($arg:tt)*) =>
	{
		$crate::logger::superdebug(format_args!($($arg)*))
	};
}

#[macro_export]
macro_rules! log_debug
{
	($($arg:tt)*) =>
	{
		$crate::logger::debug(format_args!($($arg)*))
	};
}

#[macro_export]
macro_rules! log_info
{
	($($arg:tt)*) => {
		$crate::logger::info(format_args!($($arg)*))
	};
}

#[macro_export]
macro_rules! log_warn
{
	($($arg:tt)*) =>
	{
		$crate::logger::warn(format_args!($($arg)*))
	};
}

#[macro_export]
macro_rules! log_error
{
	($($arg:tt)*) =>
	{
		$crate::logger::error(format_args!($($arg)*))
	};
}
