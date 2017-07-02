use bytes::BytesMut;
use bytes::{Buf, IntoBuf};
use bincode::{deserialize};
use phf;

#[derive(Debug)]
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

const STANDARD_MSG :u8 = 0x50;
const EXTENDED_MSG :u8 = 0x51;
const X10_RECEIVED :u8 = 0x52;
const ALL_LINKING_COMPLETED :u8 = 0x53;
const BUTTON_EVENT_REPORT :u8 = 0x54;
const USER_RESET_DETECTED :u8 = 0x55;
const ALL_LINK_CLEANUP_FAILURE_REPORT :u8 = 0x56;
const ALL_LINK_RECORD_RESPONSE :u8 = 0x57;
const ALL_LINK_CLEANUP_STATUS_REPORT :u8 = 0x58;
const SEND_STANDARD_MSG :u8 = 0x62;


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