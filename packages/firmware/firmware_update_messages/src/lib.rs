#![no_std]

use serde::{Deserialize, Serialize};

/// Messages from the host (SoC or computer) to the firmware currently running on the device.
/// Each time a response to a request is received, the TCP connection will be closed. A new
/// connection will need to be established for the next request (how TCP was meant to be used)
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Request {
    /// Requests hash of current firmware on the device.
    Hash,

    /// Host is going to upload firmware to the device. The payload of firmware updates firmware
    /// pretty much guarenteed to always be too large to fit in the RX buffer, so update data will
    /// instead be sent over the raw stream. Closing the stream indicates the end of the payload.
    ///
    /// Note that successfully uploading does not mean the Bootloader will switch to the new firmware.
    Upload,

    /// Switch to the new firmware that was uploaded with the last request.
    Switch {
        /// Expected hash signature of the uploaded firmware. Used to verify the image was
        /// correctly uploaded. If the uploaded firmware does not match the hash, the switch
        /// will fail.
        expected_hash: u64,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// Response to Request::Hash.
    Hash {
        calculated_hash: u64,
    },

    SwitchResponse(SwitchResponse),
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SwitchResponse {
    Ok,
    Failure(SwitchFailureKind),
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SwitchFailureKind {
    HashDoesNotMatch,
    FirmwareWrite,
    Other(heapless::String<256>),
}
