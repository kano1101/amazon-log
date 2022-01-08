pub fn to_default<T>(owner: &mut Option<Box<T>>) -> T {
    *::std::mem::replace(owner, None).unwrap()
}
pub fn to_option<T>(owner: &mut Option<Box<T>>, raw: T) {
    let _ = ::std::mem::replace(owner, Some(Box::new(raw)));
}
pub fn to_year(date: String) -> i32 {
    use chrono::prelude::*;
    NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap().year()
}
// pub fn to_month(date: String) -> i32 {
//     use chrono::prelude::*;
//     NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap().month()
// }
// pub fn to_day(date: String) -> i32 {
//     use chrono::prelude::*;
//     NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap().day()
// }
pub fn to_naive_date(date: String) -> chrono::NaiveDate {
    use chrono::prelude::*;
    NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap()
}
