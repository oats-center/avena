#[zbus::dbus_proxy(
    interface = "org.freedesktop.systemd1.Service",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
pub trait ServiceUnit {
    #[dbus_proxy(property, name = "Type")]
    fn service_type(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn status_text(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn memory_current(&self) -> zbus::Result<u64>;

    #[dbus_proxy(property)]
    fn memory_available(&self) -> zbus::Result<u64>;

    #[dbus_proxy(property)]
    fn memory_accounting(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property, name = "CPUUsageNSec")]
    fn cpu_usage_n_sec(&self) -> zbus::Result<u64>;

    #[dbus_proxy(property, name = "CPUShares")]
    fn cpu_shares(&self) -> zbus::Result<u64>;

    #[dbus_proxy(property, name = "CPUAccounting")]
    fn cpu_accounting(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property)]
    fn exec_main_start_timestamp(&self) -> zbus::Result<u64>;

    #[dbus_proxy(property)]
    fn exec_main_exit_timestamp(&self) -> zbus::Result<u64>;

    #[dbus_proxy(property, name = "ExecMainPID")]
    fn exec_main_pid(&self) -> zbus::Result<u32>;
}
