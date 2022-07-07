use reqwest::blocking::Response;
use serde::Serialize;
use std::fmt;

use crate::client::{RzdClientInterface, RzdQueryType, RzdRequestId};
use crate::{error::Error, Result};
use crate::{
    ReplyResult, ResultList, RouteDirection, RzdStationCode, ShowSeats, TrainDate, TrainTime,
    TrainType,
};

/// Schedule of trains search by departure and arrival station codes
/// and departure date.
pub struct TrainScheduleSearch {
    leaving_code: RzdStationCode,
    arriving_code: RzdStationCode,
    leaving_date: TrainDate,
    train_type: TrainType,
    check_seats: ShowSeats,
}

impl TrainScheduleSearch {
    /// Takes departure and arrival station codes, departure date.
    /// It needs to specify a train type, especially, for suburban trains searching.
    /// Search not all but only free seats would be preferred.
    pub fn new(
        leaving_code: RzdStationCode,
        arriving_code: RzdStationCode,
        leaving_date: TrainDate,
        train_type: TrainType,
        free_seats_only: bool,
    ) -> Self {
        let check_seats = match free_seats_only {
            true => ShowSeats::FreeOnly,
            false => ShowSeats::All,
        };

        TrainScheduleSearch {
            leaving_code,
            arriving_code,
            leaving_date,
            train_type,
            check_seats,
        }
    }
}

impl RzdClientInterface<ResultList<Route>> for TrainScheduleSearch {
    fn query_type(&self) -> RzdQueryType {
        match self.train_type {
            TrainType::ElectricTrain => RzdQueryType::Simple,
            _ => RzdQueryType::WithId,
        }
    }

    fn request_id(&self) -> String {
        format!(
            "https://pass.rzd.ru/timetable/public/ru\
            ?layer_id=5827\
            &dir={}\
            &tfl={}\
            &checkSeats={}\
            &code0={}\
            &dt0={}\
            &code1={}",
            RouteDirection::OneWay as u8,
            self.train_type as u8,
            match self.check_seats {
                ShowSeats::All => "0&withoutSeats=y",
                _ => "1",
            },
            self.leaving_code,
            self.leaving_date,
            self.arriving_code
        )
    }

    fn request_data(&self, id: RzdRequestId) -> String {
        if self.query_type() == RzdQueryType::Simple {
            return format!(
                "https://pass.rzd.ru/timetable/public/ru\
                ?layer_id=5827\
                &dir={}\
                &tfl={}\
                &code0={}\
                &dt0={}\
                &code1={}",
                RouteDirection::OneWay as u8,
                self.train_type as u8,
                self.leaving_code,
                self.leaving_date,
                self.arriving_code
            );
        }

        format!(
            "https://pass.rzd.ru/timetable/public/ru\
            ?layer_id=5827\
            &rid={}",
            id
        )
    }

    fn deserialize_reply_id(&self, response: Response) -> Result<Option<RzdRequestId>> {
        if self.query_type() == RzdQueryType::Simple {
            return Err(Error::UnsupportedOperation);
        }

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

    fn deserialize_reply_data(&self, response: Response) -> Result<Option<ResultList<Route>>> {
        let reply: ScheduleReply = match response.json() {
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

        Ok(Some(ResultList(reply.value)))
    }
}

#[derive(Debug)]
struct RidReply(ReplyResult<RzdRequestId>);

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Number of free seats on the train and seat type info.
pub struct SeatsInfo {
    free_seats: u32,
    seats_type: String,
}

impl SeatsInfo {
    /// Returns the number of free seats.
    #[inline]
    pub fn free_seats_number(&self) -> u32 {
        self.free_seats
    }

    /// Returns the seat type.
    #[inline]
    pub fn seats_type(&self) -> &str {
        &self.seats_type
    }
}

impl fmt::Display for SeatsInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.free_seats, self.seats_type)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Train info.
pub struct TrainInfo {
    train_number: String,
    train_brand: String,
    train_type: String,
    leaving_route: String,
    leaving_route_code: RzdStationCode,
    arriving_route: String,
    arriving_route_code: RzdStationCode,
    leaving_station: String,
    leaving_date: Option<TrainDate>,
    leaving_time: Option<TrainTime>,
    arriving_station: String,
    arriving_date: Option<TrainDate>,
    arriving_time: Option<TrainTime>,
    trip_duration: Option<TrainTime>,
    stops: String,
    seats: ResultList<SeatsInfo>,
}

impl TrainInfo {
    /// Returns the train number.
    #[inline]
    pub fn train_number(&self) -> &str {
        &self.train_number
    }

    /// Returns the brand name of the train.
    #[inline]
    pub fn brand(&self) -> &str {
        &self.train_brand
    }

    /// Returns the train type.
    #[inline]
    pub fn train_type(&self) -> &str {
        &self.train_type
    }

    /// Returns the name of the departure station along the train route.
    #[inline]
    pub fn leaving_route(&self) -> &str {
        &self.leaving_route
    }

    /// Returns the RZD code of the departure station along the train route.
    #[inline]
    pub fn leaving_route_code(&self) -> RzdStationCode {
        self.leaving_route_code
    }

    /// Returns the name of the arrival station along the train route.
    #[inline]
    pub fn arriving_route(&self) -> &str {
        &self.arriving_route
    }

    /// Returns the RZD code of the arrival station along the train route.
    #[inline]
    pub fn arriving_route_code(&self) -> RzdStationCode {
        self.arriving_route_code
    }

    /// Returns the name of the departure station.
    #[inline]
    pub fn leaving_station(&self) -> &str {
        &self.leaving_station
    }

    /// Returns the departure date of the train.
    #[inline]
    pub fn leaving_date(&self) -> Option<TrainDate> {
        self.leaving_date
    }

    /// Returns the departure time of the train.
    #[inline]
    pub fn leaving_time(&self) -> Option<TrainTime> {
        self.leaving_time
    }

    /// Returns the name of the arrival station.
    #[inline]
    pub fn arriving_station(&self) -> &str {
        &self.arriving_station
    }

    /// Returns the arrival date of the train.
    #[inline]
    pub fn arriving_date(&self) -> Option<TrainDate> {
        self.arriving_date
    }

    /// Returns the arrival time of the train.
    #[inline]
    pub fn arriving_time(&self) -> Option<TrainTime> {
        self.arriving_time
    }

    /// Returns the duration of the trip.
    #[inline]
    pub fn trip_duration(&self) -> Option<TrainTime> {
        self.trip_duration
    }

    /// Returns the train stops.
    #[inline]
    pub fn stops(&self) -> &str {
        &self.stops
    }

    /// Returns the immutable list of seats on the train.
    #[inline]
    pub fn seats(&self) -> &ResultList<SeatsInfo> {
        &self.seats
    }

    /// Returns the mutable list of seats on the train.
    pub fn seats_mut(&mut self) -> &mut ResultList<SeatsInfo> {
        &mut self.seats
    }
}

impl fmt::Display for TrainInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Поезд № \"{}\" {} {}\n",
            self.train_number, self.train_brand, self.train_type
        )?;
        write!(
            f,
            "по маршруту \"{}\" {} - \"{}\" {}\n",
            self.leaving_route,
            self.leaving_route_code,
            self.arriving_route,
            self.arriving_route_code
        )?;
        let date = match self.leaving_date() {
            Some(d) => format!("{}", d),
            None => String::new(),
        };
        let time = match self.leaving_time() {
            Some(t) => format!("{}", t),
            None => String::new(),
        };
        write!(
            f,
            "\tотправление от \"{}\": {} {}\n",
            self.leaving_station, date, time
        )?;
        let date = match self.arriving_date() {
            Some(d) => format!("{}", d),
            None => String::new(),
        };
        let time = match self.arriving_time() {
            Some(t) => format!("{}", t),
            None => String::new(),
        };
        write!(
            f,
            "\tприбытие в \"{}\": {} {}\n",
            self.arriving_station, date, time
        )?;
        let time = match self.trip_duration() {
            Some(t) => format!("{}", t),
            None => String::new(),
        };
        write!(f, "\tвремя в пути: {}\n", time)?;
        write!(f, "\tостановки: {}\n", self.stops)?;
        write!(f, "\tместа:\n")?;
        for s in self.seats.iter() {
            write!(f, "\t\t{}\n", s)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
/// Info about the route and available trains.
pub struct Route {
    leaving_name: String,
    leaving_code: RzdStationCode,
    arriving_name: String,
    arriving_code: RzdStationCode,
    trains: ResultList<TrainInfo>,
}

impl Route {
    /// Returns the name of the departure station.
    #[inline]
    pub fn leaving_station_name(&self) -> &str {
        &self.leaving_name
    }

    /// Returns the RZD code of the departure station.
    #[inline]
    pub fn leaving_station_code(&self) -> RzdStationCode {
        self.leaving_code
    }

    /// Returns the name of the arrival station.
    #[inline]
    pub fn arriving_station_name(&self) -> &str {
        &self.arriving_name
    }

    /// Returns the RZD code of the arrival station.
    #[inline]
    pub fn arriving_station_code(&self) -> RzdStationCode {
        self.arriving_code
    }

    /// Returns the immutable list of trains.
    #[inline]
    pub fn trains(&self) -> &ResultList<TrainInfo> {
        &self.trains
    }

    /// Returns the mutable list of trains.
    pub fn trains_mut(&mut self) -> &mut ResultList<TrainInfo> {
        &mut self.trains
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Маршрут из \"{}\" {} в \"{}\" {}\n\n",
            self.leaving_name, self.leaving_code, self.arriving_name, self.arriving_code
        )?;
        for t in self.trains.iter() {
            write!(f, "{}\n", t)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ScheduleReply(ReplyResult<Vec<Route>>);

mod de {
    use super::{RidReply, Route, ScheduleReply, SeatsInfo, TrainInfo};
    use crate::client::RzdRequestId;
    use crate::error::Error as GError;
    use crate::error::RzdErrors;
    use crate::{ReplyResult, ResultList, RzdStationCode, TrainDate, TrainTime};
    use serde::Deserialize;
    type ReplyResultId = ReplyResult<RzdRequestId>;
    type ReplyResultRoutes = ReplyResult<Vec<Route>>;
    use crate::{parse_train_date, parse_train_time};

    impl<'de> serde::Deserialize<'de> for RidReply {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize, Debug)]
            struct RzdMessage {
                #[serde(default)]
                message: String,
            }

            #[derive(Deserialize, Debug)]
            struct RzdMessList {
                #[serde(alias = "msgList")]
                #[serde(default)]
                msg_list: Vec<RzdMessage>,
            }

            #[derive(Deserialize, Debug)]
            struct RzdResult {
                #[serde(default)]
                result: String,

                #[serde(alias = "RID")]
                #[serde(default)]
                rid: RzdRequestId,

                #[serde(default)]
                tp: Vec<RzdMessList>,
            }

            let input = RzdResult::deserialize(deserializer)?;

            let res_type: &str = &(input.result);

            let reply = match res_type {
                "RID" => ReplyResultId::success(input.rid),
                "OK" => {
                    let mut errors: Vec<String> = vec![];
                    for lst in input.tp {
                        let mut e: Vec<String> = lst
                            .msg_list
                            .into_iter()
                            .map(|m| {
                                let mut m = m.message.trim();
                                match m.strip_suffix(".") {
                                    Some(r) => m = r,
                                    None => {}
                                }
                                m.to_lowercase()
                            })
                            .filter(|m| !m.is_empty())
                            .collect();

                        errors.append(&mut e);
                    }

                    ReplyResultId::fail(GError::RzdError(RzdErrors::new(errors)))
                }
                _ => ReplyResultId::fail(GError::FailRzdResponse),
            };

            Ok(RidReply(reply))
        }
    }

    impl<'de> serde::Deserialize<'de> for ScheduleReply {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize, Debug)]
            struct Car {
                #[serde(alias = "typeLoc")]
                #[serde(default)]
                car_type: String,

                #[serde(alias = "freeSeats")]
                #[serde(default)]
                free_seats: u32,
            }

            #[derive(Deserialize, Debug)]
            struct Train {
                #[serde(default)]
                number: String,

                #[serde(default)]
                brand: String,

                #[serde(default)]
                carrier: String,

                #[serde(default)]
                route0: String,

                #[serde(default)]
                route1: String,

                #[serde(alias = "routeCode0")]
                #[serde(default)]
                route_code0: RzdStationCode,

                #[serde(alias = "routeCode1")]
                #[serde(default)]
                route_code1: RzdStationCode,

                #[serde(default)]
                station0: String,

                #[serde(default)]
                station1: String,

                #[serde(default)]
                date0: String,

                #[serde(default)]
                time0: String,

                #[serde(default)]
                date1: String,

                #[serde(default)]
                time1: String,

                #[serde(alias = "timeInWay")]
                #[serde(default)]
                trip_duration: String,

                #[serde(alias = "stList")]
                #[serde(default)]
                st_list: String,

                #[serde(default)]
                cars: Vec<Car>,
            }

            #[derive(Deserialize, Debug)]
            struct RzdRoute {
                #[serde(alias = "from")]
                #[serde(default)]
                from_name: String,

                #[serde(alias = "fromCode")]
                #[serde(default)]
                from_code: RzdStationCode,

                #[serde(alias = "where")]
                #[serde(default)]
                to_name: String,

                #[serde(alias = "whereCode")]
                #[serde(default)]
                to_code: RzdStationCode,

                #[serde(default)]
                list: Vec<Train>,

                #[serde(alias = "msgList")]
                #[serde(default)]
                messages: Vec<RzdMessage>,
            }

            #[derive(Deserialize, Debug)]
            struct RzdMessage {
                #[serde(default)]
                message: String,
            }

            #[derive(Deserialize, Debug)]
            struct RzdResult {
                #[serde(default)]
                result: String,

                #[serde(default)]
                tp: Vec<RzdRoute>,
            }

            let input = RzdResult::deserialize(deserializer)?;

            let res_type: &str = &(input.result);

            match res_type {
                "OK" => {
                    if input.tp.is_empty() {
                        return Ok(ScheduleReply(ReplyResultRoutes::success(vec![])));
                    }
                }
                "RID" => {
                    return Ok(ScheduleReply(ReplyResultRoutes::fail(
                        GError::FailRzdResponse,
                    )));
                }
                _ => {
                    return Ok(ScheduleReply(ReplyResultRoutes::fail(
                        GError::FailRzdResponse,
                    )));
                }
            };

            let mut routes: Vec<Route> = vec![];
            let mut errors: Vec<String> = vec![];
            let mut is_error = false;

            for route_or_err in input.tp {
                if is_error | route_or_err.list.is_empty() {
                    let mut err_list: Vec<String> = route_or_err
                        .messages
                        .into_iter()
                        .map(|m| {
                            let mut m = m.message.trim();
                            match m.strip_suffix(".") {
                                Some(r) => m = r,
                                None => {}
                            }
                            m.to_lowercase()
                        })
                        .filter(|m| !m.is_empty())
                        .collect();

                    errors.append(&mut err_list);

                    is_error = true;
                    continue;
                }

                let mut trains: Vec<TrainInfo> = vec![];
                for train in route_or_err.list {
                    let seats: Vec<SeatsInfo> = train
                        .cars
                        .into_iter()
                        .map(|c| SeatsInfo {
                            free_seats: c.free_seats,
                            seats_type: c.car_type,
                        })
                        .collect();
                    let seats = ResultList::<SeatsInfo>(seats);

                    let date1 = parse_train_date!(train.date0);
                    let time1 = parse_train_time!(train.time0);
                    let date2 = parse_train_date!(train.date1);
                    let time2 = parse_train_time!(train.time1);
                    let duration = parse_train_time!(train.trip_duration);

                    trains.push(TrainInfo {
                        train_number: train.number,
                        train_brand: train.brand,
                        train_type: train.carrier,
                        leaving_route: train.route0,
                        leaving_route_code: train.route_code0,
                        arriving_route: train.route1,
                        arriving_route_code: train.route_code1,
                        leaving_station: train.station0,
                        leaving_date: date1,
                        leaving_time: time1,
                        arriving_station: train.station1,
                        arriving_date: date2,
                        arriving_time: time2,
                        trip_duration: duration,
                        stops: train.st_list,
                        seats,
                    });
                }
                let trains = ResultList::<TrainInfo>::new(trains);

                routes.push(Route {
                    leaving_name: route_or_err.from_name,
                    leaving_code: route_or_err.from_code,
                    arriving_name: route_or_err.to_name,
                    arriving_code: route_or_err.to_code,
                    trains,
                });
            }

            let reply = if is_error {
                ReplyResultRoutes::fail(GError::RzdError(RzdErrors::new(errors)))
            } else {
                ReplyResultRoutes::success(routes)
            };

            Ok(ScheduleReply(reply))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RidReply, Route, ScheduleReply, SeatsInfo, TrainInfo};
    use crate::client::RzdRequestId;
    use crate::{error::Error, RzdErrors};
    use crate::{parse_train_date, parse_train_time};
    use crate::{ResultList, RzdStationCode, TrainDate, TrainTime};

    #[test]
    fn rid_reply_deserialize_test() {
        let answer = r#"{"result":"FAIL","type":"SYSTEM_ERROR","error":"Произошла системная ошибка.","timestamp":"02.04.2022 14:18:02.363"}"#;
        let answer: RidReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::FailRzdResponse;

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"result":"OK","tp":[{"from":"САНКТ-ПЕТЕРБУРГ","fromCode":2004000,"where":"МОСКВА","whereCode":2000000,"date":"01.10.2021","noSeats":false,"defShowTime":"local","state":"Trains","list":[],"msgList":[{"message":"Дата отправления находится за пределами периода предварительной продажи","addInfo":null,"type":"TICKET_SEARCH_MESSAGE"},{"message":"Дата отправления находится за пределами периода 90 дней.","addInfo":null,"type":"TICKET_SEARCH_MESSAGE"}]}],"TransferSearchMode":"SEMI_AUTO","flFPKRoundBonus":false,"AutoTransferMode":false,"discounts":{},"timestamp":"02.04.2022 14:30:25.934"}"#;
        let answer: RidReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = RzdErrors::new(vec![
            "дата отправления находится за пределами периода предварительной продажи".to_string(),
            "дата отправления находится за пределами периода 90 дней".to_string(),
        ]);

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"result":"RID","RID":17355769877,"timestamp":"02.04.2022 18:31:00.189"}"#;
        let answer: RidReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = RzdRequestId::new(17355769877);

        assert!(answer.success);
        assert_eq!(answer.value, data);
    }

    #[test]
    fn train_list_deserialize_test() {
        let answer = r#"{"result":"OK","tp":[{"from":"САНКТ-ПЕТЕРБУРГ","fromCode":2004000,"where":"МОСКВА","whereCode":2000000,"date":"01.04.2022","noSeats":false,"defShowTime":"local","state":"Trains","list":[{"number":"119А","number2":"119А","type":0,"typeEx":0,"depth":89,"new":false,"elReg":true,"deferredPayment":false,"varPrice":true,"code0":2004001,"code1":2001025,"bEntire":true,"trainName":"","brand":"","carrier":"ФПК","route0":"С-ПЕТЕР-ГЛ","route1":"БЕЛГОРОД","routeCode0":2004001,"routeCode1":2014370,"trDate0":"01.04.2022","trTime0":"00:11","station0":"САНКТ-ПЕТЕРБУРГ-ГЛАВН. (МОСКОВСКИЙ ВОКЗАЛ)","station1":"МОСКВА ВК ВОСТОЧНЫЙ (ТПУ ЧЕРКИЗОВО)","date0":"01.04.2022","time0":"00:11","date1":"01.04.2022","time1":"10:08","timeInWay":"09:57","flMsk":3,"train_id":0,"cars":[{"carDataType":1,"itype":1,"type":"Плац","typeLoc":"Плацкартный","freeSeats":121,"pt":436,"tariff":1459,"servCls":"3Б"},{"carDataType":1,"itype":3,"type":"Сид","typeLoc":"Сидячий","freeSeats":106,"pt":237,"tariff":795,"servCls":"2С"},{"carDataType":1,"itype":4,"type":"Купе","typeLoc":"Купе","freeSeats":66,"pt":745,"tariff":2489,"servCls":"2К"},{"carDataType":1,"itype":4,"type":"Купе","typeLoc":"Купе","freeSeats":2,"pt":745,"tariff":1362,"servCls":"2К","disabledPerson":true}],"disabledType":true,"addCompLuggageNum":16,"addCompLuggage":true,"addHandLuggage":true},{"number":"713В","number2":"713В","type":0,"typeEx":0,"depth":89,"new":false,"elReg":true,"deferredPayment":false,"varPrice":true,"code0":2004006,"code1":2001025,"bEntire":true,"trainName":"","brandLogo":true,"brand":"СТРИЖ","brandId":19,"carrier":"ФПК","route0":"С-ПЕТ-ЛАД","route1":"САМАРА","routeCode0":2004006,"routeCode1":2024000,"trDate0":"01.04.2022","trTime0":"00:20","station0":"САНКТ-ПЕТЕРБУРГ (ЛАДОЖСКИЙ ВОКЗАЛ)","station1":"МОСКВА ВК ВОСТОЧНЫЙ (ТПУ ЧЕРКИЗОВО)","date0":"01.04.2022","time0":"00:20","date1":"01.04.2022","time1":"05:34","timeInWay":"05:14","flMsk":3,"train_id":0,"cars":[{"carDataType":1,"itype":6,"type":"Люкс","typeLoc":"СВ","freeSeats":48,"pt":802,"tariff":2679,"servCls":"1Е"},{"carDataType":1,"itype":3,"type":"Сид","typeLoc":"Сидячий","freeSeats":29,"pt":527,"tariff":1762,"servCls":"1Р"},{"carDataType":1,"itype":4,"type":"Купе","typeLoc":"Купе","freeSeats":51,"pt":679,"tariff":2269,"servCls":"2А"}],"addCompLuggage":false,"addHandLuggage":true},{"number":"725Ч","number2":"725Ч","type":0,"typeEx":0,"depth":89,"new":false,"elReg":true,"deferredPayment":false,"varPrice":false,"code0":2004001,"code1":2006004,"bEntire":true,"trainName":"","brandLogo":true,"brand":"ЛАСТОЧКА","brandId":13,"carrier":"ДОСС","route0":"С-ПЕТЕР-ГЛ","route1":"МОСКВА ОКТ","routeCode0":2004001,"routeCode1":2006004,"trDate0":"01.04.2022","trTime0":"15:16","station0":"САНКТ-ПЕТЕРБУРГ-ГЛАВН. (МОСКОВСКИЙ ВОКЗАЛ)","station1":"МОСКВА ОКТЯБРЬСКАЯ (ЛЕНИНГРАДСКИЙ ВОКЗАЛ)","date0":"01.04.2022","time0":"15:16","date1":"01.04.2022","time1":"21:58","timeInWay":"06:42","flMsk":3,"train_id":0,"cars":[{"carDataType":1,"itype":3,"type":"Сид","typeLoc":"Сидячий","freeSeats":319,"pt":328,"tariff":1099,"servCls":"1П"},{"carDataType":1,"itype":3,"type":"Сид","typeLoc":"Сидячий","freeSeats":2,"pt":328,"tariff":660,"servCls":"2Ж","disabledPerson":true}],"seatCars":[{"carDataType":2,"servCls":"2Ж","tariff":"1099","tariff2":null,"itype":3,"type":"Сид","typeLoc":"Базовый","freeSeats":129},{"carDataType":2,"servCls":"2Ж","tariff":"660","tariff2":null,"itype":3,"type":"Сид","typeLoc":"Базовый","freeSeats":2,"disabledPerson":true},{"carDataType":2,"servCls":"2П","tariff":"1199","tariff2":null,"itype":3,"type":"Сид","typeLoc":"Эконом","freeSeats":180},{"carDataType":2,"servCls":"1П","tariff":"2060","tariff2":"2299","itype":3,"type":"Сид","typeLoc":"Бизнес класс","freeSeats":10}],"disabledType":true,"nonRefundable":true}],"msgList":[]}],"TransferSearchMode":"SEMI_AUTO","flFPKRoundBonus":false,"AutoTransferMode":false,"discounts":{},"timestamp":"20.03.2022 18:28:31.458"}"#;
        let answer: ScheduleReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = vec![Route {
            leaving_name: "САНКТ-ПЕТЕРБУРГ".to_string(),
            leaving_code: RzdStationCode(2004000),
            arriving_name: "МОСКВА".to_string(),
            arriving_code: RzdStationCode(2000000),
            trains: ResultList::<TrainInfo>(vec![
                TrainInfo {
                    train_number: "119А".to_string(),
                    train_brand: "".to_string(),
                    train_type: "ФПК".to_string(),
                    leaving_route: "С-ПЕТЕР-ГЛ".to_string(),
                    leaving_route_code: RzdStationCode::new(2004001),
                    arriving_route: "БЕЛГОРОД".to_string(),
                    arriving_route_code: RzdStationCode::new(2014370),
                    leaving_station: "САНКТ-ПЕТЕРБУРГ-ГЛАВН. (МОСКОВСКИЙ ВОКЗАЛ)".to_string(),
                    leaving_date: parse_train_date!("01.04.2022"),
                    leaving_time: parse_train_time!("00:11"),
                    arriving_station: "МОСКВА ВК ВОСТОЧНЫЙ (ТПУ ЧЕРКИЗОВО)".to_string(),
                    arriving_date: parse_train_date!("01.04.2022"),
                    arriving_time: parse_train_time!("10:08"),
                    trip_duration: parse_train_time!("09:57"),
                    stops: String::new(),
                    seats: ResultList::<SeatsInfo>(vec![
                        SeatsInfo {
                            free_seats: 121,
                            seats_type: "Плацкартный".to_string(),
                        },
                        SeatsInfo {
                            free_seats: 106,
                            seats_type: "Сидячий".to_string(),
                        },
                        SeatsInfo {
                            free_seats: 66,
                            seats_type: "Купе".to_string(),
                        },
                        SeatsInfo {
                            free_seats: 2,
                            seats_type: "Купе".to_string(),
                        },
                    ]),
                },
                TrainInfo {
                    train_number: "713В".to_string(),
                    train_brand: "СТРИЖ".to_string(),
                    train_type: "ФПК".to_string(),
                    leaving_route: "С-ПЕТ-ЛАД".to_string(),
                    leaving_route_code: RzdStationCode::new(2004006),
                    arriving_route: "САМАРА".to_string(),
                    arriving_route_code: RzdStationCode::new(2024000),
                    leaving_station: "САНКТ-ПЕТЕРБУРГ (ЛАДОЖСКИЙ ВОКЗАЛ)".to_string(),
                    leaving_date: parse_train_date!("01.04.2022"),
                    leaving_time: parse_train_time!("00:20"),
                    arriving_station: "МОСКВА ВК ВОСТОЧНЫЙ (ТПУ ЧЕРКИЗОВО)".to_string(),
                    arriving_date: parse_train_date!("01.04.2022"),
                    arriving_time: parse_train_time!("05:34"),
                    trip_duration: parse_train_time!("05:14"),
                    stops: String::new(),
                    seats: ResultList::<SeatsInfo>(vec![
                        SeatsInfo {
                            free_seats: 48,
                            seats_type: "СВ".to_string(),
                        },
                        SeatsInfo {
                            free_seats: 29,
                            seats_type: "Сидячий".to_string(),
                        },
                        SeatsInfo {
                            free_seats: 51,
                            seats_type: "Купе".to_string(),
                        },
                    ]),
                },
                TrainInfo {
                    train_number: "725Ч".to_string(),
                    train_brand: "ЛАСТОЧКА".to_string(),
                    train_type: "ДОСС".to_string(),
                    leaving_route: "С-ПЕТЕР-ГЛ".to_string(),
                    leaving_route_code: RzdStationCode::new(2004001),
                    arriving_route: "МОСКВА ОКТ".to_string(),
                    arriving_route_code: RzdStationCode::new(2006004),
                    leaving_station: "САНКТ-ПЕТЕРБУРГ-ГЛАВН. (МОСКОВСКИЙ ВОКЗАЛ)".to_string(),
                    leaving_date: parse_train_date!("01.04.2022"),
                    leaving_time: parse_train_time!("15:16"),
                    arriving_station: "МОСКВА ОКТЯБРЬСКАЯ (ЛЕНИНГРАДСКИЙ ВОКЗАЛ)".to_string(),
                    arriving_date: parse_train_date!("01.04.2022"),
                    arriving_time: parse_train_time!("21:58"),
                    trip_duration: parse_train_time!("06:42"),
                    stops: String::new(),
                    seats: ResultList::<SeatsInfo>(vec![
                        SeatsInfo {
                            free_seats: 319,
                            seats_type: "Сидячий".to_string(),
                        },
                        SeatsInfo {
                            free_seats: 2,
                            seats_type: "Сидячий".to_string(),
                        },
                    ]),
                },
            ]),
        }];

        assert!(answer.success);
        assert_eq!(answer.value, data);
    }

    #[test]
    fn train_electric_list_deserialize_test() {
        let answer = r#"{"result":"OK","tp":[{"from":"САНКТ-ПЕТЕРБУРГ","fromCode":2004000,"where":"ПУПЫШЕВО","whereCode":2005283,"date":"01.04.2022","noSeats":false,"defShowTime":"local","state":"Trains","list":[{"number":"6201","number2":"6201","type":1,"typeEx":1,"subt":1,"subtrainCatName":"Пассажирский","elReg":false,"deferredPayment":false,"varPrice":false,"code0":2004001,"code1":2005283,"bEntire":true,"trainName":"","brand":"","carrier":"СЗППК","route0":"САНКТ-ПЕТЕРБУРГ-ГЛАВН.","route1":"ВОЛХОВСТРОЙ 1","routeCode0":2004001,"routeCode1":2004672,"station0":"САНКТ-ПЕТЕРБУРГ-ГЛАВН. (МОСКОВСКИЙ ВОКЗАЛ)","station1":"ПУПЫШЕВО","date0":"01.04.2022","time0":"05:50","date1":"01.04.2022","time1":"07:53","timeInWay":"02:03","flMsk":3,"stList":"Везде, кроме: ОСТ.ПУНКТ 5 КМ, УСТЬ-ТОСНЕНСКАЯ, ОСТ.ПУНКТ 77 КМ","mvMode":"Ежедневно","chWarn":false,"relev":true,"onWay":false,"suburbanTrainName":null,"subTabloVisible":0,"train_id":2212797,"cars":[]},{"number":"6208","number2":"6208","type":1,"typeEx":1,"subt":1,"subtrainCatName":"Пассажирский","elReg":false,"deferredPayment":false,"varPrice":false,"code0":2004006,"code1":2005283,"bEntire":true,"trainName":"","brand":"","carrier":"СЗППК","route0":"САНКТ-ПЕТЕРБУРГ ЛАДОЖ.","route1":"ВОЛХОВСТРОЙ 1","routeCode0":2004006,"routeCode1":2004672,"station0":"САНКТ-ПЕТЕРБУРГ (ЛАДОЖСКИЙ ВОКЗАЛ)","station1":"ПУПЫШЕВО","date0":"01.04.2022","time0":"10:29","date1":"01.04.2022","time1":"12:36","timeInWay":"02:07","flMsk":3,"stList":"Везде","mvMode":"Ежедневно","chWarn":false,"relev":true,"onWay":false,"suburbanTrainName":null,"subTabloVisible":0,"train_id":2213151,"cars":[]},{"number":"7406","number2":"7406","type":1,"typeEx":1,"subt":2,"subtrainCatName":"Экспресс","elReg":false,"deferredPayment":false,"varPrice":false,"code0":2004006,"code1":2005283,"bEntire":true,"trainName":"","brand":"","carrier":"СЗППК","route0":"САНКТ-ПЕТЕРБУРГ ЛАДОЖ.","route1":"ТИХВИН","routeCode0":2004006,"routeCode1":2004669,"station0":"САНКТ-ПЕТЕРБУРГ (ЛАДОЖСКИЙ ВОКЗАЛ)","station1":"ПУПЫШЕВО","date0":"01.04.2022","time0":"18:51","date1":"01.04.2022","time1":"20:28","timeInWay":"01:37","flMsk":3,"stList":"МГА, ЖИХАРЕВО, ПУПЫШЕВО","mvMode":"Кроме субботы","chWarn":false,"relev":true,"onWay":false,"suburbanTrainName":"Ласточка","subTabloVisible":0,"train_id":2205039,"cars":[]},{"number":"6218","number2":"6218","type":1,"typeEx":1,"subt":1,"subtrainCatName":"Пассажирский","elReg":false,"deferredPayment":false,"varPrice":false,"code0":2004006,"code1":2005283,"bEntire":true,"trainName":"","brand":"","carrier":"СЗППК","route0":"САНКТ-ПЕТЕРБУРГ ЛАДОЖ.","route1":"ВОЛХОВСТРОЙ 1","routeCode0":2004006,"routeCode1":2004672,"station0":"САНКТ-ПЕТЕРБУРГ (ЛАДОЖСКИЙ ВОКЗАЛ)","station1":"ПУПЫШЕВО","date0":"01.04.2022","time0":"21:33","date1":"01.04.2022","time1":"23:31","timeInWay":"01:58","flMsk":3,"stList":"ОСТ.ПУНКТ 5 КМ, ОСТ.ПУНКТ 7 КМ, МЯГЛОВО, ОСТ.ПУНКТ 11 КМ","stListX":"ОСТ.ПУНКТ 5 КМ, ОСТ.ПУНКТ 7 КМ, МЯГЛОВО, ОСТ.ПУНКТ 11 КМ, КОЛТУШИ, ОСТ.ПУНКТ 16 КМ, МАНУШКИНО, ОСТ.ПУНКТ 20 КМ, ОСТРОВКИ, ОСТ.ПУНКТ 26 КМ, ГЕРОЙСКАЯ, ПАВЛОВО НА НЕВЕ, ГОРЫ, ОСТ.ПУНКТ 45 КМ, МГА, МИХАЙЛОВСКАЯ, АПРАКСИН, ОСТ.ПУНКТ 63 КМ, НАЗИЯ, ЖИХАРЕВО, ВОЙБОКАЛО, ОСТ.ПУНКТ 95КМ, НОВЫЙ БЫТ, ОСТ.ПУНКТ 106 КМ, ПУПЫШЕВО","mvMode":"Ежедневно","chWarn":false,"relev":true,"onWay":false,"suburbanTrainName":null,"subTabloVisible":0,"train_id":2213204,"cars":[]}],"msgList":[]}],"TransferSearchMode":"SEMI_AUTO","flFPKRoundBonus":false,"AutoTransferMode":false,"discounts":{},"timestamp":"20.03.2022 21:21:54.543"}"#;
        let answer: ScheduleReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = vec![Route {
            leaving_name: "САНКТ-ПЕТЕРБУРГ".to_string(),
            leaving_code: RzdStationCode(2004000),
            arriving_name: "ПУПЫШЕВО".to_string(),
            arriving_code: RzdStationCode(2005283),
            trains: ResultList::<TrainInfo>(vec![
                TrainInfo {
                    train_number: "6201".to_string(),
                    train_brand: "".to_string(),
                    train_type: "СЗППК".to_string(),
                    leaving_route: "САНКТ-ПЕТЕРБУРГ-ГЛАВН.".to_string(),
                    leaving_route_code: RzdStationCode::new(2004001),
                    arriving_route: "ВОЛХОВСТРОЙ 1".to_string(),
                    arriving_route_code: RzdStationCode::new(2004672),
                    leaving_station: "САНКТ-ПЕТЕРБУРГ-ГЛАВН. (МОСКОВСКИЙ ВОКЗАЛ)".to_string(),
                    leaving_date: parse_train_date!("01.04.2022"),
                    leaving_time: parse_train_time!("05:50"),
                    arriving_station: "ПУПЫШЕВО".to_string(),
                    arriving_date: parse_train_date!("01.04.2022"),
                    arriving_time: parse_train_time!("07:53"),
                    trip_duration: parse_train_time!("02:03"),
                    stops: "Везде, кроме: ОСТ.ПУНКТ 5 КМ, УСТЬ-ТОСНЕНСКАЯ, ОСТ.ПУНКТ 77 КМ"
                        .to_string(),
                    seats: ResultList::<SeatsInfo>(vec![]),
                },
                TrainInfo {
                    train_number: "6208".to_string(),
                    train_brand: "".to_string(),
                    train_type: "СЗППК".to_string(),
                    leaving_route: "САНКТ-ПЕТЕРБУРГ ЛАДОЖ.".to_string(),
                    leaving_route_code: RzdStationCode::new(2004006),
                    arriving_route: "ВОЛХОВСТРОЙ 1".to_string(),
                    arriving_route_code: RzdStationCode::new(2004672),
                    leaving_station: "САНКТ-ПЕТЕРБУРГ (ЛАДОЖСКИЙ ВОКЗАЛ)".to_string(),
                    leaving_date: parse_train_date!("01.04.2022"),
                    leaving_time: parse_train_time!("10:29"),
                    arriving_station: "ПУПЫШЕВО".to_string(),
                    arriving_date: parse_train_date!("01.04.2022"),
                    arriving_time: parse_train_time!("12:36"),
                    trip_duration: parse_train_time!("02:07"),
                    stops: "Везде".to_string(),
                    seats: ResultList::<SeatsInfo>(vec![]),
                },
                TrainInfo {
                    train_number: "7406".to_string(),
                    train_brand: "".to_string(),
                    train_type: "СЗППК".to_string(),
                    leaving_route: "САНКТ-ПЕТЕРБУРГ ЛАДОЖ.".to_string(),
                    leaving_route_code: RzdStationCode::new(2004006),
                    arriving_route: "ТИХВИН".to_string(),
                    arriving_route_code: RzdStationCode::new(2004669),
                    leaving_station: "САНКТ-ПЕТЕРБУРГ (ЛАДОЖСКИЙ ВОКЗАЛ)".to_string(),
                    leaving_date: parse_train_date!("01.04.2022"),
                    leaving_time: parse_train_time!("18:51"),
                    arriving_station: "ПУПЫШЕВО".to_string(),
                    arriving_date: parse_train_date!("01.04.2022"),
                    arriving_time: parse_train_time!("20:28"),
                    trip_duration: parse_train_time!("01:37"),
                    stops: "МГА, ЖИХАРЕВО, ПУПЫШЕВО".to_string(),
                    seats: ResultList::<SeatsInfo>(vec![]),
                },
                TrainInfo {
                    train_number: "6218".to_string(),
                    train_brand: "".to_string(),
                    train_type: "СЗППК".to_string(),
                    leaving_route: "САНКТ-ПЕТЕРБУРГ ЛАДОЖ.".to_string(),
                    leaving_route_code: RzdStationCode::new(2004006),
                    arriving_route: "ВОЛХОВСТРОЙ 1".to_string(),
                    arriving_route_code: RzdStationCode::new(2004672),
                    leaving_station: "САНКТ-ПЕТЕРБУРГ (ЛАДОЖСКИЙ ВОКЗАЛ)".to_string(),
                    leaving_date: parse_train_date!("01.04.2022"),
                    leaving_time: parse_train_time!("21:33"),
                    arriving_station: "ПУПЫШЕВО".to_string(),
                    arriving_date: parse_train_date!("01.04.2022"),
                    arriving_time: parse_train_time!("23:31"),
                    trip_duration: parse_train_time!("01:58"),
                    stops: "ОСТ.ПУНКТ 5 КМ, ОСТ.ПУНКТ 7 КМ, МЯГЛОВО, ОСТ.ПУНКТ 11 КМ".to_string(),
                    seats: ResultList::<SeatsInfo>(vec![]),
                },
            ]),
        }];

        assert!(answer.success);
        assert_eq!(answer.value, data);
    }
}
