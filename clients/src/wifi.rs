use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    nvs::EspDefaultNvsPartition,
    wifi::{ClientConfiguration, Configuration, EspWifi},
};
use heapless::String as enString;

pub fn setup_wifi<'a>() -> Result<EspWifi<'a>, std::io::Error> {
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    let ssid: enString<32> = std::env!("WIFI_NAME").parse().unwrap();
    let password: enString<64> = std::env!("WIFI_PASS").parse().unwrap();

    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid,
            password,
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        log::info!("Waiting for station {:?}", config);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    log::info!("Connected to wifi!");

    // have to wait for network to configure itself I think
    std::thread::sleep(std::time::Duration::from_secs(5));

    Ok(wifi_driver)
}
