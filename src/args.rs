use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(name = "cloudflare-ddns-update", global_setting = AppSettings::AllowLeadingHyphen)]
pub struct Args {
    /// API token generated on the "My Account" page
    #[structopt(long)]
    pub auth_token: String,
    /// Zone ID from domain "Overview" page, "API" section
    #[structopt(long)]
    pub zone_id: String,
    /// DNS record "name" from domain "DNS" page
    #[structopt(long)]
    pub record_name: String,
    /// Enable verbose logging
    #[structopt(short, long)]
    pub verbose: bool,
}

pub fn parse_args() -> Args {
    Args::from_args()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn token_leading_dash() {
        let actual = Args::from_iter(&[
            "app",
            "--auth-token",
            "-AT",
            "--zone-id",
            "ZID",
            "--record-name",
            "RN",
        ]);
        let expected = Args {
            auth_token: "-AT".to_owned(),
            zone_id: "ZID".to_owned(),
            record_name: "RN".to_owned(),
            verbose: false,
        };
        assert_eq!(actual, expected)
    }
}
