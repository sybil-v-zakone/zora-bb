use std::fs::File;
use std::str::FromStr;

use csv::WriterBuilder;
use serde::Serialize;
use tabled::{Table, Tabled};

use alloy::{
    network::Ethereum,
    primitives::{Address, U256, address, utils::format_units},
    providers::{MulticallBuilder, ProviderBuilder},
    sol,
};

use crate::{config::Config, fs::read_lines};

sol! {
    #[sol(rpc)]
    contract ZoraTokenCommunityClaim {
        function allocations(address account) public view virtual returns (uint256);
    }
}

const ADDRESSES_FP: &str = "data/addresses.txt";
const ZORA_TOKEN_COMMUNITY_CLAIM_CA: Address =
    address!("0x0000000002ba96C69b95E32CAAB8fc38bAB8B3F8");

async fn get_stats(addresses: &[Address], config: Config) -> eyre::Result<Vec<U256>> {
    let provider = ProviderBuilder::new()
        .disable_recommended_fillers()
        .network::<Ethereum>()
        .on_http(config.base_rpc_url.parse()?);

    let claim_instance =
        ZoraTokenCommunityClaim::new(ZORA_TOKEN_COMMUNITY_CLAIM_CA, provider.clone());
    let mut multicall_builder = MulticallBuilder::new_dynamic(provider.clone());

    for a in addresses {
        multicall_builder = multicall_builder.add_dynamic(claim_instance.allocations(*a));
    }

    let out = multicall_builder.aggregate().await?;
    Ok(out)
}

pub async fn parse_stats() -> eyre::Result<()> {
    let lines = read_lines(ADDRESSES_FP).await?;
    let addresses = {
        let mut out = vec![];
        for l in lines {
            out.push(
                Address::from_str(&l)
                    .map_err(|e| eyre::eyre!("provided address is not valid `{l}`: {e}"))?,
            );
        }

        out
    };

    let cfg = Config::read_default().await;
    let allocations = get_stats(&addresses, cfg)
        .await?
        .into_iter()
        .map(|alloc| format_units(alloc, 18).unwrap())
        .collect::<Vec<_>>();

    let mut table = vec![];

    for (addr, alloc) in addresses.into_iter().zip(allocations) {
        let entry = WalletStats {
            address: addr,
            allocation: alloc,
        };

        table.push(entry);
    }

    WalletStats::export_stats_to_csv(&table)?;
    let table = Table::new(table);
    println!("{table}");

    Ok(())
}

#[derive(Serialize, Tabled, Clone, Debug)]
pub struct WalletStats {
    #[tabled(rename = "Address")]
    #[serde(rename = "Address")]
    pub address: Address,

    #[tabled(rename = "Allocation")]
    #[serde(rename = "Allocation")]
    pub allocation: String,
}

impl WalletStats {
    const EXPORT_FILE_PATH: &str = "data/stats.csv";

    pub fn export_stats_to_csv<T: Serialize>(entries: &[T]) -> eyre::Result<()> {
        let export_file = File::create(Self::EXPORT_FILE_PATH)?;

        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(export_file);

        for entry in entries {
            writer.serialize(entry)?
        }

        writer.flush()?;

        tracing::info!("Stats exported to {}", Self::EXPORT_FILE_PATH);

        Ok(())
    }
}
