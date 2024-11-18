use std::collections::HashMap;

pub fn get_board_chip_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

    // Nordic boards
    map.insert("nrfmicro", "nrf52840");
    map.insert("bluemicro840", "nrf52840");
    map.insert("puchi_ble", "nrf52840");
    map.insert("nice!nano", "nrf52840");
    map.insert("nice!nano_v2", "nrf52840");
    map.insert("XIAO BLE", "nrf52840");


    map
}