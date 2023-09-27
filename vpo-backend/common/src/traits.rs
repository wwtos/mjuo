pub trait TryRef<T> {
    type Error;

    fn try_ref(&self) -> Result<&T, Self::Error>;
}
