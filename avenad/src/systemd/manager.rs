use zbus::proxy;

// use super::service_unit::ServiceUnitProxy;
use super::unit::Systemd1UnitProxy;

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1",
    gen_blocking = false
)]
pub trait Systemd1Manager {
    // Properties
    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn features(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn progress(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn architecture(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn environment(&self) -> zbus::Result<Vec<String>>;

    #[zbus(property)]
    fn log_level(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn log_target(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn unit_path(&self) -> zbus::Result<Vec<String>>;

    #[zbus(property)]
    fn default_standard_output(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn default_standard_error(&self) -> zbus::Result<String>;

    // Methods
    fn reload(&self) -> zbus::Result<()>;

    #[zbus(object = "Systemd1Unit")]
    fn get_unit(&self, name: &str);

    fn get_unit_file_state(&self, name: &str) -> zbus::Result<String>;

    fn list_units(&self) -> zbus::Result<Vec<UnitListing>>;

    fn start_unit(&self, name: &str, mode: &str) -> zbus::Result<zvariant::OwnedObjectPath>;
    fn stop_unit(&self, name: &str, mode: &str) -> zbus::Result<zvariant::OwnedObjectPath>;
    fn restart_unit(&self, name: &str, mode: &str) -> zbus::Result<zvariant::OwnedObjectPath>;

    #[zbus(name = "ListUnitsByPatterns")]
    fn list_units_by_patterns(
        &self,
        states: Vec<&str>,
        patterns: Vec<&str>,
    ) -> zbus::Result<Vec<UnitListing>>;

    #[zbus(name = "ListUnitsByNames")]
    fn list_units_by_names(&self, names: Vec<&str>) -> zbus::Result<Vec<UnitListing>>;
}

#[derive(Debug, Clone, serde::Deserialize, zvariant::Type)]
pub struct UnitListing {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub followed_by: String,
    pub object_path: zvariant::OwnedObjectPath,
    pub job_id: u32,
    pub job_type: String,
    pub job_object_path: zvariant::OwnedObjectPath,
}
