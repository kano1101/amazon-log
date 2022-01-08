pub fn to_default<T>(owner: &mut Option<Box<T>>) -> T {
    *::std::mem::replace(owner, None).unwrap()
}
pub fn to_option<T>(owner: &mut Option<Box<T>>, raw: T) {
    let _ = ::std::mem::replace(owner, Some(Box::new(raw)));
}
