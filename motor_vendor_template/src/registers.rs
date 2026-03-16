#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterDataType {
    Float,
    UInt32,
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterInfo {
    pub rid: u8,
    pub variable: &'static str,
    pub description: &'static str,
    pub access: RegisterAccess,
    pub data_type: RegisterDataType,
}

// TODO: Replace with real register table for your vendor.
pub static REGISTER_TABLE: &[RegisterInfo] = &[];
