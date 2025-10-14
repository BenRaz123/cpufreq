use std::io::{self, ErrorKind};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

macro_rules! mk_builder {
    ($t:ty, $($idents:ident),*) => {
        impl $t {
            $(
                pub(crate) fn $idents (self, val: bool) -> Self {
                    Self { $idents: val, ..self }
                }
            )*
        }
    };
}

pub(crate) trait Fs {
    type File;
    type DirEnt;

    fn exists(&self, path: &str) -> io::Result<bool>;
    fn open(&self, options: OpenOptions, path: &str) -> io::Result<Self::File>;
    fn read_to_string(&self, f: &Self::File) -> io::Result<String>;
    fn write_to(&mut self, path: &Self::File, content: &[u8]) -> io::Result<()>;
    fn dir(&self, path: &str) -> io::Result<Vec<Self::DirEnt>>;
    fn is_dir(dirent: &Self::DirEnt) -> bool;
    fn is_file(dirent: &Self::DirEnt) -> bool {
        !Self::is_dir(dirent)
    }
}

// Vertical illumination for first one because light cant penetrate the leaf

#[derive(Debug)]
/// no directories allowed, only absolute file paths
pub(crate) struct TestFs(pub(crate) HashMap<Arc<Path>, String>);

#[derive(Debug)]
pub(crate) struct TestFile {
    path: Arc<Path>,
    options: OpenOptions,
}

#[derive(Debug, Default)]
pub(crate) struct OpenOptions {
    pub(crate) read: bool,
    pub(crate) write: bool,
    pub(crate) append: bool,
    pub(crate) create: bool,
}

mk_builder!(OpenOptions, read, write, append, create);

impl From<OpenOptions> for fs::OpenOptions {
    fn from(value: OpenOptions) -> Self {
        fs::OpenOptions::new()
            .read(value.read)
            .write(value.write)
            .append(value.append)
            .create(value.create)
            .clone()
    }
}

#[derive(Debug, PartialEq, Hash, Eq)]
pub(crate) enum TestDirEnt {
    File(Arc<Path>),
    Dir(Arc<Path>),
}

impl Fs for TestFs {
    type File = TestFile;
    type DirEnt = TestDirEnt;

    fn open<'a, 'b>(&self, options: OpenOptions, path: &'a str) -> io::Result<Self::File> {
        if self.exists(path)? || options.create {
            return Ok(Self::File {
                path: Path::new(path).into(),
                options,
            });
        }
        Err(io::Error::from(ErrorKind::NotFound))
    }

    fn write_to(&mut self, path: &Self::File, content: &[u8]) -> io::Result<()> {
        if !path.options.write {
            return Err(io::Error::from(ErrorKind::PermissionDenied));
        }

        let content = match String::from_utf8(content.to_vec()) {
            Ok(content) => content,
            Err(_) => return Err(io::Error::from(ErrorKind::Other)),
        };

        if self._is_dir(path.path.clone())? {
            return Err(io::Error::from(ErrorKind::IsADirectory));
        }

        if !self.0.contains_key(&path.path) && path.options.create {
            self.0.insert(path.path.clone(), content);
            return Ok(());
        }

        if !self.0.contains_key(&path.path) {
            return Err(io::Error::from(ErrorKind::NotFound));
        }

        //TODO?: implement errors on incorrect content (eg if the
        // /sys/devices/system/cpu/cpuX/cpufreq/scaling_available_(frequencies|governors) does not allow the operation)
        self.0.insert(path.path.clone(), content);
        Ok(())
    }

    fn dir(&self, path: &str) -> io::Result<Vec<Self::DirEnt>> {
        self._dir(Arc::from(Path::new(path)))
    }

    fn exists(&self, path: &str) -> io::Result<bool> {
        self._exists(Arc::from(Path::new(path)))
    }

    fn read_to_string(&self, f: &Self::File) -> io::Result<String> {
        if self._is_dir(f.path.clone())? {
            return Err(io::Error::from(ErrorKind::IsADirectory));
        }

        if !f.options.read {
            return Err(io::Error::from(ErrorKind::PermissionDenied));
        }

        match self.0.get(&*f.path) {
            None => Err(io::Error::from(ErrorKind::NotFound)),
            Some(content) => Ok(content.into()),
        }
    }

    fn is_dir(dirent: &Self::DirEnt) -> bool {
        matches!(dirent, Self::DirEnt::Dir(_))
    }
}

impl TestFs {
    fn _dir(&self, path: Arc<Path>) -> io::Result<Vec<TestDirEnt>> {
        if self._exists(path.clone())? {
            return Err(io::Error::from(ErrorKind::NotADirectory));
        }
        let len_path = path.iter().count();
        Ok(self
            .0
            .iter()
            .filter(|(p, _)| (**p != path) && (p.starts_with(path.clone())))
            .map(|(p, _)| p.clone())
            .map(|p| {
                // p is guaranteed to have more elements than path
                if let Some(_second_after_path) = p.iter().skip(len_path + 1).next() {
                    // we are dealing with a dir
                    return TestDirEnt::Dir(Arc::from(
                        p.components()
                            .take(len_path + 1)
                            .map(|m| m.as_os_str())
                            .collect::<PathBuf>(),
                    ));
                }
                // we are dealing with a file
                return TestDirEnt::File(p.clone());
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect())
    }
    pub(crate) fn new(l: &[&str]) -> Self {
        let mut table = HashMap::new();
        for it in l {
            table.insert(Arc::from(Path::new(it)), "no content".into());
        }
        Self(table)
    }

    fn _is_dir(&self, path: Arc<Path>) -> io::Result<bool> {
        if !self._exists(path.clone())? {return Ok(false);}
        match self._dir(path) {
            Err(e) => match e.kind() {
                ErrorKind::NotADirectory => Ok(false),
                _ => Err(e),
            },
            _ => Ok(true),
        }
    }

    fn _exists(&self, path: Arc<Path>) -> io::Result<bool> {
        Ok(self.0.contains_key(&*path))
    }
}
