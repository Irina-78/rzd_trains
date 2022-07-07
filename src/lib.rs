//! # rzd_trains
//!
//! The `rzd_trains` crate provides to get information
//! about passenger trains and routes of the JSCo "RZD".
//!
//! ## Making requests
//!
//! Create the required search and pass it to the client:
//!
//! ```rust,no_run
//! # use rzd_trains::{TrainScheduleSearch, RzdStationCode, TrainDate, TrainType, RzdClient, RouteList};
//! #
//! let from = RzdStationCode::new(2000000);
//! let to = RzdStationCode::new(2004000);
//! let leaving_date = TrainDate::new(2022, 4, 1);
//! let train_type = TrainType::AllTrains;
//! let check_seats = true;
//!
//! let q = TrainScheduleSearch::new(from, to, leaving_date, train_type, check_seats);
//!
//! let result = RzdClient::<RouteList>::get(&q).unwrap();
//! match result {
//!     Some(list) => println!("{}", list),
//!     None => println!("Nothing found"),
//! };
//! ```
//!

#[macro_use]
extern crate log;

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, fmt::Debug, fmt::Display, str::FromStr};

mod error;
pub use crate::error::{Error, RzdErrors};

/// A `Result` alias where the `Err` case is `rzd_trains::Error`.
type Result<T> = std::result::Result<T, Error>;

mod client;
pub use client::RzdClient;

mod ser;

mod des;

mod station_codes;
pub use crate::station_codes::{StationCodeSearch, StationItem};
pub type StationList = ResultList<StationItem>;

mod train_schedule;
pub use crate::train_schedule::{Route, TrainScheduleSearch};
pub type RouteList = ResultList<Route>;

mod train_info;
pub use crate::train_info::{TrainItem, TrainSearch};
pub type TrainInfoList = ResultList<TrainItem>;

mod trip_info;
pub use crate::trip_info::{TripStations, TripStopsSearch};

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
/// Direction of the route.
enum RouteDirection {
    OneWay = 0,
    _RoundTrip = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
/// Type of the trains.
pub enum TrainType {
    /// Long-distance trains.
    Train = 1,
    /// Suburban electric trains.
    ElectricTrain = 2,
    /// All kinds of the trains.
    AllTrains = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
/// What trains should it search, with free seats only or any train?
enum ShowSeats {
    /// Trains with free seats only.
    FreeOnly = 1,
    /// All trains.
    All = 0,
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// A date of departure or arrival of the train.
pub struct TrainDate(NaiveDate);

impl TrainDate {
    /// Creates `TrainDate` from a year, a month and a day.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rzd_trains::TrainDate;
    /// #
    /// let d = TrainDate::new(2022, 4, 1);
    ///
    /// assert_eq!(format!("{}", d), "01.04.2022");
    /// ```
    pub fn new(year: u32, month: u32, day: u32) -> Self {
        let date = match NaiveDate::from_ymd_opt(year as i32, month, day) {
            Some(d) => d,
            None => Utc::now().naive_utc().date(),
        };

        TrainDate(date)
    }
}

impl Display for TrainDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%d.%m.%Y"))
    }
}

impl From<NaiveDate> for TrainDate {
    fn from(d: NaiveDate) -> Self {
        TrainDate::new(d.year() as u32, d.month(), d.day())
    }
}

impl Into<NaiveDate> for TrainDate {
    #[inline]
    fn into(self) -> NaiveDate {
        self.0
    }
}

impl FromStr for TrainDate {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let v: Vec<u32> = s.split(".").filter_map(|s| s.parse().ok()).collect();

        let (day, month, year) = match &v[..] {
            &[d, m, y] => (d, m, y),
            _ => return Err(Error::ParseDateError(s.to_string())),
        };

        Ok(TrainDate::new(year, month, day))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_train_date {
    ($str:expr) => {
        match $str.parse::<TrainDate>() {
            Ok(d) => Some(d),
            Err(_) => None,
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// A time of departure or arrival of the train.
pub struct TrainTime(NaiveTime);

impl TrainTime {
    /// Creates `TrainTime` from the number of hours and minutes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rzd_trains::TrainTime;
    /// #
    /// let t = TrainTime::new(5, 7);
    ///
    /// assert_eq!(format!("{}", t), "05:07");
    /// ```
    pub fn new(hours: u32, minutes: u32) -> Self {
        let time = match NaiveTime::from_hms_opt(hours, minutes, 0) {
            Some(d) => d,
            None => NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        };
        TrainTime(time)
    }
}

impl Display for TrainTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%H:%M"))
    }
}

impl From<NaiveTime> for TrainTime {
    fn from(t: NaiveTime) -> Self {
        TrainTime::new(t.hour(), t.minute())
    }
}

impl Into<NaiveTime> for TrainTime {
    #[inline]
    fn into(self) -> NaiveTime {
        self.0
    }
}

impl FromStr for TrainTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let v: Vec<u32> = s.split(":").filter_map(|s| s.parse().ok()).collect();

        let (h, m) = match &v[..] {
            &[h, m, _s] => (h, m),
            &[h, m] => (h, m),
            _ => return Err(Error::ParseTimeError(s.to_string())),
        };

        Ok(TrainTime::new(h, m))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_train_time {
    ($str:expr) => {
        match $str.parse::<TrainTime>() {
            Ok(d) => Some(d),
            Err(_) => None,
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
/// A digital designation of the station used by RZD.
pub struct RzdStationCode(u32);

impl RzdStationCode {
    /// Creates a new item from the digits.
    pub fn new(code: u32) -> Self {
        RzdStationCode(code)
    }

    /// Performs the conversion into digits.
    #[inline]
    pub fn to_uint(&self) -> u32 {
        self.0
    }
}

impl Default for RzdStationCode {
    fn default() -> Self {
        RzdStationCode(0)
    }
}

impl Display for RzdStationCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RzdStationCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.parse::<u32>() {
            Ok(u) => Ok(RzdStationCode::new(u)),
            Err(_) => Err(Error::ParseStationCodeError(s.to_string())),
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_station_code {
    ($str:expr) => {
        match $str.parse::<RzdStationCode>() {
            Ok(r) => r,
            Err(_) => RzdStationCode::default(),
        }
    };
}

#[derive(Debug)]
#[doc(hidden)]
// Wrapper for the result of the deserialization.
struct ReplyResult<T>
where
    T: Default,
{
    success: bool,
    value: T,
    error: Error,
}

impl<T> ReplyResult<T>
where
    T: Default,
{
    // Returns a result contained a value on success.
    fn success(value: T) -> Self {
        ReplyResult {
            success: true,
            value,
            error: Error::Empty,
        }
    }

    // Returns a result contained an error on fail.
    fn fail(error: Error) -> Self {
        ReplyResult {
            success: false,
            value: T::default(),
            error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// List of the request results.
pub struct ResultList<T>(Vec<T>)
where
    T: Debug + Display + Serialize;

impl<T> ResultList<T>
where
    T: Debug + Display + Serialize,
{
    /// Creates a new list from `Vec`.
    pub fn new(v: Vec<T>) -> Self {
        ResultList(v)
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the data of the list.
    #[inline]
    pub fn as_ref(&self) -> &[T] {
        &self.0.as_slice()
    }

    /// Performs the conversion from the list into `Vec`.
    pub fn to_vec(self) -> Vec<T> {
        self.0
    }

    /// Performs the conversion into a JSON string.
    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(v) => v.to_string(),
            Err(_) => "[]".to_string(),
        }
    }

    /// Creates a non-consuming iterator.
    pub fn iter(&self) -> ResultListIter<T> {
        ResultListIter::<T> {
            item: &self,
            index: 0,
        }
    }

    /// Creates a consuming iterator.
    pub fn into_iter(self) -> ResultListIntoIter<T> {
        ResultListIntoIter::<T> {
            item: self.0.into_iter(),
        }
    }
}

impl<T> Display for ResultList<T>
where
    T: Debug + Display + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for s in &self.0 {
            write!(f, "{}\n", s)?;
        }
        Ok(())
    }
}

impl<T> Default for ResultList<T>
where
    T: Debug + Display + Serialize,
{
    fn default() -> Self {
        ResultList::<T>(vec![])
    }
}

#[doc(hidden)]
pub struct ResultListIter<'a, T: Debug + Display + Serialize> {
    item: &'a ResultList<T>,
    index: usize,
}

#[doc(hidden)]
impl<'a, T: Debug + Display + Serialize> Iterator for ResultListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.item.0.get(self.index) {
            Some(x) => {
                self.index += 1;
                Some(x)
            }
            _ => None,
        }
    }
}

#[doc(hidden)]
pub struct ResultListIntoIter<T: Debug + Display + Serialize> {
    item: std::vec::IntoIter<T>,
}

#[doc(hidden)]
impl<T: Debug + Display + Serialize> Iterator for ResultListIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.item.next()
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_station_code, parse_train_date, parse_train_time};
    use super::{RzdStationCode, TrainDate, TrainTime};

    #[test]
    fn train_date_test() {
        assert_eq!(
            parse_train_date!("01.04.2022"),
            Some(TrainDate::new(2022, 4, 1))
        );
    }

    #[test]
    fn train_time_test() {
        assert_eq!(parse_train_time!("23:05"), Some(TrainTime::new(23, 5)));
    }

    #[test]
    fn station_code_test() {
        assert_eq!(RzdStationCode::default().to_uint(), 0);
        assert_eq!(parse_station_code!("2000000"), RzdStationCode::new(2000000));
    }
}
