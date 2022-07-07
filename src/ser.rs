//! Serializers for the core types.

use serde::ser::{Serialize, Serializer};

use crate::{TrainDate, TrainTime};

impl Serialize for TrainDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl Serialize for TrainTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[cfg(test)]
mod tests {
    use super::{TrainDate, TrainTime};

    #[test]
    fn train_date_test() {
        let d = TrainDate::new(2022, 4, 1);
        let json = serde_json::to_string(&d).unwrap();

        assert_eq!(json, r#""01.04.2022""#);
    }

    #[test]
    fn train_time_test() {
        let t = TrainTime::new(5, 7);
        let json = serde_json::to_string(&t).unwrap();

        assert_eq!(json, r#""05:07""#);
    }
}
