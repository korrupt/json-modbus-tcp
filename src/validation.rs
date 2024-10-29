use std::time::Duration;

use ipnetwork::IpNetwork;


pub fn parse_whitelist(
    target: Vec<String>,
) -> Result<(Option<Vec<IpNetwork>>, Option<Vec<IpNetwork>>), String> {
    let mut read_whitelist: Vec<IpNetwork> = Vec::new();
    let mut write_whitelist: Vec<IpNetwork> = Vec::new();

    for cidr_string in &target {
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
            read_whitelist.push(net);
        }

        if matches!(op, Op::Write | Op::ReadWrite) {
            write_whitelist.push(net);
        }
    }

    Ok((
        Some(read_whitelist).filter(|w| w.len() > 0),
        Some(write_whitelist).filter(|w| w.len() > 0),
    ))
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


pub fn validate_time(val: &str) -> Result<Duration, String> {
    if let Some(suffix) = val.strip_suffix("ms") {
        if let Ok(num) = suffix.parse::<u64>() {
            return Ok(Duration::from_millis(num));
        }
    } else if let Some(suffix) = val.strip_suffix("us") {
        if let Ok(num) = suffix.parse::<u64>() {
            return Ok(Duration::from_micros(num));
        }
    } else if let Some(suffix) = val.strip_suffix("s") {
        if let Ok(num) = suffix.parse::<u64>() {
            return Ok(Duration::from_secs(num));
        }
    }

    Err(String::from(
        "The time must be a whole number suffixed by 's', 'ms', or 'us'",
    ))
}