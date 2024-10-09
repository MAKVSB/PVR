//! Run this file with `cargo test --test 01_read_after_eof`.

// while reading.
mod os {
    // Simulate a single byte of data being read from the OS for the passed file descriptor.
    // When this function returns `0`, it marks end-of-file.
    pub fn read(_: u32) -> u8 {
        // It doesn't matter what is actually returned here, only the function signatures are
        // important in this example.
        0
    }
}

// Library code
struct OpenedFile {
    // File descriptor
    fd: u32,
}

enum ReadResult {
    Data(u8, OpenedFile),
    EndOfFile,
}

// Implement this function in a way that when the file reaches end-of-file (there is nothing
// else to read), it will not be possible to use it anymore (such usage should result in a
// compile-time error).
fn read(file: OpenedFile) -> ReadResult {
    let byte = os::read(file.fd);
    if byte == 0 {
        return ReadResult::EndOfFile;
    }
    ReadResult::Data(byte, file)
}
// End of library code

// User code
fn main() {
    let mut file = OpenedFile { fd: 1 };

    loop {
        match read(file) {
            ReadResult::Data(data, f) => {
                println!("{}", data);
                file = f;
            },
            ReadResult::EndOfFile => {
                break;
            }
        };
    }
}
