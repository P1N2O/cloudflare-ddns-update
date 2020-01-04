use std::io::{Error, ErrorKind};
use std::net::Ipv4Addr;
use std::str::FromStr;

use cloudflare::endpoints::dns::{DnsContent, DnsRecord, ListDnsRecords};
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::endpoint::{Endpoint, Method};
use exitfailure::ExitFailure;
use failure::ResultExt;
use serde::Serialize;

mod args;

fn main() -> Result<(), ExitFailure> {
    let args::Args { auth_token, zone_id, record_name, verbose } = args::parse_args();

    let public_ip_str = reqwest::blocking::get("https://api.ipify.org")
        .and_then(|response| response.text())
        .context("Unable to reach ipify.org to resolve public IP")?;
    let public_ip = Ipv4Addr::from_str(&public_ip_str)
        .with_context(|_| format!("Unable to parse {} as IP", &public_ip_str))?;
    println!("Public IP: {}", &public_ip);

    let cloudflare_client = HttpApiClient::new(
        Credentials::UserAuthToken {
            token: auth_token
        },
        HttpApiClientConfig::default(),
        Environment::Production,
    )?;

    let record_list: Vec<DnsRecord> = cloudflare_client.request(
        &ListDnsRecords {
            zone_identifier: &zone_id,
            params: Default::default(),
        })
        .context("Unable to list DNS records")?
        .result;
    if verbose {
        println!("Found {} DNS records", &record_list.len());
    }

    let a_records: Vec<DnsRecord> = record_list.into_iter()
        .filter(|record| {
            return match &record.content {
                DnsContent::A { .. } => true,
                _ => false,
            }
        }).collect();
    let record = a_records.iter()
        .find(|record| record.name == record_name)
        .ok_or(Error::new(ErrorKind::InvalidData, "No DNS record found with specified name"))
        .with_context(|_| {
            let dns_names: Vec<String> = a_records.iter()
                .map(|record| {
                    let ip = match &record.content {
                        DnsContent::A { content: ip } => ip,
                        _ => unreachable!(), // Source Vec only contains A records.
                    };
                    return format!("A {} {}", &record.name, ip)
                })
                .collect();
            return format!("No matching DNS record in {:?}", dns_names);
        })?;
    let record_id = &record.id;
    if verbose {
        println!("Current {:#?}", &record);
    }

    match &record.content {
        DnsContent::A { content } if *content == public_ip => {
            println!("{} already up-to-date!", &record_name)
        },
        _ => {
            let new_record: DnsRecord = cloudflare_client.request(
                &PatchDnsRecord {
                    zone_identifier: &zone_id,
                    record_identifier: &record_id,
                    params: PatchDnsRecordParams {
                        content: Some(DnsContent::A {
                            content: public_ip
                        }),
                        ..Default::default()
                    },
                })
                .context("Unable to update DNS record")?
                .result;
            println!("{} successfully updated!", &record_name);
            if verbose {
                println!("New {:#?}", &new_record)
            }
        }
    }

    Ok(())
}

// TODO switch to https://github.com/cloudflare/cloudflare-rs/pull/76 once released
struct PatchDnsRecord<'a> {
    zone_identifier: &'a str,
    record_identifier: &'a str,
    params: PatchDnsRecordParams,
}
impl<'a> Endpoint<DnsRecord, (), PatchDnsRecordParams> for PatchDnsRecord<'a> {
    fn method(&self) -> Method {
        Method::Patch
    }
    fn path(&self) -> String {
        format!("zones/{}/dns_records/{}", self.zone_identifier, self.record_identifier)
    }
    fn body(&self) -> Option<PatchDnsRecordParams> {
        Some(self.params.clone())
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
struct PatchDnsRecordParams {
    name: Option<String>,
    #[serde(flatten)]
    content: Option<DnsContent>,
    ttl: Option<u32>,
    proxied: Option<bool>,
}
