use tty::*;

#[derive(Debug)]
pub enum InsteonMsg {

    StandardMsg {
        addr_from: [u8; 3],
        addr_to: [u8; 3],
        msg_flags: u8,
        cmd1: u8,
        cmd2: u8,
    },

    SendStandardMsg {
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

impl InsteonMsg {
    pub fn new(port: &mut Port) -> InsteonMsg {
        let command_type :u8 = port.take(1).next().unwrap();

        match command_type {
            STANDARD_MSG => {
                let v : Vec<u8> = port.take(9).collect();

                InsteonMsg::StandardMsg {
                    addr_from : [v[0], v[1], v[2]],
                    addr_to : [v[3], v[4], v[5]],
                    msg_flags : v[6],
                    cmd1 : v[7],
                    cmd2 : v[8],
                }
            },

            EXTENDED_MSG => {
                let v : Vec<u8> = port.take(23).collect();

                InsteonMsg::ExtendedMsg {
                    addr_from : [v[0], v[1], v[2]],
                    addr_to : [v[3], v[4], v[5]],
                    msg_flags : v[6],
                    cmd1 : v[7],
                    cmd2 : v[8],
                    user_data : [v[9], v[10], v[11], v[12], v[13], v[14], v[15], v[16],
                                 v[17], v[18], v[19], v[20], v[21], v[22]]

                }
            },

            X10_RECEIVED => {
                let v : Vec<u8> = port.take(9).collect();

                InsteonMsg::X10Received{
                    raw_x10 : v[0],
                    x10_flag : v[1],
                }
            },

            ALL_LINKING_COMPLETED => {
                let v : Vec<u8> = port.take(8).collect();

                InsteonMsg::AllLinkingCompleted{
                    link_code: v[0],
                    all_link_group: v[1],
                    id: [v[2], v[3], v[4]],
                    device_category: v[5],
                    device_subcategory: v[6],
                    firmware_version: v[7],
                }
            },


            BUTTON_EVENT_REPORT => {
                let v : Vec<u8> = port.take(1).collect();

                InsteonMsg::ButtonEventReport {
                    button_event : v[0],
                }
            },

            USER_RESET_DETECTED => InsteonMsg::UserResetDetected {},

            ALL_LINK_CLEANUP_FAILURE_REPORT => {
                let v : Vec<u8> = port.take(1).collect();

                InsteonMsg::AllLinkCleanupFailureReport {
                    x01: v[0],
                    all_link_group: v[1],
                    id: [v[2], v[3], v[4]],
                }
            },

            ALL_LINK_RECORD_RESPONSE => {
                let v: Vec<u8> = port.take(8).collect();

                InsteonMsg::AllLinkRecordResponse {
                    all_link_record_flags: v[0],
                    all_link_group: v[1],
                    id: [v[2], v[3], v[4]],
                    link_data: [v[5], v[6], v[7]],
                }
            },

            ALL_LINK_CLEANUP_STATUS_REPORT => {
                let v : Vec<u8> = port.take(1).collect();

                InsteonMsg::AllLinkCleanupStatusReport {
                     status_byte : v[0],
                }
            },

            _ => panic!("unknown command type: {:?}", command_type),
        }
    }
}