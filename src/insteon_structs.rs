use bytes::BytesMut;
use bytes::{Buf, IntoBuf};
use bincode::{deserialize};
use phf;

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Serialize, Deserialize, PartialEq)]
pub enum InsteonMsg {

    StandardMsg {
        addr_from: [u8; 3],
        addr_to: [u8; 3],
        msg_flags: u8,
        cmd1: u8,
        cmd2: u8,
    },

    ExtendedMsg {
        addr_from: [u8; 3],
        addr_to: [u8; 3],
        msg_flags: u8,
        cmd1: u8,
        cmd2: u8,
        user_data: [u8; 14],
    },

    X10Received {
        raw_x10: u8,
        x10_flag: u8,
    },

    AllLinkingCompleted {
        link_code: u8,
        all_link_group: u8,
        id: [u8; 3],
        device_category: u8,
        device_subcategory: u8,
        firmware_version: u8,
    },

    ButtonEventReport {
        button_event: u8,
    },

    UserResetDetected {},

    AllLinkCleanupFailureReport {
        x01: u8,
        all_link_group: u8,
        id: [u8; 3],
    },

    AllLinkRecordResponse {
        all_link_record_flags: u8,
        all_link_group: u8,
        id: [u8; 3],
        link_data: [u8; 3],
    },

    AllLinkCleanupStatusReport {
        status_byte: u8,
    },

    SendStandardMsg {
        addr_to: [u8; 3],
        msg_flags: u8,
        cmd1: u8,
        cmd2: u8,
    }

}

#[derive(Copy, Clone)]
pub enum Command {
    DataReq = 0x03,
    InsteonVer = 0x0D,
    IdReq = 0x10,
    On = 0x11,
    FastOn = 0x12,
    Off = 0x13,
    FastOff = 0x14,
    BrightStep = 0x15,
    DimStep = 0x16,
    StartChange = 0x17,
    StopChange = 0x18,
    StatusReq = 0x19,
    GetOpFlags = 0x1F,
    SetOpFlags = 0x20,
    SetHiAddr = 0x28,
    PokeEE = 0x29,
    PeekEE = 0x2B,
    OnAtRate = 0x2E,
    OffAtRate = 0x2F,
    WriteOutput = 0x48,
    ReadInput = 0x49,
    GetSensorVal = 0x4A,
    ReadCfg = 0x4E,
    IoModuleCtrl = 0x4F,
    ThermalZoneInfo = 0x6A,
    SetImCfg = 0x6B
}

pub fn u8Command(cmd: Command) -> u8 {
    cmd as u8
}

pub const DISCRIMINANT_SIZE : usize = 4;

pub const MSG_BEGIN :u8 = 0x02;

pub const STANDARD_MSG :u8 = 0x50;
pub const EXTENDED_MSG :u8 = 0x51;
pub const X10_RECEIVED :u8 = 0x52;
pub const ALL_LINKING_COMPLETED :u8 = 0x53;
pub const BUTTON_EVENT_REPORT :u8 = 0x54;
pub const USER_RESET_DETECTED :u8 = 0x55;
pub const ALL_LINK_CLEANUP_FAILURE_REPORT :u8 = 0x56;
pub const ALL_LINK_RECORD_RESPONSE :u8 = 0x57;
pub const ALL_LINK_CLEANUP_STATUS_REPORT :u8 = 0x58;
pub const SEND_STANDARD_MSG :u8 = 0x62;


static SIZE_MAP: phf::Map<u8, usize> = phf_map!(
    0x50u8 => 9,
    0x51u8 => 23,
    0x52u8 => 2,
    0x53u8 => 8,
    0x54u8 => 1,
    0x55u8 => 0,
    0x56u8 => 1,
    0x57u8 => 8,
    0x58u8 => 1,
    0x62u8 => 6,
);

static DISCRIMINANT_MAP: phf::Map<u8, u8> = phf_map!(
    0x50u8 => 0,
    0x51u8 => 1,
    0x52u8 => 2,
    0x53u8 => 3,
    0x54u8 => 4,
    0x55u8 => 5,
    0x56u8 => 6,
    0x57u8 => 7,
    0x58u8 => 8,
    0x62u8 => 9,
);

pub fn get_msg_size(msg_type: &u8) -> Option<usize> {
    SIZE_MAP.get(msg_type).cloned()
}

pub fn get_discriminant(msg_type: &u8) -> Option<u8> {
    DISCRIMINANT_MAP.get(msg_type).cloned()
}

impl InsteonMsg {
    pub fn new(buf: &BytesMut) -> Option<(InsteonMsg, usize)> {

        if buf.is_empty() {
            return None
        };

        let mut current = buf.into_buf().iter();
        let command_type = current.next().unwrap();
        let buf_size = buf.len() - 1;

        pub fn decode(discriminant: u8, v: Vec<u8>) -> InsteonMsg {
            let mut encoded = vec!(discriminant, 0, 0, 0);
            encoded.extend(v);
            deserialize(&encoded).unwrap()
        }

        match get_msg_size(&command_type) {
            Some(size) if buf_size < size  => {
                None
            },

            Some(size) => {
                let v : Vec<u8> = current.take(size).collect();
                let discriminant = get_discriminant(&command_type).unwrap();
                Some((decode(discriminant, v), size))
            },

            None => {
                None
            },
        }
    }
}