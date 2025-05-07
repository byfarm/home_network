use load_dotenv;

load_dotenv::load_dotenv!("../.env");

fn main() {
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let _wifi = clients::wifi::setup_wifi();

    clients::communicate::run_server().unwrap();
}

