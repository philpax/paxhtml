use std::{
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
/// A path for a route in an HTML document.
pub struct RoutePath {
    /// The segments of the path.
    segments: Vec<String>,
    /// The (optional) filename for this path.
    filename: Option<String>,
}
impl RoutePath {
    /// Create a new route path from a list of segments.
    pub fn new<'a>(
        segments: impl IntoIterator<Item = &'a str>,
        filename: impl Into<Option<String>>,
    ) -> Self {
        Self {
            segments: segments.into_iter().map(|s| s.to_string()).collect(),
            filename: filename.into(),
        }
    }
    /// Set the `filename` of this [`RoutePath`].
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }
    /// Get the `filename` of this [`RoutePath`].
    ///
    /// If no `filename` is present, this will use `index.html` instead.
    pub fn filename(&self) -> &str {
        self.filename.as_deref().unwrap_or("index.html")
    }
    /// Get the directory path for the route (i.e. the directory for which files
    /// should be written to).
    pub fn dir_path(&self, out_dir: &Path) -> PathBuf {
        let mut path = out_dir.to_path_buf();
        for segment in &self.segments {
            path.push(segment);
        }
        path
    }
    /// Get the file path for the route (i.e. the path to the file that should
    /// be written to the directory).
    ///
    /// If no `filename` is present, this will use `index.html` instead.
    pub fn file_path(&self, out_dir: &Path) -> PathBuf {
        self.dir_path(out_dir).join(self.filename())
    }
    /// Create a file writer for this path based on [`Self::file_path`].
    /// This will create the parent folder, too.
    pub fn writer(&self, out_dir: &Path) -> std::io::Result<std::io::BufWriter<std::fs::File>> {
        let path = self.file_path(out_dir);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::io::BufWriter::new(std::fs::File::create(path)?))
    }
    /// Write the given contents to the path at [`Self::file_path`].
    /// This will create the parent folder, too.
    pub fn write(&self, out_dir: &Path, content: impl AsRef<[u8]>) -> std::io::Result<()> {
        let mut writer = self.writer(out_dir)?;
        writer.write_all(content.as_ref())
    }
    /// Get the URL path for the route (i.e. the path that should be used in the
    /// URL).
    ///
    /// This always starts and ends with a `/`.
    pub fn url_path(&self) -> String {
        let mut path = format!("/{}", self.segments.join("/"));
        if !path.ends_with('/') {
            path.push('/');
        }
        if let Some(filename) = &self.filename {
            path += filename;
        }
        path
    }
    /// Get the absolute URL for the route (i.e. the URL that points to this
    /// route).
    pub fn abs_url(&self, domain: &str) -> String {
        format!("{domain}{}", self.url_path())
    }
}
