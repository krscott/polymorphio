use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

const STDIO_FILENAME: &str = "-";

pub enum FileOrStdin {
    File(File),
    Stdin(io::Stdin),
}

pub enum FileOrStdinLock<'a> {
    FileBufReader(BufReader<&'a File>),
    StdinLock(io::StdinLock<'a>),
}

impl FileOrStdin {
    pub fn from_path(path: &PathBuf) -> io::Result<Self> {
        Ok(if path.to_string_lossy() == STDIO_FILENAME {
            io::stdin().into()
        } else {
            File::open(path)?.into()
        })
    }

    #[allow(dead_code)]
    pub fn new<T: Into<Self>>(handle: T) -> Self {
        handle.into()
    }

    pub fn lock<'a>(&'a mut self) -> FileOrStdinLock<'a> {
        match self {
            Self::File(file) => FileOrStdinLock::FileBufReader(BufReader::new(file)),
            Self::Stdin(stdin) => FileOrStdinLock::StdinLock(stdin.lock()),
        }
    }
}

impl From<File> for FileOrStdin {
    fn from(file: File) -> Self {
        Self::File(file)
    }
}

impl From<io::Stdin> for FileOrStdin {
    fn from(stdin: io::Stdin) -> Self {
        Self::Stdin(stdin)
    }
}

impl<'a> Read for FileOrStdinLock<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::FileBufReader(reader) => reader.read(buf),
            Self::StdinLock(lock) => lock.read(buf),
        }
    }
}

impl<'a> BufRead for FileOrStdinLock<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self {
            Self::FileBufReader(reader) => reader.fill_buf(),
            Self::StdinLock(lock) => lock.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Self::FileBufReader(reader) => reader.consume(amt),
            Self::StdinLock(lock) => lock.consume(amt),
        }
    }
}

pub enum FileOrStdout {
    File(File),
    Stdout(io::Stdout),
}

pub enum FileOrStdoutLock<'a> {
    FileBufWriter(BufWriter<&'a File>),
    StdoutLock(io::StdoutLock<'a>),
}

impl FileOrStdout {
    pub fn from_path(path: &PathBuf) -> io::Result<Self> {
        Ok(if path.to_string_lossy() == STDIO_FILENAME {
            io::stdout().into()
        } else {
            File::create(path)?.into()
        })
    }

    #[allow(dead_code)]
    pub fn new<T: Into<Self>>(handle: T) -> Self {
        handle.into()
    }

    pub fn lock<'a>(&'a mut self) -> FileOrStdoutLock<'a> {
        match self {
            Self::File(file) => FileOrStdoutLock::FileBufWriter(BufWriter::new(file)),
            Self::Stdout(stdout) => FileOrStdoutLock::StdoutLock(stdout.lock()),
        }
    }
}

impl From<File> for FileOrStdout {
    fn from(file: File) -> Self {
        Self::File(file)
    }
}

impl From<io::Stdout> for FileOrStdout {
    fn from(stdout: io::Stdout) -> Self {
        Self::Stdout(stdout)
    }
}

impl<'a> Write for FileOrStdoutLock<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::FileBufWriter(file) => file.write(buf),
            Self::StdoutLock(stdout) => stdout.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::FileBufWriter(file) => file.flush(),
            Self::StdoutLock(stdout) => stdout.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
