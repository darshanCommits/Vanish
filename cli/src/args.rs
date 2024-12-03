use std::path::PathBuf;

use clap::{command, Command, Parser, Subcommand};

use crate::chunk_type::ChunkType;

#[derive(Parser)]
#[command(
	name = "Vanish",
	version = "0.1",
	about = "A tool for encoding, decoding, and managing PNG metadata"
)]
struct Cli {
	#[arg(short, long, action = clap::ArgAction::Count)]
	debug: u8,

	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	/// Encode data in a png.
	/// `chunk_type` double as label to refer the hidden data.
	/// Optionally define a output file which creates a new file instead of replacing the orignal.
	Encode {
		file: PathBuf,
		chunk_type: ChunkType,
		message: String,
		output_file: Option<PathBuf>,
	},
	/// Encode data in a png.
	/// use `chunk_type` to refer to the hidden message.
	Decode {
		file: PathBuf,
		chunk_type: ChunkType,
	},
	/// Remove a chunk from a png.
	/// Must provide the `chunk_type` which act as label.
	Remove {
		file: PathBuf,
		chunk_type: ChunkType,
	},
	/// Displays PNG in bytes form.
	Print { file: PathBuf },
}
