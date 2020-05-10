use syn::{Lit, Path};

trait ValueEnum {
    fn read_from_lit(lit: &Lit, path: &Path);
}
