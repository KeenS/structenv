/// initialize the struct from env.
/// # Panics
/// if the value of corresponding variable is not a valid unicode squence or
/// fail to parse it, then panics.
pub trait StructEnv {
    fn from_env() -> Self;
}
