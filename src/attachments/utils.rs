use axum::{body::Bytes, BoxError};
use futures::{Stream, TryStreamExt};
use std::{io, path::PathBuf};
use tokio::{
    fs::{create_dir_all, remove_file, File},
    io::BufWriter,
};
use tokio_util::io::StreamReader;

use crate::common::errors::FieldError;

pub fn filename_is_valid(filename: &str) -> bool {
    let path = std::path::Path::new(filename);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}

pub async fn stream_to_file<S, E>(
    base_dir: PathBuf,
    filename: &str,
    stream: S,
) -> Result<u64, FieldError>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !filename_is_valid(filename) {
        return Err(FieldError::InvalidParams(filename.to_string()));
    }

    async {
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        if !base_dir.exists() {
            create_dir_all(&base_dir).await?;
        }
        let path = base_dir.join(filename);
        let mut file = BufWriter::new(File::create(path).await?);

        let size = tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<u64, io::Error>(size)
    }
    .await
    .map_err(|_| FieldError::InvalidParams("files".to_string()))
}

pub async fn delete_file(base_dir: PathBuf, filepath: &str) -> bool {
    let filepath = filepath.strip_prefix("/");
    if filepath.is_none() {
        return false;
    }
    let filepath = filepath.unwrap();
    let path = base_dir.join(filepath);
    let _ = remove_file(path).await;
    true
}
