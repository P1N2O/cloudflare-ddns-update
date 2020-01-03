use clap::{App, AppSettings, Arg};
use cloudflare::endpoints::dns::{DnsRecord, ListDnsRecords};
use cloudflare::framework::{Environment, HttpApiClient, HttpApiClientConfig};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::endpoint::{Endpoint, Method};
use serde::Serialize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = App::new("cloudflare-ddns")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
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
        .setting(AppSettings::ArgRequiredElseHelp);

    let matches = cli.get_matches();
    let token = matches.value_of("auth-token").unwrap();
    let zone_id = matches.value_of("zone-id").unwrap();
    let record_name = matches.value_of("record-name").unwrap();

    let api_client = HttpApiClient::new(
        Credentials::UserAuthToken {
            token: token.to_string(),
        },
        HttpApiClientConfig::default(),
        Environment::Production,
    )?;

    let record_list_response = api_client.request(&ListDnsRecords {
        zone_identifier: zone_id,
        params: Default::default()
    });
    let record_list: Vec<DnsRecord> = record_list_response.unwrap().result;

    let record = record_list.iter().find(|record| record.name == record_name).unwrap();
    let record_id = &record.id;

    let public_ip = reqwest::blocking::get("https://api.ipify.org")?.text()?.to_owned();

    let record_patch_response = api_client.request(&PatchDnsRecord {
        zone_identifier: zone_id,
        record_identifier: record_id,
        params: PatchDnsRecordParams {
            content: &public_ip
        }
    });
    let new_record: DnsRecord = record_patch_response.unwrap().result;

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
