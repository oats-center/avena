use zbus::blocking::Connection;

use super::service_unit::{ServiceUnitProxy, ServiceUnitProxyBlocking};

#[zbus::dbus_proxy(
    default_service = "org.freedesktop.systemd1",
    interface = "org.freedesktop.systemd1.Manager",
    default_path = "/org/freedesktop/systemd1"
)]
trait Manager {
    // Properties
    #[dbus_proxy(property)]
    fn version(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn features(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn progress(&self) -> zbus::Result<f64>;

    #[dbus_proxy(property)]
    fn architecture(&self) -> zbus::Result<String>;

    // Methods
    #[dbus_proxy(object = "ServiceUnit")]
    fn get_unit(&self, name: &str);

    fn list_units(&self) -> zbus::Result<Vec<UnitListing>>;

    fn start_unit(&self, name: &str, mode: &str) -> zbus::Result<zvariant::OwnedObjectPath>;
    fn stop_unit(&self, name: &str, mode: &str) -> zbus::Result<zvariant::OwnedObjectPath>;
    fn restart_unit(&self, name: &str, mode: &str) -> zbus::Result<zvariant::OwnedObjectPath>;

    #[dbus_proxy(name = "ListUnitsByPatterns")]
    fn list_units_by_patterns(
        &self,
        states: Vec<&str>,
        patterns: Vec<&str>,
    ) -> zbus::Result<Vec<UnitListing>>;
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

pub fn get_manager<'a>() -> zbus::Result<ManagerProxyBlocking<'a>> {
    let conn = Connection::system()?;
    ManagerProxyBlocking::new(&conn)
}
