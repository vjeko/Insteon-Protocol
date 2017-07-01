use bytes::BytesMut;
use bytes::{Buf, IntoBuf};

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

impl InsteonMsg {
    pub fn new(buf: &BytesMut) -> Option<(InsteonMsg, usize)> {

        if buf.is_empty() {
            return None
        };

        let mut current = buf.into_buf().iter();
        let command_type = current.next().unwrap();
        let buf_size = buf.len() - 1;

        //debug!("Received command: {}", command_type);
        match command_type {
            STANDARD_MSG => {
                const MSG_SIZE : usize = 9;
                let v : Vec<u8> = current.take(MSG_SIZE).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::StandardMsg {
                    addr_from : [v[0], v[1], v[2]],
                    addr_to : [v[3], v[4], v[5]],
                    msg_flags : v[6],
                    cmd1 : v[7],
                    cmd2 : v[8],
                }, MSG_SIZE))
            },

            EXTENDED_MSG => {
                const MSG_SIZE : usize = 23;
                let v : Vec<u8> = current.take(MSG_SIZE).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::ExtendedMsg {
                    addr_from : [v[0], v[1], v[2]],
                    addr_to : [v[3], v[4], v[5]],
                    msg_flags : v[6],
                    cmd1 : v[7],
                    cmd2 : v[8],
                    user_data : [v[9], v[10], v[11], v[12], v[13], v[14], v[15], v[16],
                        v[17], v[18], v[19], v[20], v[21], v[22]]

                }, MSG_SIZE))
            },

            X10_RECEIVED => {
                const MSG_SIZE : usize = 2;
                let v : Vec<u8> = current.take(MSG_SIZE).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::X10Received{
                    raw_x10 : v[0],
                    x10_flag : v[1],
                }, MSG_SIZE))
            },

            ALL_LINKING_COMPLETED => {
                const MSG_SIZE : usize = 8;
                let v : Vec<u8> = current.take(MSG_SIZE).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::AllLinkingCompleted{
                    link_code: v[0],
                    all_link_group: v[1],
                    id: [v[2], v[3], v[4]],
                    device_category: v[5],
                    device_subcategory: v[6],
                    firmware_version: v[7],
                }, MSG_SIZE))
            },


            BUTTON_EVENT_REPORT => {
                const MSG_SIZE : usize = 1;
                let v : Vec<u8> = current.take(MSG_SIZE).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::ButtonEventReport {
                    button_event : v[0],
                }, MSG_SIZE))
            },

            USER_RESET_DETECTED => {
                const MSG_SIZE : usize = 0;
                Some((InsteonMsg::UserResetDetected {}, MSG_SIZE))
            },

            ALL_LINK_CLEANUP_FAILURE_REPORT => {
                const MSG_SIZE : usize = 1;
                let v : Vec<u8> = current.take(MSG_SIZE).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::AllLinkCleanupFailureReport {
                    x01: v[0],
                    all_link_group: v[1],
                    id: [v[2], v[3], v[4]],
                }, MSG_SIZE))
            },

            ALL_LINK_RECORD_RESPONSE => {
                const MSG_SIZE : usize = 8;
                let v: Vec<u8> = current.take(MSG_SIZE).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::AllLinkRecordResponse {
                    all_link_record_flags: v[0],
                    all_link_group: v[1],
                    id: [v[2], v[3], v[4]],
                    link_data: [v[5], v[6], v[7]],
                }, MSG_SIZE))
            },

            ALL_LINK_CLEANUP_STATUS_REPORT => {
                const MSG_SIZE : usize = 1;
                let v : Vec<u8> = current.take(MSG_SIZE).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::AllLinkCleanupStatusReport {
                    status_byte : v[0],
                }, MSG_SIZE))
            },

            SEND_STANDARD_MSG => {
                const MSG_SIZE : usize = 6;
                let v : Vec<u8> = current.take(6).collect();
                if buf_size < MSG_SIZE {
                    return None
                }

                Some((InsteonMsg::SendStandardMsg {
                    addr_to : [v[0], v[1], v[2]],
                    msg_flags : v[3],
                    cmd1: v[4],
                    cmd2: v[5]
                }, MSG_SIZE))
            },

            _ => panic!("unknown command type: {:?}", command_type),
        }
    }
}