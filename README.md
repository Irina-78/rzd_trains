# rzd_trains

Библиотека для получения информации о пассажирских поездах с сайта rzd.ru:

- расписания движения поездов;

- информации о свободных местах в поезде;

- маршрута следования выбранного поезда;

- кодов станций РЖД.


## Использование

Добавьте в зависимости `Cargo.toml` библиотеку:

```toml
[dependencies]
rzd_trains = "0.1"
```

## Пример

Получение информации о расписании движения поездов:

```rust,no_run
use rzd_trains::RzdClient;
use rzd_trains::{RouteList, RzdStationCode, TrainDate, TrainScheduleSearch, TrainType};

fn main() {
    let from = RzdStationCode::new(2000000);
    let to = RzdStationCode::new(2004000);
    let leaving_date = TrainDate::new(2022, 4, 1);
    let train_type = TrainType::AllTrains;
    let check_seats = true;

    let q = TrainScheduleSearch::new(from, to, leaving_date, train_type, check_seats);

    let result = RzdClient::<RouteList>::get(&q).unwrap();

    match result {
        Some(list) => println!("{}", list),
        None => println!("Nothing found"),
    }
}
```

## Получение кода станции

Сервер "РЖД" оперирует кодами станций. Зная часть имени станции, можно найти ее код следующим образом:

```rust,no_run
use rzd_trains::RzdClient;
use rzd_trains::{StationCodeSearch, StationList};

fn main() {
    let q = StationCodeSearch::new("москва").unwrap();

    let result = RzdClient::<StationList>::get(&q).unwrap();

    match result {
        Some(list) => println!("{}", list),
        None => println!("Nothing found"),
    }
}
```

## License

The library is dual licensed under the Apache 2.0 license and the MIT license.
