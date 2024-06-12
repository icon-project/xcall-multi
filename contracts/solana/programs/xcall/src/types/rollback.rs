use xcall_lib::network_address::NetworkAddress;

pub struct Rollback {
    from: String,
    to: NetworkAddress,
    protocols: Vec<String>,
    rollback: Vec<u8>,
    enabled: bool,
}

impl Rollback {
    pub fn new(
        from: String,
        to: NetworkAddress,
        protocols: Vec<String>,
        rollback: Vec<u8>,
        enabled: bool,
    ) -> Self {
        Self {
            from,
            to,
            rollback,
            protocols,
            enabled,
        }
    }

    pub fn from(&self) -> &String {
        &self.from
    }

    pub fn to(&self) -> &NetworkAddress {
        &self.to
    }

    pub fn rollback(&self) -> &[u8] {
        &self.rollback
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn protocols(&self) -> &Vec<String> {
        &self.protocols
    }

    pub fn enable_rollback(&mut self){
        self.enabled = true;
    }
}
