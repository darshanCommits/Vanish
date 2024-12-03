use std::{
	fs::{read, File},
	io::Write,
};

use args::Cli;
use clap::{error::Result, Parser};
use edpg::{chunk::Chunk, png::Png};

pub mod args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Cli::parse();

	match args.command {
		args::Commands::Encode {
			file,
			chunk_type,
			message,
			output_file,
		} => {
			let file_as_bytes = read(&file)?;
			let mut file_as_png = Png::try_from(file_as_bytes.as_ref())?;

			let new_data = Chunk::new(chunk_type, message.into_bytes());

			if let Some(x) = output_file {
				let mut copy = file_as_png.clone();
				copy.append_chunk(new_data);
				let mut new_file = File::create(x)?;
				new_file.write_all(copy.as_bytes().as_ref())?;
			} else {
				file_as_png.append_chunk(new_data);
				let mut original_file = File::create(&file)?;
				original_file.write_all(&file_as_png.as_bytes())?;
			}
		},

		args::Commands::Decode { file, chunk_type } => {
			let png = Png::try_from(file)?;

			let idx = png
				.find_by_chunk(&chunk_type)
				.expect("Failed to find such chunk");

			let msg = png.chunks().get(idx).expect("Nothing here!");

			println!("{}", msg);
		},

		args::Commands::Remove { file, chunk_type } => {
			let mut png = Png::try_from(file)?;

			let popped = png.remove_first_chunk(&chunk_type)?;
			println!("{popped}");
		},
		args::Commands::Print { file } => {
			let png = Png::try_from(file)?;
			println!("{png}");
		},
	};

	Ok(())
}
