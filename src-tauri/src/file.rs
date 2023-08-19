use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

pub fn compute_path<P: AsRef<Path>>(base: P, hash: &str) -> PathBuf {
    if hash.len() <= 4 {
        return base.as_ref().join(hash);
    }

    let f1 = &hash[..2];
    let f2 = &hash[2..4];
    base.as_ref().join(f1).join(f2).join(hash)
}

pub fn copy_to_storage<P: AsRef<Path>, S: AsRef<Path>>(
    base: P,
    src: S,
) -> Result<String, std::io::Error> {
    let ext = src.as_ref().extension().map(|x| x.to_string_lossy());
    let hash = sha256::try_digest(&src.as_ref())?;
    let mut path = compute_path(base, &hash);
    if let Some(ext) = &ext {
        path.set_extension(ext.as_ref());
    }

    let path_parent = path.parent().unwrap();
    if !path_parent.exists() {
        fs::create_dir_all(path_parent)?;
    }

    fs::copy(src.as_ref(), path)?;
    if let Some(ext) = &ext {
        Ok(format!("{}.{}", hash, ext.as_ref()))
    } else {
        Ok(hash)
    }
}
