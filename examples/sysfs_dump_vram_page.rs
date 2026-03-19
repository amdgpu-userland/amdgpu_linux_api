use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::process;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <file_path> <offset>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];
    let offset: u64 = args[2].parse().expect("Offset must be a valid u64");

    const PAGE_SIZE: usize = 4096;
    const BYTES_PER_LINE: usize = 64; // Widened for less scrolling
    let mut buffer = vec![0u8; PAGE_SIZE];

    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(offset))?;

    let bytes_read = reader.read(&mut buffer)?;

    println!(
        "File: {} | Offset: {} | Bytes: {}",
        file_path, offset, bytes_read
    );
    println!("{:=<110}", ""); // Longer separator line

    for (i, chunk) in buffer[..bytes_read].chunks(BYTES_PER_LINE).enumerate() {
        // Print the current file offset in hex
        let current_offset = offset + (i * BYTES_PER_LINE) as u64;
        print!("{:010x}: ", current_offset);

        // Print hex bytes in two blocks of 16 for readability
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02x} ", byte);
            if (j + 1) % 16 == 0 && j + 1 != BYTES_PER_LINE {
                print!("  "); // Extra gap in the middle
            }
        }

        // Padding for the final line if it's incomplete
        if chunk.len() < BYTES_PER_LINE {
            let missing = BYTES_PER_LINE - chunk.len();
            for j in 0..missing {
                print!("   ");
                if (chunk.len() + j + 1) % 16 == 0 && (chunk.len() + j + 1) != BYTES_PER_LINE {
                    print!("  ");
                }
            }
        }

        // Print ASCII sidebar
        print!(" | ");
        for &byte in chunk {
            if byte.is_ascii_graphic() || byte == b' ' {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!(" |");
    }

    Ok(())
}
