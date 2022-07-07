use reqwest::blocking::Response;
use serde::Serialize;
use std::fmt;
use url::form_urlencoded::byte_serialize;

use crate::client::{RzdClientInterface, RzdQueryType, RzdRequestId};
use crate::{error::Error, Result};
use crate::{ReplyResult, ResultList, RouteDirection, RzdStationCode, TrainDate, TrainTime};

/// Train info search.
pub struct TrainSearch {
    leaving_code: RzdStationCode,
    leaving_date: TrainDate,
    leaving_time: TrainTime,
    arriving_code: RzdStationCode,
    train_number: String,
}

impl TrainSearch {
    /// Takes departure and arrival station codes, departure and arrival date,
    /// departure time and train number.
    ///
    /// # Errors
    ///
    /// The method fails if the train number is empty.
    pub fn new(
        leaving_code: RzdStationCode,
        arriving_code: RzdStationCode,
        leaving_date: TrainDate,
        leaving_time: TrainTime,
        train_number: &str,
    ) -> Result<Self> {
        let train_number = train_number.trim().to_uppercase();

        if train_number.is_empty() {
            return Err(Error::EmptyTrainNumber);
        }
        debug!("query: {}", train_number);

        Ok(TrainSearch {
            leaving_code,
            leaving_date,
            leaving_time,
            arriving_code,
            train_number,
        })
    }
}

impl RzdClientInterface<ResultList<TrainItem>> for TrainSearch {
    fn query_type(&self) -> RzdQueryType {
        RzdQueryType::WithId
    }

    fn request_id(&self) -> String {
        let train_encoded: String = byte_serialize(self.train_number.as_bytes()).collect();

        format!(
            "https://pass.rzd.ru/timetable/public/ru\
            ?layer_id=5764\
            &dir={}\
            &code0={}\
            &dt0={}\
            &time0={}\
            &code1={}\
            &tnum0={}",
            RouteDirection::OneWay as u8,
            self.leaving_code,
            self.leaving_date,
            self.leaving_time,
            self.arriving_code,
            train_encoded
        )
    }

    fn request_data(&self, id: RzdRequestId) -> String {
        format!(
            "https://pass.rzd.ru/timetable/public/ru\
            ?layer_id=5764\
            &rid={}",
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

    fn deserialize_reply_data(&self, response: Response) -> Result<Option<ResultList<TrainItem>>> {
        let reply: TrainReply = match response.json() {
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
/// Number of free seats on the train car, seat type and price info.
pub struct SeatsInfo {
    free_seats: u32,
    seats_type: String,
    price: String,
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

    /// Returns the seat price.
    #[inline]
    pub fn price(&self) -> &str {
        &self.price
    }
}

impl fmt::Display for SeatsInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} по {}",
            self.free_seats, self.seats_type, self.price
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Name, URL and cost of insurance.
pub struct InsuranceInfo {
    name: String,
    url: String,
    price: String,
}

impl InsuranceInfo {
    /// Returns the short name of insurance.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns URL with insurance info.
    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the insurance price.
    #[inline]
    pub fn price(&self) -> &str {
        &self.price
    }
}

impl fmt::Display for InsuranceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {} р. ({})", self.name, self.price, self.url)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Train car info.
pub struct TrainCar {
    number: String,
    type_loc: String,
    service_class: String,
    services: ResultList<String>,
    tariff1: String,
    tariff2: String,
    tariff_service: String,
    carrier: String,
    insurance: Option<InsuranceInfo>,
    seats: ResultList<SeatsInfo>,
    places: String,
}

impl TrainCar {
    /// Returns the number of the train car.
    #[inline]
    pub fn number(&self) -> &str {
        &self.number
    }

    /// Returns the seat type.
    #[inline]
    pub fn type_loc(&self) -> &str {
        &self.type_loc
    }

    /// Returns the class of service.
    #[inline]
    pub fn service_class(&self) -> &str {
        &self.service_class
    }

    /// Returns the immutable list of available services.
    #[inline]
    pub fn services(&self) -> &ResultList<String> {
        &self.services
    }

    /// Returns the mutable list of available services.
    pub fn services_mut(&mut self) -> &mut ResultList<String> {
        &mut self.services
    }

    /// Returns the price of a seat.
    #[inline]
    pub fn price1(&self) -> &str {
        &self.tariff1
    }

    /// Returns the price of a seat.
    #[inline]
    pub fn price2(&self) -> &str {
        &self.tariff2
    }

    /// Returns the price of services.
    #[inline]
    pub fn price_service(&self) -> &str {
        &self.tariff_service
    }

    /// Returns the carrier.
    #[inline]
    pub fn carrier(&self) -> &str {
        &self.carrier
    }

    /// Returns insurance info of a seat.
    #[inline]
    pub fn insurance(&self) -> &Option<InsuranceInfo> {
        &self.insurance
    }

    /// Returns the immutable list of seats.
    #[inline]
    pub fn seats(&self) -> &ResultList<SeatsInfo> {
        &self.seats
    }

    /// Returns the mutable list of seats.
    pub fn seats_mut(&mut self) -> &mut ResultList<SeatsInfo> {
        &mut self.seats
    }

    /// Returns the available places.
    #[inline]
    pub fn places(&self) -> &str {
        &self.places
    }
}

impl fmt::Display for TrainCar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Вагон {}, {}, класс {}:\n",
            self.number, self.type_loc, self.service_class
        )?;
        if !self.services.is_empty() {
            write!(f, "\tуслуги: {}\n", self.services)?;
        }
        write!(f, "\tтарифы:")?;
        if !self.tariff1.is_empty() {
            write!(f, "\t\t{} (билет)\n", self.tariff1)?;
        }
        if !self.tariff2.is_empty() {
            write!(f, "\t\t{} (плацкарта)\n", self.tariff2)?;
        }
        if !self.tariff_service.is_empty() {
            write!(f, "\t\t{} (сервис)\n", self.tariff_service)?;
        }
        if !self.insurance.is_none() {
            write!(f, "\tстраховка: {}\n", self.insurance.as_ref().unwrap())?;
        }
        write!(f, "\tместа: {}\n", self.places)?;
        write!(f, "\tвсего мест:")?;
        for s in self.seats.iter() {
            write!(f, "\t\t{}\n", s)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
/// Info about the train.
pub struct TrainItem {
    train_number: String,
    leaving_date: Option<TrainDate>,
    leaving_time: Option<TrainTime>,
    arriving_date: Option<TrainDate>,
    arriving_time: Option<TrainTime>,
    leaving_station_name: String,
    arriving_station_name: String,
    leaving_station_code: RzdStationCode,
    arriving_station_code: RzdStationCode,
    cars: ResultList<TrainCar>,
}

impl TrainItem {
    /// Returns the train number.
    #[inline]
    pub fn train_number(&self) -> &str {
        &self.train_number
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

    /// Returns the name of the departure station.
    #[inline]
    pub fn leaving_station_name(&self) -> &str {
        &self.leaving_station_name
    }

    /// Returns the name of the arrival station.
    #[inline]
    pub fn arriving_station_name(&self) -> &str {
        &self.arriving_station_name
    }

    /// Returns the RZD code of the departure station.
    #[inline]
    pub fn leaving_station_code(&self) -> RzdStationCode {
        self.leaving_station_code
    }

    /// Returns the RZD code of the arrival station.
    #[inline]
    pub fn arriving_station_code(&self) -> RzdStationCode {
        self.arriving_station_code
    }

    /// Returns the immutable list of train cars.
    #[inline]
    pub fn cars(&self) -> &ResultList<TrainCar> {
        &self.cars
    }

    /// Returns the mutable list of train cars.
    pub fn cars_mut(&mut self) -> &mut ResultList<TrainCar> {
        &mut self.cars
    }
}

impl fmt::Display for TrainItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Поезд № \"{}\"\n", self.train_number)?;
        write!(
            f,
            "Станция отправления: \"{}\" {}\n",
            self.leaving_station_name, self.leaving_station_code
        )?;
        write!(
            f,
            "Станция прибытия: \"{}\" {}\n",
            self.arriving_station_name, self.arriving_station_code
        )?;
        for c in self.cars.iter() {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct TrainReply(ReplyResult<Vec<TrainItem>>);

mod de {
    use super::{InsuranceInfo, RidReply, SeatsInfo, TrainCar, TrainItem, TrainReply};
    use crate::client::RzdRequestId;
    use crate::des::des_null_to_default;
    use crate::error::Error as GError;
    use crate::error::RzdErrors;
    use crate::{ReplyResult, ResultList, RzdStationCode, TrainDate, TrainTime};
    use serde::Deserialize;
    type ReplyResultId = ReplyResult<RzdRequestId>;
    type ReplyResultTrains = ReplyResult<Vec<TrainItem>>;
    use crate::{parse_station_code, parse_train_date, parse_train_time};

    impl<'de> serde::Deserialize<'de> for RidReply {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize, Debug)]
            struct RzdResult {
                #[serde(default)]
                result: String,

                #[serde(alias = "RID")]
                #[serde(default)]
                rid: RzdRequestId,
            }

            let input = RzdResult::deserialize(deserializer)?;

            let res_type: &str = &(input.result);

            let reply = match res_type {
                "RID" => ReplyResultId::success(input.rid),
                _ => ReplyResultId::fail(GError::FailRzdResponse),
            };

            Ok(RidReply(reply))
        }
    }

    impl<'de> serde::Deserialize<'de> for TrainReply {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize, Debug)]
            struct Seats {
                #[serde(alias = "free")]
                #[serde(default)]
                free_seats: u32,

                #[serde(alias = "label")]
                #[serde(default)]
                type_seats: String,

                #[serde(default)]
                tariff: String,
            }

            #[allow(dead_code)]
            #[derive(Deserialize, Debug)]
            struct RzdService {
                #[serde(default)]
                id: u32,

                #[serde(default)]
                description: String,
            }

            #[derive(Deserialize, Debug)]
            struct TrainCars {
                #[serde(default)]
                cnumber: String,

                #[serde(alias = "typeLoc")]
                #[serde(default)]
                type_loc: String,

                #[serde(alias = "clsType")]
                #[serde(default)]
                cls_type: String,

                #[serde(default)]
                services: Vec<RzdService>,

                #[serde(default)]
                tariff: String,

                #[serde(deserialize_with = "des_null_to_default")]
                tariff2: String,

                #[serde(alias = "tariffServ")]
                #[serde(deserialize_with = "des_null_to_default")]
                tariff_serv: String,

                #[serde(default)]
                carrier: String,

                #[serde(alias = "insuranceTypeId")]
                #[serde(default)]
                insurance_id: u32,

                #[serde(default)]
                seats: Vec<Seats>,

                #[serde(default)]
                places: String,
            }

            #[derive(Deserialize, Debug)]
            struct RzdTrain {
                #[serde(default)]
                result: String,

                #[serde(default)]
                number: String,

                #[serde(default)]
                date0: String,

                #[serde(default)]
                time0: String,

                #[serde(default)]
                date1: String,

                #[serde(default)]
                time1: String,

                #[serde(default)]
                station0: String,

                #[serde(default)]
                station1: String,

                #[serde(default)]
                code0: String,

                #[serde(default)]
                code1: String,

                #[serde(default)]
                cars: Vec<TrainCars>,

                #[serde(default)]
                error: String,
            }

            #[derive(Deserialize, Debug)]
            struct RzdResult {
                #[serde(default)]
                result: String,

                #[serde(default)]
                lst: Vec<RzdTrain>,

                #[serde(alias = "insuranceCompany")]
                #[serde(default)]
                insurance: Vec<RzdInsurance>,
            }

            #[derive(Deserialize, Debug)]
            struct RzdInsurance {
                #[serde(default)]
                id: u32,

                #[serde(alias = "shortName")]
                #[serde(default)]
                short_name: String,

                #[serde(alias = "offerUrl")]
                #[serde(default)]
                offer_url: String,

                #[serde(alias = "insuranceCost")]
                #[serde(default)]
                insurance_cost: u32,
            }

            let input = RzdResult::deserialize(deserializer)?;

            let res_type: &str = &(input.result);

            match res_type {
                "OK" => {
                    if input.lst.is_empty() {
                        return Ok(TrainReply(ReplyResultTrains::success(vec![])));
                    }
                }
                "RID" => {
                    return Ok(TrainReply(ReplyResultTrains::fail(GError::FailRzdResponse)));
                }
                _ => {
                    return Ok(TrainReply(ReplyResultTrains::fail(GError::FailRzdResponse)));
                }
            };

            let mut trains: Vec<TrainItem> = vec![];
            for train in input.lst {
                let res_error: &str = &(train.result);
                if res_error != "OK" {
                    let mut err = train.error.trim().to_lowercase();
                    match err.strip_suffix(".") {
                        Some(r) => err = r.to_string(),
                        None => {}
                    }
                    let err = GError::RzdError(RzdErrors::new(vec![err]));
                    return Ok(TrainReply(ReplyResultTrains::fail(err)));
                }

                let mut cars: Vec<TrainCar> = vec![];
                for car in train.cars {
                    let seats: Vec<SeatsInfo> = car
                        .seats
                        .into_iter()
                        .map(|s| SeatsInfo {
                            free_seats: s.free_seats,
                            seats_type: s.type_seats,
                            price: s.tariff,
                        })
                        .collect();

                    let services: Vec<String> =
                        car.services.into_iter().map(|s| s.description).collect();

                    let mut insurance = None;
                    for ins in input.insurance.iter() {
                        if ins.id == car.insurance_id {
                            insurance = Some(InsuranceInfo {
                                name: ins.short_name.clone(),
                                url: ins.offer_url.clone(),
                                price: ins.insurance_cost.to_string(),
                            });
                            break;
                        }
                    }

                    cars.push(TrainCar {
                        number: car.cnumber,
                        type_loc: car.type_loc,
                        service_class: car.cls_type,
                        services: ResultList(services),
                        tariff1: car.tariff,
                        tariff2: car.tariff2,
                        tariff_service: car.tariff_serv,
                        carrier: car.carrier,
                        insurance,
                        seats: ResultList(seats),
                        places: car.places,
                    });
                }

                trains.push(TrainItem {
                    train_number: train.number,
                    leaving_date: parse_train_date!(train.date0),
                    leaving_time: parse_train_time!(train.time0),
                    arriving_date: parse_train_date!(train.date1),
                    arriving_time: parse_train_time!(train.time1),
                    leaving_station_name: train.station0,
                    arriving_station_name: train.station1,
                    leaving_station_code: parse_station_code!(train.code0),
                    arriving_station_code: parse_station_code!(train.code1),
                    cars: ResultList(cars),
                });
            }

            Ok(TrainReply(ReplyResultTrains::success(trains)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InsuranceInfo, RidReply, SeatsInfo, TrainCar, TrainItem, TrainReply};
    use crate::client::RzdRequestId;
    use crate::{error::Error, RzdErrors};
    use crate::{parse_train_date, parse_train_time};
    use crate::{ResultList, RzdStationCode, TrainDate, TrainTime};

    #[test]
    fn rid_reply_deserialize_test() {
        let answer = r#"{"result":"FAIL","type":"SYSTEM_ERROR","error":"Произошла системная ошибка.","timestamp":"01.04.2022 13:58:10.003"}"#;
        let answer: RidReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::FailRzdResponse;

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"result":"RID","RID":18605390978,"timestamp":"01.04.2022 13:44:28.459"}"#;
        let answer: RidReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = RzdRequestId::new(18605390978);

        assert!(answer.success);
        assert_eq!(answer.value, data);
    }

    #[test]
    fn train_list_deserialize_test() {
        let answer = r#"{"result":"FAIL","type":"ERROR","error":"Ошибка обмена данными со шлюзом в АСУ «Экспресс-3»","detail":"Ошибка обмена данными со шлюзом в АСУ «Экспресс-3»","timestamp":"01.04.2022 13:55:11.116"}"#;
        let answer: TrainReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::FailRzdResponse;

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"result":"RID","RID":18605390978,"timestamp":"01.04.2022 13:44:28.459"}"#;
        let answer: TrainReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::FailRzdResponse;

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"result":"OK","lst":[{"result":"FAIL","type":"NEGATIVE_RESPONSE","error":"Неверная дата отправления","detail":"Неверная дата отправления","timestamp":"30.03.2022 13:54:43.191"}],"schemes":[],"psaction":null,"childrenAge":10,"motherAndChildAge":1,"partialPayment":false,"timestamp":"01.04.2022 13:54:43.191"}"#;
        let answer: TrainReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = Error::RzdError(RzdErrors::new(
            vec!["неверная дата отправления".to_string()],
        ));

        assert!(!answer.success);
        assert_eq!(answer.error.to_string(), data.to_string());

        let answer = r#"{"result":"OK","lst":[{"result":"OK","number":"001А","number2":"001А","defShowTime":"local","date0":"01.04.2022","time0":"23:55","date1":"02.04.2022","time1":"07:55","type":"СК ФИРМ","virtual":false,"bus":false,"boat":false,"station0":"САНКТ-ПЕТЕРБУРГ-ГЛАВН. (МОСКОВСКИЙ ВОКЗАЛ)","code0":"2004001","station1":"МОСКВА ОКТЯБРЬСКАЯ (ЛЕНИНГРАДСКИЙ ВОКЗАЛ)","code1":"2006004","timeSt0":"","timeSt1":"","route0":"С-ПЕТЕР-ГЛ","route1":"МОСКВА ОКТ","cars":[{"cnumber":"01","type":"Купе","catLabelLoc":"Купе","typeLoc":"Купе","catCode":"Купе","ctypei":4,"ctype":4,"letter":"А","clsType":"2Э","subType":"66К","clsName":"4-местные купе.<p>Вагон повышенной комфортности (рацион питания, санитарно-гигиенический набор*, пресса*, белье).</p><p><strong>Кондиционер, биотуалет в вагоне.</strong></p><p>Вагон с услугой перевозки животных. </p><p>*кроме двухэтажных вагонов</p>","services":[{"id":2,"name":"[иконка сайта] Биотуалет","description":"Биотуалет","hasImage":true},{"id":3,"name":"[иконка сайта] Кондиционер","description":"Кондиционер работает в летний период","hasImage":true},{"id":73,"name":"[иконка сайта] Попутчик","description":"Мультимедийный портал \"Попутчик\"","hasImage":true},{"id":80,"name":"[иконка сайта] Животные 2Э,2Б,2Ф,2Ц","description":"Для провоза мелких животных необходим выкуп всего купе.<br>Провоз мелких животных бесплатный.<br>Для провоза крупной собаки необходим выкуп всего купе.<br>Можно провести только одну крупную собаку.<br>Провоз крупной собаки бесплатный","hasImage":true},{"id":135,"name":"[иконка сайта] Гигиена 1Э 2Э","description":"Гигиенический набор","hasImage":true},{"id":136,"name":"[иконка сайта] Пресса 1Э 2Э","description":"Пресса","hasImage":true},{"id":30,"name":"[иконка сайта] Постель","description":"Постельное белье","hasImage":true}],"tariff":"3966","tariff2":"5090","tariffServ":"766","addSigns":"У1","carrier":"ФПК","carrierId":1,"insuranceFlag":true,"insuranceTypeId":1,"owner":"РЖД/ОКТ","elReg":true,"food":true,"selFood":false,"equippedSIOP":true,"addFood":true,"regularFoodService":false,"noSmok":false,"inetSaleOff":false,"bVip":false,"conferenceRoomFlag":false,"bDeck2":false,"intServiceClass":null,"specialSeatTypes":null,"deferredPayment":false,"varPrice":true,"ferry":false,"seniorTariff":0,"bedding":false,"nonRefundable":false,"addTour":false,"addGoods":true,"addHandLuggage":true,"youth":false,"unior":false,"seats":[{"type":"dn","free":9,"label":"Нижнее","tariff":"3966"},{"type":"up","free":15,"label":"Верхнее","tariff":"3966"}],"places":"002-004,006-010,012-014,016,020-028,030-032","schemeId":830,"schemeInfo":{"dir":"/dbmm/images/61/28209/14","dirVert":"/dbmm/images/61/28216/14","legend":""},"forcedBedding":true,"policyEnabled":true,"msr":true,"medic":true},{"cnumber":"02","type":"Купе","catLabelLoc":"Купе","typeLoc":"Купе","catCode":"Купе","ctypei":4,"ctype":4,"letter":"А","clsType":"2Т","subType":"66К","clsName":"4-местные купе.<p>Вагон повышенной комфортности (рацион питания, санитарно-гигиенический набор*, пресса*, белье).</p><p><strong>Кондиционер, биотуалет в вагоне.</strong></p><p>","services":[{"id":2,"name":"[иконка сайта] Биотуалет","description":"Биотуалет","hasImage":true},{"id":3,"name":"[иконка сайта] Кондиционер","description":"Кондиционер работает в летний период","hasImage":true},{"id":4,"name":"[иконка сайта] Гигиена","description":"Гигиенический набор","hasImage":true},{"id":30,"name":"[иконка сайта] Постель","description":"Постельное белье","hasImage":true},{"id":73,"name":"[иконка сайта] Попутчик","description":"Мультимедийный портал \"Попутчик\"","hasImage":true},{"id":6,"name":"[иконка сайта] Пресса","description":"Пресса","hasImage":true},{"id":14,"name":"[иконка сайта] Провоз животных запрещен","description":"Провоз животных запрещен","hasImage":true}],"tariff":"3966","tariff2":"5090","tariffServ":"766","addSigns":"У1","carrier":"ФПК","carrierId":1,"insuranceFlag":true,"insuranceTypeId":1,"owner":"РЖД/ОКТ","elReg":true,"food":true,"selFood":false,"equippedSIOP":true,"addFood":true,"regularFoodService":false,"noSmok":false,"inetSaleOff":false,"bVip":false,"conferenceRoomFlag":false,"bDeck2":false,"intServiceClass":null,"specialSeatTypes":null,"deferredPayment":false,"varPrice":true,"ferry":false,"seniorTariff":0,"bedding":false,"nonRefundable":false,"addTour":false,"addGoods":true,"addHandLuggage":true,"youth":false,"unior":false,"seats":[{"type":"dn","free":9,"label":"Нижнее","tariff":"3966"},{"type":"up","free":12,"label":"Верхнее","tariff":"3966"}],"places":"005,006,008-016,021,022,024-026,028-032","schemeId":830,"schemeInfo":{"dir":"/dbmm/images/61/28209/14","dirVert":"/dbmm/images/61/28216/14","legend":""},"forcedBedding":true,"policyEnabled":true,"msr":true,"medic":true},{"cnumber":"03","type":"Купе","catLabelLoc":"Купе","typeLoc":"Купе","catCode":"Купе","ctypei":4,"ctype":4,"letter":"А","clsType":"2Э","subType":"66К","clsName":"4-местные купе.<p>Вагон повышенной комфортности (рацион питания, санитарно-гигиенический набор*, пресса*, белье).</p><p><strong>Кондиционер, биотуалет в вагоне.</strong></p><p>Вагон с услугой перевозки животных. </p><p>*кроме двухэтажных вагонов</p>","services":[{"id":2,"name":"[иконка сайта] Биотуалет","description":"Биотуалет","hasImage":true},{"id":3,"name":"[иконка сайта] Кондиционер","description":"Кондиционер работает в летний период","hasImage":true},{"id":73,"name":"[иконка сайта] Попутчик","description":"Мультимедийный портал \"Попутчик\"","hasImage":true},{"id":80,"name":"[иконка сайта] Животные 2Э,2Б,2Ф,2Ц","description":"Для провоза мелких животных необходим выкуп всего купе.<br>Провоз мелких животных бесплатный.<br>Для провоза крупной собаки необходим выкуп всего купе.<br>Можно провести только одну крупную собаку.<br>Провоз крупной собаки бесплатный","hasImage":true},{"id":135,"name":"[иконка сайта] Гигиена 1Э 2Э","description":"Гигиенический набор","hasImage":true},{"id":136,"name":"[иконка сайта] Пресса 1Э 2Э","description":"Пресса","hasImage":true},{"id":30,"name":"[иконка сайта] Постель","description":"Постельное белье","hasImage":true}],"tariff":"3966","tariff2":"5090","tariffServ":"766","addSigns":"МЖ У1","carrier":"ФПК","carrierId":1,"insuranceFlag":true,"insuranceTypeId":1,"owner":"РЖД/ОКТ","elReg":true,"food":true,"selFood":false,"equippedSIOP":true,"addFood":true,"regularFoodService":false,"noSmok":false,"inetSaleOff":false,"bVip":false,"conferenceRoomFlag":false,"bDeck2":false,"intServiceClass":null,"specialSeatTypes":null,"deferredPayment":false,"varPrice":true,"ferry":false,"seniorTariff":0,"bedding":false,"nonRefundable":false,"addTour":false,"addGoods":true,"addHandLuggage":true,"youth":false,"unior":false,"seats":[{"type":"dn","free":5,"label":"Нижнее","tariff":"3966"},{"type":"up","free":13,"label":"Верхнее","tariff":"3966"}],"places":"002Ж,004Ж,006-008М,012-014Ж,018М,022-024Ж,026-028С,030-032М","schemeId":830,"schemeInfo":{"dir":"/dbmm/images/61/28209/14","dirVert":"/dbmm/images/61/28216/14","legend":""},"forcedBedding":true,"policyEnabled":true,"msr":true,"medic":true},{"cnumber":"08","type":"Люкс","catLabelLoc":"СВ","typeLoc":"СВ","catCode":"СВ","ctypei":6,"ctype":6,"letter":"А","clsType":"1Э","subType":"23Л","clsName":"2-местные купе. </p><strong>Биотуалет, кондиционер в вагоне.</strong><p></p> Перевозка домашних животных.</p> Вагон повышенной комфортности (санитарно-гигиенический набор, пресса, белье)","services":[{"id":2,"name":"[иконка сайта] Биотуалет","description":"Биотуалет","hasImage":true},{"id":3,"name":"[иконка сайта] Кондиционер","description":"Кондиционер работает в летний период","hasImage":true},{"id":73,"name":"[иконка сайта] Попутчик","description":"Мультимедийный портал \"Попутчик\"","hasImage":true},{"id":135,"name":"[иконка сайта] Гигиена 1Э 2Э","description":"Гигиенический набор","hasImage":true},{"id":136,"name":"[иконка сайта] Пресса 1Э 2Э","description":"Пресса","hasImage":true},{"id":9,"name":"[иконка сайта] Телевизор","description":"Телевизор","hasImage":true},{"id":18,"name":"[иконка сайта] Животные 1Э, 1У, 1Л, 2Э, 2Б, 1Б (ТКС)","description":"Возможен провоз мелких животных или одной крупной собаки. Для провоза необходим выкуп всего купе.","hasImage":true},{"id":30,"name":"[иконка сайта] Постель","description":"Постельное белье","hasImage":true}],"tariff":"7950","tariff2":null,"tariffServ":"1643","addSigns":"У1","carrier":"ФПК","carrierId":1,"insuranceFlag":true,"insuranceTypeId":1,"owner":"РЖД/ОКТ","elReg":true,"food":true,"selFood":false,"equippedSIOP":true,"addFood":true,"regularFoodService":false,"noSmok":false,"inetSaleOff":false,"bVip":false,"conferenceRoomFlag":false,"bDeck2":false,"intServiceClass":null,"specialSeatTypes":null,"deferredPayment":false,"varPrice":true,"ferry":false,"seniorTariff":0,"bedding":false,"nonRefundable":false,"addTour":false,"addGoods":true,"addHandLuggage":true,"youth":false,"unior":false,"seats":[{"type":"dn","free":6,"label":"Нижнее","tariff":"7950"}],"places":"001,002,012,013,015,016","schemeId":324,"schemeInfo":{"dir":"/dbmm/images/61/28209/44","dirVert":"/dbmm/images/61/28216/44","legend":""},"forcedBedding":true,"policyEnabled":true,"medic":true},{"cnumber":"16","type":"Мягкий","catLabelLoc":"Люкс","typeLoc":"Люкс","catCode":"Люкс","ctypei":5,"ctype":5,"letter":"А","clsType":"1А","subType":"19М","clsName":"<p>1/1-купе с 1-местным размещением, 1/2 -купе с 2-местным размещением. Салон-бар в вагоне. <p><strong>Душ, биотуалет, умывальник, кондиционер в купе.</strong><p>Продается только целое купе.<p>Особые условия провоза детей. Перевозка домашних животных.","services":[{"id":2,"name":"[иконка сайта] Биотуалет","description":"Биотуалет","hasImage":true},{"id":3,"name":"[иконка сайта] Кондиционер","description":"Кондиционер работает в летний период","hasImage":true},{"id":4,"name":"[иконка сайта] Гигиена","description":"Гигиенический набор","hasImage":true},{"id":6,"name":"[иконка сайта] Пресса","description":"Пресса","hasImage":true},{"id":73,"name":"[иконка сайта] Попутчик","description":"Мультимедийный портал \"Попутчик\"","hasImage":true},{"id":9,"name":"[иконка сайта] Телевизор","description":"Телевизор","hasImage":true},{"id":17,"name":"[иконка сайта] Животные 1А, 1И, 1М, 1Е, 1В","description":"Возможен провоз мелких животных. За провоз плата не взимается. Провоз крупных собак не предусмотрен.","hasImage":true},{"id":30,"name":"[иконка сайта] Постель","description":"Постельное белье","hasImage":true}],"tariff":"23587","tariff2":"26740","tariffServ":"3153","addSigns":"У1","carrier":"ФПК","carrierId":1,"insuranceFlag":true,"insuranceTypeId":1,"owner":"РЖД/ОКТ","elReg":true,"food":true,"selFood":false,"equippedSIOP":true,"addFood":true,"regularFoodService":false,"noSmok":false,"inetSaleOff":false,"bVip":true,"conferenceRoomFlag":false,"bDeck2":false,"intServiceClass":null,"specialSeatTypes":null,"deferredPayment":false,"varPrice":true,"ferry":false,"seniorTariff":0,"bedding":false,"nonRefundable":false,"addTour":false,"addGoods":true,"addHandLuggage":true,"youth":false,"unior":false,"seats":[{"type":"kupe","free":1,"label":"Купе","tariff":"23587"}],"places":"007,008","schemeId":320,"forcedBedding":true,"policyEnabled":true,"medic":true}],"addCompLuggage":false,"functionBlocks":[{"className":"s-type-lo","name":"Нижнее место"},{"className":"s-type-mid","name":"Среднее место"},{"className":"s-type-up","name":"Верхнее место"},{"className":"s-type-jumpseat","name":"Откидное место"},{"className":"s-type-seatrot","name":"Место с изменением направления по ходу движения"},{"className":"s-type-seat","name":"Сидячее место"},{"className":"s-prop-undef","name":"Мужское / Женское / Смешанное - признак не определен"},{"className":"s-prop-man","name":"Мужское купе"},{"className":"s-prop-woman","name":"Женское купе"},{"className":"s-prop-mixed","name":"Смешанное купе"},{"className":"s-type-bicycle","name":"Место для пассажира с велосипедом"},{"className":"s-type-pet","name":"Место для проезда с мелким домашним животным"},{"className":"s-type-infant","name":"Место матери и ребенка"},{"className":"s-type-kid","name":"Место пассажира с детьми"},{"className":"s-type-cripple","name":"Место для пассажиров с ограниченными физическими возможностями"},{"className":"s-type-table","name":"Стол"},{"className":"s-type-luggage","name":"Тумба/багаж"},{"className":"s-type-wardrobe","name":"Шкаф/гардероб"},{"className":"s-type-buffet","name":"Буфет"},{"className":"s-type-toilet","name":"Туалет"},{"className":"s-type-shower","name":"Душевая кабина"},{"className":"s-type-conf","name":"Переговорная"},{"className":"s-type-playroom","name":"Детская площадка"},{"className":"s-type-ac","name":"Электрическая розетка"},{"className":"s-type-stairs","name":"Лестничный пролет"},{"className":"s-type-exit","name":"Выход"},{"className":"s-type-cashbox","name":"Касса"},{"className":"s-type-water","name":"Вода"},{"className":"s-prop-kid","name":"Купе для пассажиров с детьми"},{"className":"s-type-kofe-bar","name":"Кафе-бар"},{"className":"s-type-babycarriage","name":"Детские коляски"}],"timestamp":"02.12.2021 18:40:15.064"}],"schemes":[{"id":320,"html":"{\"len\":15,\"cells\":[{\"type\":\"up\",\"number\":2,\"style\":\";border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"wc\",\"style\":\"border-right-color:#000\"},{\"type\":\"wc\",\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":4,\"style\":\"border-right-color:#000\"},{\"type\":\"up\",\"number\":6,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"wc\",\"style\":\"border-right-color:#000\"},{\"type\":\"wc\",\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":8,\"style\":\"border-right-color:#000\"},{\"type\":\"bu\"},{\"type\":\"bu\"},{\"type\":\"bu\"},{\"type\":\"dn\",\"number\":1,\"style\":\";border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"wc\",\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"wc\",\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":3,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":5,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"wc\",\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"wc\",\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":7,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"bu\"},{\"type\":\"bu\"},{\"type\":\"bu\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"bu\"},{\"type\":\"bu\"},{\"type\":\"bu\"}]}","image":null},{"id":324,"html":"{\"len\":24,\"cells\":[{\"type\":\"dn\",\"number\":1,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"st\"},{\"type\":\"dn\",\"number\":2,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":3,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"st\"},{\"type\":\"dn\",\"number\":4,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":5,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"st\"},{\"type\":\"dn\",\"number\":6,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":7,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"st\"},{\"type\":\"dn\",\"number\":8,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":9,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"st\"},{\"type\":\"dn\",\"number\":10,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":11,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"st\"},{\"type\":\"dn\",\"number\":12,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":13,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"st\"},{\"type\":\"dn\",\"number\":14,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":15,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"st\"},{\"type\":\"dn\",\"number\":16,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"}]}","image":null},{"id":830,"html":"{\"len\":24,\"cells\":[{\"type\":\"up\",\"number\":2,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":4,\"style\":\"border-right-color:#000\"},{\"type\":\"up\",\"number\":6,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":8,\"style\":\"border-right-color:#000\"},{\"type\":\"up\",\"number\":10,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":12,\"style\":\"border-right-color:#000\"},{\"type\":\"up\",\"number\":14,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":16,\"style\":\"border-right-color:#000\"},{\"type\":\"up\",\"number\":18,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":20,\"style\":\"border-right-color:#000\"},{\"type\":\"up\",\"number\":22,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":24,\"style\":\"border-right-color:#000\"},{\"type\":\"up\",\"number\":26,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":28,\"style\":\"border-right-color:#000\"},{\"type\":\"up\",\"number\":30,\"style\":\"border-left-color:#000\"},{\"type\":\"st\"},{\"type\":\"up\",\"number\":32,\"style\":\"border-right-color:#000\"},{\"type\":\"dn\",\"number\":1,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":3,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":5,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":7,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":9,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":11,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":13,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":15,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":17,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":19,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":21,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":23,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":25,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":27,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"dn\",\"number\":29,\"style\":\"border-left-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"dn\",\"number\":31,\"style\":\"border-right-color:#000;border-bottom-color:#000\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"XX\"},{\"type\":\"prohod\"},{\"type\":\"XX\"},{\"type\":\"XX\"}]}","image":null}],"insuranceCompany":[{"id":10,"shortName":"ПАО СК «Росгосстрах»","offerUrl":"https://old.rgs.ru/upload/medialibrary/c96/pravila_strakhovaniya_passazhirov_215_ru_eng.pdf","insuranceCost":150,"insuranceBenefit":1500000,"sortOrder":1},{"id":1,"shortName":"АО «СОГАЗ»","offerUrl":"https://direct.sogaz.ru/products/persona/rail-passenger/rules.pdf","insuranceCost":150,"insuranceBenefit":1500000,"sortOrder":2}],"insuranceCompanyTypes":[{"typeId":1,"insuranceTariffs":[{"id":1,"name":"Базовый","insuranceCost":150,"insuranceBenefit":1500000,"default":false,"InsurancePrograms":[{"id":2,"offerUrl":"https://old.rgs.ru/upload/medialibrary/c96/pravila_strakhovaniya_passazhirov_215_ru_eng.pdf","sortOrder":1,"shortName":"ПАО СК «Росгосстрах»"},{"id":1,"offerUrl":"https://direct.sogaz.ru/products/persona/rail-passenger/rules.pdf","sortOrder":2,"shortName":"АО «СОГАЗ»"}]}]}],"psaction":null,"childrenAge":10,"motherAndChildAge":1,"partialPayment":false,"timestamp":"20.03.2022 18:40:15.065"}"#;
        let answer: TrainReply = serde_json::from_str(answer).unwrap();
        let answer = answer.0;

        let data = vec![TrainItem {
            train_number: String::from("001А"),
            leaving_date: parse_train_date!("01.04.2022"),
            leaving_time: parse_train_time!("23:55"),
            arriving_date: parse_train_date!("02.04.2022"),
            arriving_time: parse_train_time!("07:55"),
            leaving_station_name: String::from("САНКТ-ПЕТЕРБУРГ-ГЛАВН. (МОСКОВСКИЙ ВОКЗАЛ)"),
            arriving_station_name: String::from("МОСКВА ОКТЯБРЬСКАЯ (ЛЕНИНГРАДСКИЙ ВОКЗАЛ)"),
            leaving_station_code: RzdStationCode::new(2004001),
            arriving_station_code: RzdStationCode::new(2006004),
            cars: ResultList(vec![
                TrainCar {
                    number: String::from("01"),
                    type_loc: String::from("Купе"),
                    service_class: String::from("2Э"),
                    services: ResultList::new(vec![
                        String::from("Биотуалет"),
                        String::from("Кондиционер работает в летний период"),
                        String::from("Мультимедийный портал \"Попутчик\""),
                        String::from("Для провоза мелких животных необходим выкуп всего купе.<br>Провоз мелких животных бесплатный.<br>Для провоза крупной собаки необходим выкуп всего купе.<br>Можно провести только одну крупную собаку.<br>Провоз крупной собаки бесплатный"),
                        String::from("Гигиенический набор"),
                        String::from("Пресса"),
                        String::from("Постельное белье"),
                    ]),
                    tariff1: String::from("3966"),
                    tariff2: String::from("5090"),
                    tariff_service: String::from("766"),
                    carrier: String::from("ФПК"),
                    insurance: Some(InsuranceInfo {
                        name: String::from("АО «СОГАЗ»"),
                        url: String::from("https://direct.sogaz.ru/products/persona/rail-passenger/rules.pdf"),
                        price: String::from("150"),
                    }),
                    places: String::from("002-004,006-010,012-014,016,020-028,030-032"),
                    seats: ResultList(vec![
                        SeatsInfo {
                            free_seats: 9,
                            seats_type: String::from("Нижнее"),
                            price: String::from("3966"),
                        },
                        SeatsInfo {
                            free_seats: 15,
                            seats_type: String::from("Верхнее"),
                            price: String::from("3966"),
                        },
                    ]),
                },
                TrainCar {
                    number: String::from("02"),
                    type_loc: String::from("Купе"),
                    service_class: String::from("2Т"),
                    services: ResultList::new(vec![
                        String::from("Биотуалет"),
                        String::from("Кондиционер работает в летний период"),
                        String::from("Гигиенический набор"),
                        String::from("Постельное белье"),
                        String::from("Мультимедийный портал \"Попутчик\""),
                        String::from("Пресса"),
                        String::from("Провоз животных запрещен"),
                    ]),
                    tariff1: String::from("3966"),
                    tariff2: String::from("5090"),
                    tariff_service: String::from("766"),
                    carrier: String::from("ФПК"),
                    insurance: Some(InsuranceInfo {
                        name: String::from("АО «СОГАЗ»"),
                        url: String::from("https://direct.sogaz.ru/products/persona/rail-passenger/rules.pdf"),
                        price: String::from("150"),
                    }),
                    places: String::from("005,006,008-016,021,022,024-026,028-032"),
                    seats: ResultList(vec![
                        SeatsInfo {
                            free_seats: 9,
                            seats_type: String::from("Нижнее"),
                            price: String::from("3966"),
                        },
                        SeatsInfo {
                            free_seats: 12,
                            seats_type: String::from("Верхнее"),
                            price: String::from("3966"),
                        },
                    ]),
                },
                TrainCar {
                    number: String::from("03"),
                    type_loc: String::from("Купе"),
                    service_class: String::from("2Э"),
                    services: ResultList::new(vec![
                        String::from("Биотуалет"),
                        String::from("Кондиционер работает в летний период"),
                        String::from("Мультимедийный портал \"Попутчик\""),
                        String::from("Для провоза мелких животных необходим выкуп всего купе.<br>Провоз мелких животных бесплатный.<br>Для провоза крупной собаки необходим выкуп всего купе.<br>Можно провести только одну крупную собаку.<br>Провоз крупной собаки бесплатный"),
                        String::from("Гигиенический набор"),
                        String::from("Пресса"),
                        String::from("Постельное белье"),
                    ]),
                    tariff1: String::from("3966"),
                    tariff2: String::from("5090"),
                    tariff_service: String::from("766"),
                    carrier: String::from("ФПК"),
                    insurance: Some(InsuranceInfo {
                        name: String::from("АО «СОГАЗ»"),
                        url: String::from("https://direct.sogaz.ru/products/persona/rail-passenger/rules.pdf"),
                        price: String::from("150"),
                    }),
                    places: String::from("002Ж,004Ж,006-008М,012-014Ж,018М,022-024Ж,026-028С,030-032М"),
                    seats: ResultList(vec![
                        SeatsInfo {
                            free_seats: 5,
                            seats_type: String::from("Нижнее"),
                            price: String::from("3966"),
                        },
                        SeatsInfo {
                            free_seats: 13,
                            seats_type: String::from("Верхнее"),
                            price: String::from("3966"),
                        },
                    ]),
                },
                TrainCar {
                    number: String::from("08"),
                    type_loc: String::from("СВ"),
                    service_class: String::from("1Э"),
                    services: ResultList::new(vec![
                        String::from("Биотуалет"),
                        String::from("Кондиционер работает в летний период"),
                        String::from("Мультимедийный портал \"Попутчик\""),
                        String::from("Гигиенический набор"),
                        String::from("Пресса"),
                        String::from("Телевизор"),
                        String::from("Возможен провоз мелких животных или одной крупной собаки. Для провоза необходим выкуп всего купе."),
                        String::from("Постельное белье"),
                    ]),
                    tariff1: String::from("7950"),
                    tariff2: String::new(),
                    tariff_service: String::from("1643"),
                    carrier: String::from("ФПК"),
                    insurance: Some(InsuranceInfo {
                        name: String::from("АО «СОГАЗ»"),
                        url: String::from("https://direct.sogaz.ru/products/persona/rail-passenger/rules.pdf"),
                        price: String::from("150"),
                    }),
                    places: String::from("001,002,012,013,015,016"),
                    seats: ResultList(vec![
                        SeatsInfo {
                            free_seats: 6,
                            seats_type: String::from("Нижнее"),
                            price: String::from("7950"),
                        },
                    ]),
                },
                TrainCar {
                    number: String::from("16"),
                    type_loc: String::from("Люкс"),
                    service_class: String::from("1А"),
                    services: ResultList::new(vec![
                        String::from("Биотуалет"),
                        String::from("Кондиционер работает в летний период"),
                        String::from("Гигиенический набор"),
                        String::from("Пресса"),
                        String::from("Мультимедийный портал \"Попутчик\""),
                        String::from("Телевизор"),
                        String::from("Возможен провоз мелких животных. За провоз плата не взимается. Провоз крупных собак не предусмотрен."),
                        String::from("Постельное белье"),
                    ]),
                    tariff1: String::from("23587"),
                    tariff2: String::from("26740"),
                    tariff_service: String::from("3153"),
                    carrier: String::from("ФПК"),
                    insurance: Some(InsuranceInfo {
                        name: String::from("АО «СОГАЗ»"),
                        url: String::from("https://direct.sogaz.ru/products/persona/rail-passenger/rules.pdf"),
                        price: String::from("150"),
                    }),
                    places: String::from("007,008"),
                    seats: ResultList(vec![
                        SeatsInfo {
                            free_seats: 1,
                            seats_type: String::from("Купе"),
                            price: String::from("23587"),
                        },
                    ]),
                },
            ]),
        }];

        assert!(answer.success);
        assert_eq!(answer.value, data);
    }
}
