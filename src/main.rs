use std::io::{Error, ErrorKind};

use clap::{App, AppSettings, Arg};
use cloudflare::endpoints::dns::{DnsRecord, ListDnsRecords};
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::endpoint::{Endpoint, Method};
use exitfailure::ExitFailure;
use failure::ResultExt;
use serde::Serialize;

fn main() -> Result<(), ExitFailure> {
    let matches = App::new("cloudflare-ddns")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Enable verbose logging"))
        .arg(Arg::with_name("auth-token")
            .long("auth-token")
            .help("API token generated on the \"My Account\" page")
            .takes_value(true)
            .allow_hyphen_values(true)
            .required(true))
        .arg(Arg::with_name("zone-id")
            .long("zone-id")
            .help("Zone ID from domain \"Overview\" page, \"API\" section")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("record-name")
            .long("record-name")
            .help("DNS record \"name\" from domain \"DNS\" page")
            .takes_value(true)
            .required(true))
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    let token = matches.value_of("auth-token").unwrap(); // safe because required
    let zone_id = matches.value_of("zone-id").unwrap(); // safe because required
    let record_name = matches.value_of("record-name").unwrap(); // safe because required
    let verbose_logging = matches.is_present("verbose");

    let public_ip = reqwest::blocking::get("https://api.ipify.org")
        .and_then(|response| response.text())
        .context("Unable to reach ipify.org to resolve public IP")?
        .to_owned();
    println!("Public IP: {}", &public_ip);

    let api_client = HttpApiClient::new(
        Credentials::UserAuthToken {
            token: token.to_string(),
        },
        HttpApiClientConfig::default(),
        Environment::Production,
    )?;

    let record_list: Vec<DnsRecord> = api_client.request(
        &ListDnsRecords {
            zone_identifier: zone_id,
            params: Default::default(),
        })
        .context("Unable to list DNS records")?
        .result;
    if verbose_logging {
        println!("Found {} DNS records", &record_list.len());
    }

    // TODO filter by A records only.
    let record = record_list.iter()
        .find(|record| record.name == record_name)
        .ok_or(Error::new(ErrorKind::InvalidData, "No DNS record found with specified name"))
        .with_context(|_| {
            let dns_names: Vec<String> = record_list.iter().map(|record| record.name.to_owned()).collect();
            return format!("No matching DNS record in [{:?}]", dns_names);
        })?;
    let record_id = &record.id;
    if verbose_logging {
        println!("Current {:#?}", &record);
    }

    let new_record: DnsRecord = api_client.request(
        &PatchDnsRecord {
            zone_identifier: zone_id,
            record_identifier: record_id,
            params: PatchDnsRecordParams {
                content: &public_ip
            },
        })
        .context("Unable to update DNS record")?
        .result;
    println!("Successfully updated {}!", &record_name);
    if verbose_logging {
        println!("New {:#?}", &new_record)
    }

    Ok(())
}

// TODO upstream into cloudflare-rs
struct PatchDnsRecord<'a> {
    zone_identifier: &'a str,
    record_identifier: &'a str,
    params: PatchDnsRecordParams<'a>,
}
impl<'a> Endpoint<DnsRecord, (), PatchDnsRecordParams<'a>> for PatchDnsRecord<'a> {
    fn method(&self) -> Method {
        Method::Patch
    }
    fn path(&self) -> String {
        format!("zones/{}/dns_records/{}", self.zone_identifier, self.record_identifier)
    }
    fn body(&self) -> Option<PatchDnsRecordParams<'a>> {
        Some(self.params.clone())
    }
}

#[derive(Serialize, Clone, Debug, Default)]
struct PatchDnsRecordParams<'a> {
    content: &'a str,
}
