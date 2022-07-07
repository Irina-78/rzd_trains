use reqwest::blocking::Response;
use serde::Serialize;
use std::fmt;
use url::form_urlencoded::byte_serialize;

use crate::client::{RzdClientInterface, RzdQueryType, RzdRequestId};
use crate::{error::Error, Result};
use crate::{ResultList, RzdStationCode};

const MIN_QUERY_LENGTH: usize = 2;

/// Station code search by part of the name.
pub struct StationCodeSearch {
    query: String,
}

impl StationCodeSearch {
    /// Takes part of the station name and creates a new search query.
    ///
    /// # Errors
    ///
    /// The method fails if the query has less than 2 characters.
    pub fn new(query: &str) -> Result<Self> {
        let query = query.trim().to_uppercase();

        if query.is_empty() {
            return Err(Error::TooShortQuery);
        }
        if query.chars().count() < MIN_QUERY_LENGTH {
            return Err(Error::TooShortQuery);
        }
        debug!("query: {}", query);

        Ok(StationCodeSearch { query })
    }
}

impl RzdClientInterface<ResultList<StationItem>> for StationCodeSearch {
    fn query_type(&self) -> RzdQueryType {
        RzdQueryType::Simple
    }

    fn request_id(&self) -> String {
        String::new()
    }

    fn request_data(&self, _id: RzdRequestId) -> String {
        let query_encoded: String = byte_serialize(self.query.as_bytes()).collect();

        format!(
            "https://pass.rzd.ru/suggester\
                ?stationNamePart={}\
                &lang=ru\
                &compactMode=y",
            query_encoded
        )
    }

    fn deserialize_reply_id(&self, _response: Response) -> Result<Option<RzdRequestId>> {
        Err(Error::UnsupportedOperation)
    }

    fn deserialize_reply_data(
        &self,
        response: Response,
    ) -> Result<Option<ResultList<StationItem>>> {
        let answer: AnswerList = match response.json() {
            Err(e) => return Err(Error::DeserializeError(format!("{}", e))),
            Ok(r) => r,
        };
        debug!("answer: {}", answer);

        let stations: Vec<StationItem> = answer
            .0
            .into_iter()
            .filter(|s| is_first_letters_found(&s.name, &self.query))
            .collect();
        info!("{} stations found", stations.len());

        if stations.len() == 0 {
            return Ok(None);
        }

        Ok(Some(ResultList(stations)))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Info about the correlation of the name and the station code returned by RZD.
pub struct StationItem {
    name: String,
    code: RzdStationCode,
}

impl StationItem {
    /// Takes a name and a code of the station and creates a new item.
    fn new(name: String, code: RzdStationCode) -> Self {
        StationItem { name, code }
    }

    /// Returns the name of the station.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the code of the station.
    #[inline]
    pub fn code(&self) -> RzdStationCode {
        self.code
    }
}

impl fmt::Display for StationItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.code(), self.name)
    }
}

#[derive(Debug)]
struct AnswerList(Vec<StationItem>);

impl fmt::Display for AnswerList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for s in &self.0 {
            write!(f, "\n{}", s)?;
        }
        Ok(())
    }
}

mod de {
    use super::{AnswerList, StationItem};
    use crate::RzdStationCode;
    use serde::Deserialize;

    impl<'de> serde::Deserialize<'de> for AnswerList {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize, Debug, Default)]
            struct RzdStation {
                #[serde(alias = "n")]
                #[serde(default)]
                name: String,

                #[serde(alias = "c")]
                #[serde(default)]
                code: RzdStationCode,
            }

            let input = Vec::<RzdStation>::deserialize(deserializer)?;

            let stations: Vec<StationItem> = input
                .into_iter()
                .map(|s| StationItem::new(s.name, s.code))
                .collect();

            Ok(AnswerList(stations))
        }
    }
}

// The RZD server searches only by two letters of the query,
// so it needs to analyze the whole request.
fn is_first_letters_found(text: &str, letters: &str) -> bool {
    let text = text.trim().to_uppercase();
    let letters = letters.trim().to_uppercase();

    if letters.is_empty() | text.is_empty() {
        return false;
    }

    for word in text.split_whitespace() {
        if word.starts_with(&letters) {
            trace!("searching {} in {}...ok", letters, text);
            return true;
        }
    }

    for word in text.split_terminator('-') {
        if word.starts_with(&letters) {
            trace!("searching {} in {}...ok", letters, text);
            return true;
        }
    }

    trace!("searching {} in {}...nok", letters, text);
    false
}

#[cfg(test)]
mod tests {
    use super::is_first_letters_found;
    use super::{AnswerList, StationCodeSearch, StationItem};
    use crate::RzdStationCode;

    #[test]
    fn search_test() {
        assert!(StationCodeSearch::new("").is_err());
        assert!(StationCodeSearch::new(" ").is_err());
        assert!(StationCodeSearch::new("м").is_err());
        assert!(!StationCodeSearch::new("мОс").is_err());
    }

    #[test]
    fn is_first_letters_found_test() {
        assert!(!is_first_letters_found("", ""));
        assert!(!is_first_letters_found("", "МОС"));
        assert!(!is_first_letters_found("МОСКОВСКАЯ", ""));
        assert!(!is_first_letters_found("МОСКОВСКАЯ", " "));
        assert!(is_first_letters_found("МОСКОВСКАЯ", "мОс"));
        assert!(is_first_letters_found("ВОЕННЫЙ ГОРОДОК", "гоР"));
        assert!(is_first_letters_found("САНКТ-ПЕТЕРБУРГ-ГЛАВН", "пет"));
    }

    #[test]
    fn stations_deserialize_test() {
        let data = AnswerList(vec![]);

        let answer = "[]";
        let answer: AnswerList = serde_json::from_str(answer).unwrap();

        assert_eq!(&answer.0, &data.0);

        let data = AnswerList(vec![
            StationItem::new(String::from("ВОЕННОЕ ШОССЕ"), RzdStationCode::new(2034058)),
            StationItem::new(
                String::from("БУРЛИТ-ВОЛОЧАЕВСКИЙ"),
                RzdStationCode::new(2034458),
            ),
            StationItem::new(String::from("ВОРОПАЕВО"), RzdStationCode::new(2100047)),
        ]);

        let answer = r#"[{"n":"ВОЕННОЕ ШОССЕ","c":2034058,"S":4,"L":0},{"n":"БУРЛИТ-ВОЛОЧАЕВСКИЙ","c":2034458,"S":0,"L":2},{"n":"ВОРОПАЕВО","c":2100047,"S":0,"L":4}]"#;
        let answer: AnswerList = serde_json::from_str(answer).unwrap();

        assert_eq!(&answer.0, &data.0);
    }
}
