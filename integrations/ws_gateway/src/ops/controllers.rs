use crate::model::{Target, Transport};
use motor_core::bus::CanBus;
#[cfg(target_os = "windows")]
use motor_core::pcan::PcanBus;
#[cfg(target_os = "linux")]
use motor_core::socketcan::SocketCanBus;
use motor_vendor_damiao::DamiaoController;
use motor_vendor_hexfellow::HexfellowController;
use motor_vendor_myactuator::MyActuatorController;
use motor_vendor_robstride::RobstrideController;

pub(crate) fn myactuator_feedback_default(motor_id: u16) -> u16 {
    0x240u16.saturating_add(motor_id)
}

pub(crate) fn open_damiao_controller(
    base: &Target,
    transport: Transport,
) -> Result<DamiaoController, String> {
    match transport {
        Transport::Auto | Transport::SocketCan => {
            DamiaoController::new_socketcan(&base.channel).map_err(|e| e.to_string())
        }
        Transport::SocketCanFd => {
            DamiaoController::new_socketcanfd(&base.channel).map_err(|e| e.to_string())
        }
        Transport::DmSerial => {
            DamiaoController::new_dm_serial(&base.serial_port, base.serial_baud)
                .map_err(|e| e.to_string())
        }
    }
}

pub(crate) fn open_robstride_controller(
    base: &Target,
    transport: Transport,
) -> Result<RobstrideController, String> {
    match transport {
        Transport::Auto | Transport::SocketCan => {
            RobstrideController::new_socketcan(&base.channel).map_err(|e| e.to_string())
        }
        Transport::SocketCanFd => {
            RobstrideController::new_socketcanfd(&base.channel).map_err(|e| e.to_string())
        }
        Transport::DmSerial => Err("transport dm-serial is damiao-only".to_string()),
    }
}

pub(crate) fn open_myactuator_controller(
    base: &Target,
    transport: Transport,
) -> Result<MyActuatorController, String> {
    match transport {
        Transport::Auto | Transport::SocketCan => {
            MyActuatorController::new_socketcan(&base.channel).map_err(|e| e.to_string())
        }
        Transport::SocketCanFd => {
            MyActuatorController::new_socketcanfd(&base.channel).map_err(|e| e.to_string())
        }
        Transport::DmSerial => Err("transport dm-serial is damiao-only".to_string()),
    }
}

pub(crate) fn open_hexfellow_controller(
    base: &Target,
    transport: Transport,
) -> Result<HexfellowController, String> {
    match transport {
        Transport::Auto | Transport::SocketCanFd => {
            HexfellowController::new_socketcanfd(&base.channel).map_err(|e| e.to_string())
        }
        Transport::SocketCan => Err("hexfellow requires transport socketcanfd (or auto)".to_string()),
        Transport::DmSerial => Err("transport dm-serial is damiao-only".to_string()),
    }
}

pub(crate) fn open_hightorque_bus(
    _base: &Target,
    transport: Transport,
) -> Result<Box<dyn CanBus>, String> {
    match transport {
        Transport::Auto | Transport::SocketCan => {
            #[cfg(target_os = "linux")]
            {
                return Ok(Box::new(
                    SocketCanBus::open(&_base.channel).map_err(|e| e.to_string())?,
                ));
            }
            #[cfg(target_os = "windows")]
            {
                return Ok(Box::new(
                    PcanBus::open(&_base.channel).map_err(|e| e.to_string())?,
                ));
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                Err("no CAN backend for current platform".to_string())
            }
        }
        Transport::SocketCanFd => {
            Err("hightorque currently uses standard CAN transport only".to_string())
        }
        Transport::DmSerial => Err("transport dm-serial is damiao-only".to_string()),
    }
}
