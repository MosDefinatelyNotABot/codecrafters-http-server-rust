use flate2::Compression;
use flate2::write::GzEncoder;
use std::{collections::HashMap, io::Write, sync::LazyLock};

type CompressionFn = Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>;

pub(crate) static COMPRESSION_METHODS: LazyLock<HashMap<String, CompressionFn>> =
    LazyLock::new(|| {
        HashMap::from([
            (
                "identity".to_string(),
                Box::new(no_compression) as CompressionFn,
            ),
            (
                "gzip".to_string(),
                Box::new(gzip_compression) as CompressionFn,
            ),
        ])
    });

pub(crate) fn compress_data(data: &[u8], method: Option<&str>) -> Vec<u8> {
    let method = method.unwrap_or("identity");

    let Some(compression_fn) = COMPRESSION_METHODS.get(method) else {
        return data.to_vec();
    };
    compression_fn(data)
}

pub fn no_compression(data: &[u8]) -> Vec<u8> {
    // identity compression
    data.to_vec()
}

pub fn gzip_compression(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .expect("gzip compression failed to write to buffer. :(");
    encoder
        .finish()
        .expect("gzip compression failed to finish. :(")
}
