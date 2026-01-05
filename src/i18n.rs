#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    En,
    Zh,
}

pub struct Dict {
    pub device_status_title: &'static str,
    pub device_status_subtitle: &'static str,
    pub device_disconnected: &'static str,
    pub settings: &'static str,
    pub connect: &'static str,

    pub quick_actions_title: &'static str,
    pub flash_firmware: &'static str,
    pub monitor: &'static str,
    pub files: &'static str,

    pub home_nav: &'static str,
    pub devices_nav: &'static str,
    pub settings_nav: &'static str,

    // Device Info UI
    pub ready_to_flash: &'static str,
    pub probing_error: &'static str,
    pub connection_info: &'static str,
    pub hardware_details: &'static str,
    pub port: &'static str,
    pub vid_pid: &'static str,
    pub serial_number: &'static str,
    pub chip_model: &'static str,
    pub flash_size: &'static str,
    pub mac_address: &'static str,
    pub chip_revision: &'static str,
    pub crystal_frequency: &'static str,
    pub features: &'static str,

    pub connection_type: &'static str,
    pub type_native_usb: &'static str,
    pub type_uart_bridge: &'static str,

    pub driver_check_btn: &'static str,
    pub driver_installed: &'static str,
    pub driver_not_found: &'static str,

    // Devices Page
    pub devices_title_flashing: &'static str,
    pub devices_subtitle_flashing: &'static str,
    pub devices_label_firmware_file: &'static str,
    pub devices_placeholder_firmware_file: &'static str,
    pub devices_btn_browse: &'static str,
    pub devices_label_flash_address: &'static str,
    pub devices_flashing_status: &'static str,
    pub devices_btn_start_flash: &'static str,

    pub devices_title_monitor: &'static str,
    pub devices_subtitle_monitor: &'static str,
    pub devices_label_baud_rate: &'static str,
    pub devices_log_placeholder: &'static str,
    pub devices_input_placeholder: &'static str,
    pub devices_btn_send: &'static str,
    pub devices_btn_disconnect: &'static str,
    pub devices_btn_clear: &'static str,
    pub monitor_tab: &'static str,
    pub board_view_tab: &'static str,
    pub board_view_title: &'static str,
}

pub const EN_DICT: Dict = Dict {
    device_status_title: "Device Status",
    device_status_subtitle: "Current Connection",
    device_disconnected: "Disconnected",
    settings: "Settings",
    connect: "Connect",

    quick_actions_title: "Quick Actions",
    flash_firmware: "Flash Firmware",
    monitor: "Monitor",
    files: "Files",

    home_nav: "Home",
    devices_nav: "Devices",
    settings_nav: "Settings",

    ready_to_flash: "Ready to flash",
    probing_error: "Probing Error",
    connection_info: "Connection Info",
    hardware_details: "Hardware Details",
    port: "Port",
    vid_pid: "VID:PID",
    serial_number: "Serial Number",
    chip_model: "Model",
    flash_size: "Flash Size",
    mac_address: "MAC Address",
    chip_revision: "Revision",
    crystal_frequency: "Crystal Frequency",
    features: "Features",

    connection_type: "Type",
    type_native_usb: "Native USB",
    type_uart_bridge: "UART Bridge",

    driver_check_btn: "Check Driver",
    driver_installed: "Driver Installed",
    driver_not_found: "Driver Not Found",

    devices_title_flashing: "Firmware Flashing",
    devices_subtitle_flashing: "Flash .bin files to ESP32",
    devices_label_firmware_file: "Firmware File",
    devices_placeholder_firmware_file: "/path/to/firmware.bin",
    devices_btn_browse: "Browse",
    devices_label_flash_address: "Flash Address (Hex)",
    devices_flashing_status: "Flashing...",
    devices_btn_start_flash: "Start Flash",

    devices_title_monitor: "Serial Monitor",
    devices_subtitle_monitor: "Real-time logs",
    devices_label_baud_rate: "Baud Rate",
    devices_log_placeholder: "No logs yet...",
    devices_input_placeholder: "Send command...",
    devices_btn_send: "Send",
    devices_btn_disconnect: "Disconnect",
    devices_btn_clear: "Clear Logs",
    monitor_tab: "Monitor",
    board_view_tab: "Board View",
    board_view_title: "Board View",
};

pub const ZH_DICT: Dict = Dict {
    device_status_title: "设备状态",
    device_status_subtitle: "当前连接",
    device_disconnected: "未连接",
    settings: "设置",
    connect: "连接",

    quick_actions_title: "快捷操作",
    flash_firmware: "烧录固件",
    monitor: "串口监视",
    files: "文件管理",

    home_nav: "主页",
    devices_nav: "设备",
    settings_nav: "设置",

    ready_to_flash: "就绪",
    probing_error: "读取失败",
    connection_info: "连接信息",
    hardware_details: "硬件详情",
    port: "端口",
    vid_pid: "VID:PID",
    serial_number: "序列号",
    chip_model: "芯片型号",
    flash_size: "Flash 容量",
    mac_address: "MAC 地址",
    chip_revision: "芯片版本",
    crystal_frequency: "晶振频率",
    features: "功能特性",

    connection_type: "连接类型",
    type_native_usb: "原生 USB",
    type_uart_bridge: "UART 桥接",

    driver_check_btn: "检查驱动",
    driver_installed: "驱动已安装",
    driver_not_found: "未检测到 CH34X 驱动",

    devices_title_flashing: "固件烧录",
    devices_subtitle_flashing: "烧录 .bin 文件到 ESP32",
    devices_label_firmware_file: "固件文件",
    devices_placeholder_firmware_file: "/path/to/firmware.bin",
    devices_btn_browse: "浏览",
    devices_label_flash_address: "烧录地址 (Hex)",
    devices_flashing_status: "正在烧录...",
    devices_btn_start_flash: "开始烧录",

    devices_title_monitor: "串口监视器",
    devices_subtitle_monitor: "实时日志监控",
    devices_label_baud_rate: "波特率",
    devices_log_placeholder: "暂无日志...",
    devices_input_placeholder: "发送指令...",
    devices_btn_send: "发送",
    devices_btn_disconnect: "断开连接",
    devices_btn_clear: "清空日志",
    monitor_tab: "串口监视",
    board_view_tab: "开发板视图",
    board_view_title: "开发板视图",
};

pub fn get_dict(lang: Language) -> Dict {
    match lang {
        Language::En => EN_DICT,
        Language::Zh => ZH_DICT,
    }
}
