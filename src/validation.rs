use ipnetwork::IpNetwork;

#[derive(Clone, Debug)]
pub struct Whitelist {
    pub read: Vec<IpNetwork>,
    pub write: Vec<IpNetwork>,
}

impl Default for Whitelist {
    fn default() -> Self {
        Whitelist { read: Vec::new(), write: Vec::new() }
    }
}

pub fn parse_whitelist(
    target: &Vec<String>,
) -> Result<Whitelist, String> {
    let mut read: Vec<IpNetwork> = Vec::new();
    let mut write: Vec<IpNetwork> = Vec::new();

    for cidr_string in target {
        let (net, op) = cidr_string
        .find(":")
        .map_or(Ok((cidr_string.as_str(), Op::ReadWrite)), |idx| {
            Op::parse(&cidr_string[(idx + 1)..]).map(|op| (&cidr_string[..idx], op))
        })
        .and_then(|(c, op)| {
            c.parse::<IpNetwork>()
                .map(|network| (network, op))
                .map_err(|e| format!("Error parsing CIDR part of string: {}", e))
        })?;

        if matches!(op, Op::Read | Op::ReadWrite) {
            read.push(net);
        }

        if matches!(op, Op::Write | Op::ReadWrite) {
            write.push(net);
        }
    }

    Ok(Whitelist { read, write })
}


pub enum Op {
    Read,
    Write,
    ReadWrite,
}

impl Op {
    pub fn parse(target: &str) -> Result<Self, String> {
        match target {
            "r" => Ok(Self::Read),
            "w" => Ok(Self::Write),
            "rw" => Ok(Self::ReadWrite),
            _ => Err("Error parsing operation".into()),
        }
    }
}