use std::path::PathBuf;

use clap::command;
use clap::{Parser, Subcommand};

use edpg::chunk_type::ChunkType;

#[derive(Parser)]
#[command(
	name = "Vanish",
	version = "0.1",
	about = "Hide secret information in .png",
	long_about = "A cli for encoding, decoding, and managing PNG metadata"
)]

pub struct Cli {
	#[arg(short, long, action = clap::ArgAction::Count)]
	debug: u8,

	#[command(subcommand)]
	pub command: Commands,
}

// Help me figure out how to avoid repeated documentation here.

#[derive(Subcommand)]
pub enum Commands {
	/// Encode data in a png.
	/// `chunk_type` double as label to refer the hidden data.
	Encode {
		/// Accepts a valid .png file.
		file: PathBuf,
		/// Accepts an exact 4byte ASCII(alphabetic only) sequence. eg: [rust, bOAT].
		chunk_type: ChunkType,
		/// The data you want to hide.
		message: String,
		/// Optionally a output path to store the new encoded png.
		output_file: Option<PathBuf>,
	},
	/// Encode data in a png.
	/// use `chunk_type` to refer to the hidden message.
	Decode {
		/// Accepts a valid .png file.
		file: PathBuf,
		/// Accepts an exact 4byte ASCII(alphabetic only) sequence. eg: [rust, bOAT].
		chunk_type: String,
	},
	/// Remove a chunk from a png.
	/// Must provide the `chunk_type` which act as label.
	Remove {
		/// Accepts a valid .png file.
		file: PathBuf,
		/// Accepts an exact 4byte ASCII(alphabetic only) sequence. eg: [rust, bOAT].
		chunk_type: String,
	},
	/// Displays PNG in bytes form.
	Print {
		/// Accepts a valid .png file.
		file: PathBuf,
	},
}
