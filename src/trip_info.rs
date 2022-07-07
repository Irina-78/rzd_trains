use reqwest::blocking::Response;
use serde::Serialize;
use std::fmt;
use url::form_urlencoded::byte_serialize;

use crate::client::{RzdClientInterface, RzdQueryType, RzdRequestId};
use crate::{error::Error, Result};
use crate::{ReplyResult, ResultList, RzdStationCode, TrainDate, TrainTime};

/// Train stops search.
pub struct TripStopsSearch {
    train_number: String,
    train_date: TrainDate,
}

impl TripStopsSearch {
    /// Takes a number and a date of the train and creates a new search query.
    pub fn new(train_number: &str, train_date: TrainDate) -> Result<Self> {
        let train_number = train_number.trim().to_uppercase();

        if train_number.is_empty() {
            return Err(Error::EmptyTrainNumber);
        }
        debug!("query: {} at {}", train_number, train_date);

        Ok(TripStopsSearch {
            train_number,
            train_date,
        })
    }
}

impl RzdClientInterface<TripStations> for TripStopsSearch {
    fn query_type(&self) -> RzdQueryType {
        RzdQueryType::WithId
    }

    fn request_id(&self) -> String {
        let train_encoded: String = byte_serialize(self.train_number.as_bytes()).collect();

        format!(
            "https://pass.rzd.ru/timetable/public/ru\
            ?layer_id=5804\
            &date={}\
            &train_num={}\
            &json=y\
            &format=array",
            self.train_date, train_encoded
        )
    }

    fn request_data(&self, id: RzdRequestId) -> String {
        format!(
            "https://pass.rzd.ru/timetable/public/ru\
            ?layer_id=5804\
            &rid={}\
            &json=y\
            &format=array",
            id
        )
    }

    fn deserialize_reply_id(&self, response: Response) -> Result<Option<RzdRequestId>> {
        let reply: RidReply = match response.json() {
            Err(e) => return Err(Error::DeserializeError(format!("{}", e))),
            Ok(r) => r,
        };
        let reply = reply.0;
        trace!("reply: {:?}", reply);

        if !reply.success {
            return Err(reply.error);
        }

        Ok(Some(reply.value))
    }

    fn deserialize_reply_data(&self, response: Response) -> Result<Option<TripStations>> {
        let reply: TripInfoReply = match response.json() {
            Err(e) => return Err(Error::DeserializeError(format!("{}", e))),
            Ok(r) => r,
        };
        let reply = reply.0;
        trace!("reply: {:?}", reply);

        if !reply.success {
            return Err(reply.error);
        }

        if reply.value.is_empty() {
            return Ok(None);
        }

        Ok(Some(reply.value))
    }
}

#[derive(Debug)]
struct RidReply(ReplyResult<RzdRequestId>);

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Info about railway stops.
pub struct TripStop {
    station: String,
    code: RzdStationCode,
    trip_days: u32,
    leaving_time: Option<TrainTime>,
    arriving_time: Option<TrainTime>,
}

impl TripStop {
    /// Returns the name of the railway stop.
    #[inline]
    pub fn station(&self) -> &str {
        &self.station
    }

    /// Returns RZD code of the railway stop.
    #[inline]
    pub fn code(&self) -> RzdStationCode {
        self.code
    }

    /// Returns how many days will pass by the time the train arrive at the stopping station.
    #[inline]
    pub fn trip_days(&self) -> u32 {
        self.trip_days
    }

    /// Returns the departure time of the train.
    #[inline]
    pub fn leaving_time(&self) -> Option<TrainTime> {
        self.leaving_time
    }

    /// Returns the arrival time of the train.
    #[inline]
    pub fn arriving_time(&self) -> Option<TrainTime> {
        self.arriving_time
    }
}

impl fmt::Display for TripStop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let time1 = match self.leaving_time {
            Some(t) => format!(" отправление - {},", t),
            None => String::new(),
        };
        let time2 = match self.arriving_time {
            Some(t) => format!(" прибытие - {},", t),
            None => String::new(),
        };
        write!(
            f,
            "\t\"{}\" {},{}{} в пути {} дн.",
            self.station, self.code, time1, time2, self.trip_days
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
/// Info about the stops the train makes.
pub struct TripStations {
    train_number: String,
    stations: ResultList<TripStop>,
}

impl TripStations {
    /// Returns the train number.
    #[inline]
    pub fn train_number(&self) -> &str {
        &self.train_number
    }

    /// Returns the immutable list of stopping stations.
    #[inline]
    pub fn stations(&self) -> &ResultList<TripStop> {
        &self.stations
    }

    /// Returns the mutable list of stopping stations.
    pub fn stations_mut(&mut self) -> &mut ResultList<TripStop> {
        &mut self.stations
    }

    /// Returns true if the list of stops is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.stations.is_empty()
    }

    /// Performs the conversion into a JSON string.
    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(v) => v.to_string(),
            Err(_) => "{}".to_string(),
        }
    }
}

impl fmt::Display for TripStations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Остановки поезда № \"{}\":\n", self.train_number)?;
        for s in self.stations.iter() {
            write!(f, "{}\n", s)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct TripInfoReply(ReplyResult<TripStations>);

mod de {
    use super::{RidReply, TripInfoReply, TripStations, TripStop};
    use crate::client::RzdRequestId;
    use crate::error::Error as GError;
    use crate::error::RzdErrors;
    use crate::{ReplyResult, ResultList, RzdStationCode, TrainTime};
    use serde::Deserialize;
    type ReplyResultId = ReplyResult<RzdRequestId>;
    type ReplyResultStations = ReplyResult<TripStations>;
    use crate::parse_train_time;

    impl<'de> serde::Deserialize<'de> for RidReply {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize, Debug)]
            struct RzdResult {
                #[serde(alias = "type")]
                #[serde(default)]
                result: String,

                #[serde(default)]
                rid: RzdRequestId,
            }

            let input = RzdResult::deserialize(deserializer)?;

            let res_type: &str = &(input.result);

            let reply = match res_type {
                "REQUEST_ID" => ReplyResultId::success(input.rid),
                _ => ReplyResultId::fail(GError::FailRzdResponse),
            };

            Ok(RidReply(reply))
        }
    }

    impl<'de> serde::Deserialize<'de> for TripInfoReply {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize, Debug, Default)]
            struct RzdStop {
                #[serde(alias = "Station")]
                #[serde(default)]
                station: String,

                #[serde(alias = "Days")]
                #[serde(default)]
                days: String,

                #[serde(alias = "DepTime")]
                #[serde(default)]
                dep_time: String,

                #[serde(alias = "ArvTime")]
                #[serde(default)]
                arv_time: String,

                #[serde(alias = "Code")]
                #[serde(default)]
                code: RzdStationCode,
            }

            #[derive(Deserialize, Debug, Default)]
            struct RzdRoutes {
                #[serde(alias = "Stop")]
                #[serde(default)]
                stops: Vec<RzdStop>,
            }

            #[derive(Deserialize, Debug, Default)]
            struct RzdTrain {
                #[serde(alias = "Number")]
                #[serde(default)]
                train_number: String,
            }

            #[derive(Deserialize, Debug, Default)]
            struct RzdInfo {
                #[serde(alias = "Train")]
                #[serde(default)]
                train: RzdTrain,

                #[serde(alias = "Routes")]
                #[serde(default)]
                routes: RzdRoutes,

                #[serde(alias = "Error")]
                #[serde(default)]
                error: RzdError,
            }

            #[derive(Deserialize, Debug, Default)]
            struct RzdError {
                #[serde(default)]
                content: String,
            }

            #[derive(Deserialize, Debug, Default)]
            struct RzdResult {
                #[serde(alias = "GtExpress_Response")]
                #[serde(default)]
                result: RzdInfo,

                #[serde(alias = "Error")]
                #[serde(default)]
                error: RzdError,

                #[serde(alias = "type")]
                #[serde(default)]
                fst_reply_result: String,
            }

            let input = RzdResult::deserialize(deserializer)?;

            if !input.fst_reply_result.is_empty() {
                return Ok(TripInfoReply(ReplyResultStations::fail(
                    GError::FailRzdResponse,
                )));
            }

            let mut error = input.error.content.trim().to_lowercase();
            if error.is_empty() {
                error = input.result.error.content.trim().to_lowercase();
            }

            if !error.is_empty() {
                match error.strip_suffix(".") {
                    Some(r) => error = r.to_string(),
                    None => {}
                }

                let error = GError::RzdError(RzdErrors::new(vec![error]));
                return Ok(TripInfoReply(ReplyResultStations::fail(error)));
            }

            let stops: Vec<TripStop> = input
                .result
                .routes
                .stops
                .into_iter()
                .map(|s| TripStop {
                    station: s.station,
                    code: s.code,
                    trip_days: s.days.trim().parse().unwrap_or_else(|_| 0),
                    leaving_time: parse_train_time!(s.dep_time),
                    arriving_time: parse_train_time!(s.arv_time),
                })
                .collect();

            let list = TripStations {
                train_number: input.result.train.train_number,
                stations: ResultList::new(stops),
            };

            Ok(TripInfoReply(ReplyResultStations::success(list)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RidReply, TripInfoReply, TripStations, TripStop};
    use crate::client::RzdRequestId;
    use crate::parse_train_time;
    use crate::{error::Error, RzdErrors};
    use crate::{ResultList, RzdStationCode, TrainTime};

    #[test]
    fn rid_reply_deserialize_test() {
        let answer = r#"{"type":"FAIL","rid":0,"fail_msg":"Произошла системная ошибка."}"#;
        let answer: RidReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::FailRzdResponse;

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"type":"REQUEST_ID","rid":17872768326,"fail_msg":"null"}"#;
        let answer: RidReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = RzdRequestId::new(17872768326);

        assert!(answer.success);
        assert_eq!(answer.value, data);
    }

    #[test]
    fn trip_stations_deserialize_test() {
        let answer = r#"{"Error":{"Version":"2.7.81","content":"Parameter [Train::Number]: not found or invalid format","Code":"040311"}}"#;
        let answer: TripInfoReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::RzdError(RzdErrors::new(vec![
            "parameter [train::number]: not found or invalid format".to_string(),
        ]));

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"GtExpress_Response":{"ExprInfo":"II","ReqExpressZK":3189015,"ReqLocalRecv":"20.03.2022 21:54:01","ReqLocalSend":"20.03.2022 21:54:01","ReqAddress":"MZD:5433","Error":{"content":"Неверная дата отправления.","Code":2010},"Version":"2.7.86","Type":"TrainRoute","ReqExpressDateTime":"20.03.2022 21:54"}}"#;
        let answer: TripInfoReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::RzdError(RzdErrors::new(
            vec!["неверная дата отправления".to_string()],
        ));

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"type":"REQUEST_ID","rid":17872768326,"fail_msg":"null"}"#;
        let answer: TripInfoReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::FailRzdResponse;

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"GtExpress_Response":{"ReqExpressZK":3263309,"ReqLocalRecv":"20.03.2022 19:04:56","ReqLocalSend":"20.03.2022 19:04:56","ReqAddress":"MZD:5431","Train":{"Route":{"Station":["С-ПЕТЕР-ГЛ","МОСКВА ОКТ"],"CodeFrom":2004001,"CodeTo":2006004},"Number":"001А"},"Version":"2.7.81","Routes":{"Stop":[{"Station":"С-ПЕТЕР-ГЛ","Distance":0,"Days":"00","DepTime":"23:55","Code":2004001},{"ArvTime":"07:55","Station":"МОСКВА ОКТ","Distance":650,"Days":"01","Code":2006004}],"Title":"ОСНОВНОЙ МАРШРУТ","Route":{"Station":["С-ПЕТЕР-ГЛ","МОСКВА ОКТ"],"CodeFrom":2004001,"CodeTo":2006004}},"Type":"TrainRoute","ReqExpressDateTime":"20.03.2022 00:00"}}"#;
        let answer: TripInfoReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = TripStations {
            train_number: String::from("001А"),
            stations: ResultList::new(vec![
                TripStop {
                    station: String::from("С-ПЕТЕР-ГЛ"),
                    code: RzdStationCode::new(2004001),
                    trip_days: 0,
                    leaving_time: parse_train_time!("23:55"),
                    arriving_time: parse_train_time!(""),
                },
                TripStop {
                    station: String::from("МОСКВА ОКТ"),
                    code: RzdStationCode::new(2006004),
                    trip_days: 1,
                    leaving_time: parse_train_time!(""),
                    arriving_time: parse_train_time!("07:55"),
                },
            ]),
        };

        assert!(answer.success);
        assert_eq!(answer.value, data);
    }
}
