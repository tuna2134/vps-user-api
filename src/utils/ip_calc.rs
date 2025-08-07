pub fn cidr_to_list(cidr: &str) -> anyhow::Result<(Vec<String>, String)> {
    let (base_ip, prefix) = cidr
        .split_once('/')
        .ok_or(anyhow::anyhow!("Invalid CIDR format"))?;
    let base_parts: Vec<i32> = base_ip
        .split('.')
        .map(|s| s.parse::<i32>())
        .collect::<Result<Vec<_>, _>>()?;
    let ip_count = 1 << (32 - prefix.parse::<u8>()?);
    let mut ips = Vec::with_capacity(ip_count as usize);

    for i in 0..ip_count {
        let mut ip_parts = base_parts.clone();
        ip_parts[3] += i;

        for j in (0..4).rev() {
            if ip_parts[j] > 255 {
                if j == 0 {
                    return anyhow::bail!("IP address overflow");
                }
                ip_parts[j - 1] += ip_parts[j] / 256;
                ip_parts[j] %= 256;
            }
        }
        ips.push(
            ip_parts
                .iter()
                .map(|part| part.to_string())
                .collect::<Vec<_>>()
                .join(".")
                + "/"
                + prefix,
        );
    }
    // remove first and last IPs if they are network and broadcast addresses
    if ips.len() > 2 {
        ips.remove(0);
        ips.pop();
    }
    Ok((ips, prefix.to_string()))
}
