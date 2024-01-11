use std::sync::Arc;

use brontes_database::{Metadata, Pair};
use brontes_database_libmdbx::Libmdbx;
use brontes_types::{
    classified_mev::{ClassifiedMev, Liquidation, MevType, SpecificMev},
    normalized_actions::{Actions, NormalizedLiquidation, NormalizedSwap},
    tree::{BlockTree, GasDetails, Node, Root},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reth_primitives::{Address, B256};

use crate::{shared_utils::SharedInspectorUtils, Inspector};

pub struct LiquidationInspector<'db> {
    inner: SharedInspectorUtils<'db>,
}

impl<'db> LiquidationInspector<'db> {
    pub fn new(quote: Address, db: &'db Libmdbx) -> Self {
        Self { inner: SharedInspectorUtils::new(quote, db) }
    }
}

#[async_trait::async_trait]
impl Inspector for LiquidationInspector<'_> {
    async fn process_tree(
        &self,
        tree: Arc<BlockTree<Actions>>,
        metadata: Arc<Metadata>,
    ) -> Vec<(ClassifiedMev, Box<dyn SpecificMev>)> {
        let liq_txs =
            tree.inspect_all(|node| node.subactions.iter().any(|action| action.is_liquidation()));

        liq_txs
            .into_par_iter()
            .filter_map(|(tx_hash, liq)| {
                let root = tree.get_root(tx_hash)?;
                let eoa = root.head.address;
                let mev_contract = root.head.data.get_to_address();
                let idx = root.get_block_position();
                let gas_details = tree.get_gas_details(tx_hash)?;

                self.calculate_liquidation(
                    tx_hash,
                    idx,
                    mev_contract,
                    eoa,
                    metadata.clone(),
                    liq,
                    gas_details,
                )
            })
            .collect::<Vec<_>>()
    }
}

impl LiquidationInspector<'_> {
    fn calculate_liquidation(
        &self,
        tx_hash: B256,
        idx: usize,
        mev_contract: Address,
        eoa: Address,
        metadata: Arc<Metadata>,
        liq: Vec<Vec<Actions>>,
        gas_details: &GasDetails,
    ) -> Option<(ClassifiedMev, Box<dyn SpecificMev>)> {
        let liq_swap_sequences =
            liq.iter()
                .map(|liq_swap_seq| {
                    liq_swap_seq
                        .iter()
                        .filter_map(|action| {
                            if let Actions::Swap(swap) = action {
                                Some(swap)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

        let liqs = liq.iter()
            .map(|l| {
                l.iter()
                    .filter_map(|action| {
                        if let Actions::Liquidation(liq) = action {
                            Some(liq)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let flat_swaps = liq.into_iter().flatten().collect::<Vec<_>>();

        let mev = ClassifiedMev {
            block_number: metadata.block_num,
            eoa,
            tx_hash,
            mev_contract,
            mev_profit_collector: todo!(),
            finalized_profit_usd: todo!(),
            finalized_bribe_usd: todo!(),
            mev_type: MevType::Liquidation,
        };

        // TODO: filter swaps not related to liqs?
        let new_liquidation = Liquidation {
            liquidation_tx_hash: tx_hash,
            trigger: B256::default(),
            liquidation_swaps_index: flat_swaps.iter()
                .filter(|s| s.is_swap())
                .map(|s| s.clone().force_swap().trace_index)
                .collect::<Vec<_>>(),
            liquidation_swaps_from: flat_swaps.iter()
                .filter(|s| s.is_swap())
                .map(|s| s.clone().force_swap().from)
                .collect::<Vec<_>>(),
            liquidation_swaps_pool: flat_swaps.iter()
                .filter(|s| s.is_swap())
                .map(|s| s.clone().force_swap().pool)
                .collect::<Vec<_>>(),
            liquidation_swaps_token_in: flat_swaps.iter()
                .filter(|s| s.is_swap())
                .map(|s| s.clone().force_swap().token_in)
                .collect::<Vec<_>>(),
            liquidation_swaps_token_out: flat_swaps.iter()
                .filter(|s| s.is_swap())
                .map(|s| s.clone().force_swap().token_out)
                .collect::<Vec<_>>(),
            liquidation_swaps_amount_in: flat_swaps.iter()
                .filter(|s| s.is_swap())
                .map(|s| s.clone().force_swap().amount_in.to())
                .collect::<Vec<_>>(),
            liquidation_swaps_amount_out: flat_swaps.iter()
                .filter(|s| s.is_swap())
                .map(|s| s.clone().force_swap().amount_out.to())
                .collect::<Vec<_>>(),
            liquidations_index: todo!(),
            liquidations_liquidator: todo!(),
            liquidations_liquidatee: todo!(),
            liquidations_tokens: todo!(),
            liquidations_amounts: todo!(),
            liquidations_rewards: todo!(),
            gas_details: gas_details.clone(),
        };

        Some((mev, Box::new(new_liquidation)))
    }
}
