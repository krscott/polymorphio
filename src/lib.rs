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

    /// Read the entire contents into a string.
    ///
    /// This is a convenience function similar to
    /// [`std::fs::read_to_string`](https://doc.rust-lang.org/std/fs/fn.read_to_string.html).
    pub fn read_to_string(path: &PathBuf) -> io::Result<String> {
        let mut string = String::new();
        Self::from_path(path)?.lock().read_to_string(&mut string)?;
        Ok(string)
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

    /// Write the entire contents of a buffer to a path.
    ///
    /// This is a convenience function that is the complementary to `FileOrStdin::read_to_string`.
    pub fn write_all(path: &PathBuf, buf: &[u8]) -> io::Result<()> {
        let mut writer = Self::from_path(path)?;
        let mut write_buf = writer.lock();
        write_buf.write_all(buf)
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
    use super::*;
    use std::fs;
    use std::io;
    use tempdir::TempDir;

    fn with_temp_dir<F>(f: F) -> Result<(), io::Error>
    where
        F: FnOnce(&TempDir) -> Result<(), io::Error>,
    {
        let tmp_dir = TempDir::new("test_dir")?;

        f(&tmp_dir)?;

        tmp_dir.close()?;
        Ok(())
    }

    #[test]
    fn read_file() -> Result<(), io::Error> {
        with_temp_dir(|tmp_dir| {
            let expected_content = "Test read file content";
            let test_file_path = tmp_dir.path().join("test_file.txt");
            let mut test_file = File::create(test_file_path.clone())?;
            write!(test_file, "{}", expected_content)?;
            drop(test_file);

            let mut actual_content = String::new();
            FileOrStdin::from_path(&test_file_path)
                .unwrap()
                .lock()
                .read_to_string(&mut actual_content)?;
            assert_eq!(actual_content, expected_content);

            assert_eq!(
                FileOrStdin::read_to_string(&test_file_path).unwrap(),
                expected_content
            );

            Ok(())
        })
    }

    #[test]
    fn write_file() -> Result<(), io::Error> {
        with_temp_dir(|tmp_dir| {
            let expected_content = "Test write file content";

            let test_file_path = tmp_dir.path().join("test_write_file.txt");
            FileOrStdout::from_path(&test_file_path)
                .unwrap()
                .lock()
                .write(expected_content.as_bytes())?;
            let actual_content = fs::read_to_string(test_file_path)?;
            assert_eq!(actual_content, expected_content);

            let test_file_path2 = tmp_dir.path().join("test_write_file2.txt");
            FileOrStdout::write_all(&test_file_path2, expected_content.as_bytes())?;
            let actual_content = fs::read_to_string(test_file_path2)?;
            assert_eq!(actual_content, expected_content);

            Ok(())
        })
    }

    // TODO: stdin/stdout
}
