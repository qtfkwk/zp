use crate::*;

#[derive(Debug)]
pub struct Zip {
    path: Option<PathBuf>,
    entries: Entries,
}

impl Zip {
    /// Process a zip file at path
    pub fn from<P>(path: P) -> Result<Self, String>
    where
        P: Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        if !path.exists() {
            return Err(format!("Path does not exist: `{}`", path.display()));
        } else if !path.is_file() {
            return Err(format!("Path is not a file: `{}`", path.display()));
        }
        match File::open(&path) {
            Ok(f) => {
                let mut r = BufReader::new(f);
                let mut zip = Zip::process(&mut r)?;
                zip.path = Some(path);
                Ok(zip)
            }
            Err(e) => Err(format!("{e}: `{}`", path.display())),
        }
    }

    /// Process the raw bytes of a zip file
    pub fn process<R>(r: &mut BufReader<R>) -> Result<Self, String>
    where
        R: Read + Seek,
    {
        match r.read_le::<Entries>() {
            Ok(entries) => {
                if entries.list.is_empty() {
                    Err(String::from("Unexpected end of file"))
                } else {
                    Ok(Self {
                        path: None,
                        entries,
                    })
                }
            }
            Err(e) => {
                let e = e.root_cause(); // not the backtrace error

                // Check if the outer error is an `EnumErrors`
                if let Error::EnumErrors {
                    pos: _,
                    variant_errors,
                } = &e
                {
                    // Count the bad magic errors and save the magic value seen
                    let mut count_bad_magic = 0;
                    let mut magic = None;
                    for (_, i) in variant_errors {
                        let i = i.root_cause();
                        if let Error::BadMagic { pos: _, found } = &i {
                            magic = Some(magic_hex(&format!("{:?}", found)));
                            count_bad_magic += 1;
                        }
                    }

                    // If got a magic value and all the variant errors were bad magic errors,
                    // return an invalid signature error.
                    if magic.is_some() && count_bad_magic == variant_errors.len() {
                        return Err(format!("Invalid signature: `{}`", magic.as_ref().unwrap()));
                    }
                }

                // Return the error
                Err(e.to_string())
            }
        }
    }

    /// Helper to call `verbose()` or `summary()` based on the value of `verbose`
    pub fn output(&self, verbose: bool) -> Result<String, String> {
        if verbose {
            self.verbose()
        } else {
            self.summary()
        }
    }

    /// Generate a complete analysis of the zip file contents
    pub fn verbose(&self) -> Result<String, String> {
        let mut s = vec![];
        for entry in &self.entries.list {
            s.push(String::from("---\n"));
            match entry {
                Entry::LocalFile(i) => s.push(i.verbose()),
                Entry::CentralDirectoryFileHeader(i) => s.push(i.verbose()),
                Entry::EndOfCentralDirectoryRecord(i) => s.push(i.verbose()),
            }
        }
        s.push(String::from("---\nEOF\n---\n"));
        Ok(s.join(""))
    }

    /// Generate a summary of the the zip file contents
    /// (file name, whether item is a folder, uncompressed size, modified date/time, and comment)
    pub fn summary(&self) -> Result<String, String> {
        let mut s = vec![];
        for entry in &self.entries.list {
            if let Entry::CentralDirectoryFileHeader(i) = entry {
                s.push(i.summary());
            }
        }
        Ok(s.join(""))
    }
}
