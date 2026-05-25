use std::io::Error;

pub(crate) fn path_spilter(path: &String) -> Result<(String, Vec<String>), Error> {
    // seperates the base path and path chunks from a URL path
    if let Some(path_clean) = path.strip_prefix("/") {
        let chunks: Vec<String> = path_clean.split('/').map(|c| c.to_string()).collect();
        match chunks.first() {
            Some(base_path) if !base_path.is_empty() => {
                let path_chunks: Vec<String> = chunks[1..].iter().map(|c| c.to_string()).collect();
                Ok((format!("/{}", base_path), path_chunks))
            }
            _ => Err(Error::new(
                std::io::ErrorKind::Other,
                "Path must start with '/'",
            )),
        }
    } else {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Path must start with '/'",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_base_from_single_chunk() {
        let (base, chunks) = path_spilter(&"/echo/hello".to_string()).unwrap();
        assert_eq!(base, "echo");
        assert_eq!(chunks, vec!["hello"]);
    }

    #[test]
    fn splits_base_from_multiple_chunks() {
        let (base, chunks) = path_spilter(&"/echo/hello/world/spliter".to_string()).unwrap();
        assert_eq!(base, "echo");
        assert_eq!(chunks, vec!["hello", "world", "spliter"]);
    }

    #[test]
    fn base_only_path_yields_empty_chunks() {
        let (base, chunks) = path_spilter(&"/echo".to_string()).unwrap();
        assert_eq!(base, "echo");
        assert!(chunks.is_empty());
    }

    #[test]
    fn root_path_yields_error() {
        assert!(path_spilter(&"/".to_string()).is_err());
    }

    #[test]
    fn path_without_leading_slash_yields_error() {
        assert!(path_spilter(&"noslash".to_string()).is_err());
    }

    #[test]
    fn empty_path_yields_error() {
        assert!(path_spilter(&"".to_string()).is_err());
    }
}
