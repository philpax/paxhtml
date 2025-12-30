use std::{
    io::Write,
    path::{Path, PathBuf},
};

use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

#[derive(Clone, Debug, PartialEq, Eq)]
/// A path for a route in an HTML document.
pub struct RoutePath<'bump> {
    /// The segments of the path.
    segments: BumpVec<'bump, BumpString<'bump>>,
    /// The (optional) filename for this path.
    filename: Option<BumpString<'bump>>,
}
impl<'bump> RoutePath<'bump> {
    /// Create a new route path from a list of segments.
    pub fn new(
        bump: &'bump Bump,
        segments: impl IntoIterator<Item = &'bump str>,
        filename: Option<&str>,
    ) -> Self {
        Self {
            segments: BumpVec::from_iter_in(
                segments
                    .into_iter()
                    .map(|s| BumpString::from_str_in(s, bump)),
                bump,
            ),
            filename: filename.map(|f| BumpString::from_str_in(f, bump)),
        }
    }

    /// Set the `filename` of this [`RoutePath`].
    pub fn with_filename(mut self, bump: &'bump Bump, filename: &str) -> Self {
        self.filename = Some(BumpString::from_str_in(filename, bump));
        self
    }

    /// Get the `filename` of this [`RoutePath`].
    ///
    /// If no `filename` is present, this will use `index.html` instead.
    pub fn filename(&self) -> &str {
        self.filename
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("index.html")
    }

    /// Get the directory path for the route (i.e. the directory for which files
    /// should be written to).
    pub fn dir_path(&self, out_dir: &Path) -> PathBuf {
        let mut path = out_dir.to_path_buf();
        for segment in &self.segments {
            path.push(segment.as_str());
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
        let segments: Vec<&str> = self.segments.iter().map(|s| s.as_str()).collect();
        let mut path = format!("/{}", segments.join("/"));
        if !path.ends_with('/') {
            path.push('/');
        }
        if let Some(filename) = &self.filename {
            path += filename.as_str();
        }
        path
    }

    /// Get the absolute URL for the route (i.e. the URL that points to this
    /// route).
    pub fn abs_url(&self, domain: &str) -> String {
        format!("{domain}{}", self.url_path())
    }
}
