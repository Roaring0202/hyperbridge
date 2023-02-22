use alloc::vec::Vec;
use base2::Base2;
use ethereum_consensus::{
	bellatrix::{BeaconBlockHeader, SyncAggregate, SyncCommittee},
	domains::DomainType,
	primitives::{Epoch, Hash32, Slot},
};

pub const DOMAIN_SYNC_COMMITTEE: DomainType = DomainType::SyncCommittee;
pub const FINALIZED_ROOT_INDEX: u64 = 52;
pub const EXECUTION_PAYLOAD_STATE_ROOT_INDEX: u64 = 18;
pub const EXECUTION_PAYLOAD_BLOCK_NUMBER_INDEX: u64 = 22;
pub const EXECUTION_PAYLOAD_INDEX: u64 = 56;
pub const NEXT_SYNC_COMMITTEE_INDEX: u64 = 55;
pub const BLOCK_ROOTS_INDEX: u64 = 37;
pub const HISTORICAL_BATCH_BLOCK_ROOTS_INDEX: u64 = 0;
pub const HISTORICAL_ROOTS_INDEX: u64 = 39;
pub const GENESIS_VALIDATORS_ROOT: [u8; 32] =
	hex_literal::hex!("4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95");
// pub const NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2: usize = NEXT_SYNC_COMMITTEE_INDEX.floor_log2()
// as usize; pub const FINALIZED_ROOT_INDEX_FLOOR_LOG_2: usize = FINALIZED_ROOT_INDEX.floor_log2()
// as usize;

/// This holds the relevant data required to prove the state root in the execution payload.
#[derive(Debug, Clone)]
pub struct ExecutionPayloadProof {
	/// The state root in the `ExecutionPayload` which represents the commitment to
	/// the ethereum world state in the yellow paper.
	pub state_root: Hash32,
	/// the block number of the execution header.
	pub block_number: u64,
	/// merkle mutli proof for the state_root & block_number in the [`ExecutionPayload`].
	pub multi_proof: Vec<Hash32>,
	/// merkle proof for the `ExecutionPayload` in the [`BeaconBlockBody`].
	pub execution_payload_branch: Vec<Hash32>,
}

/// Holds the neccessary proofs required to verify a header in the `block_roots` field
/// either in [`BeaconState`] or [`HistoricalBatch`].
#[derive(Debug, Clone)]
pub struct BlockRootsProof {
	/// Generalized index of the header in the `block_roots` list.
	pub block_header_index: u64,
	/// The proof for the header, needed to reconstruct `hash_tree_root(state.block_roots)`
	pub block_header_branch: Vec<Hash32>,
}

/// The block header ancestry proof, this is an enum because the header may either exist in
/// `state.block_roots` or `state.historical_roots`.
#[derive(Debug, Clone)]
pub enum AncestryProof {
	/// This variant defines the proof data for a beacon chain header in the `state.block_roots`
	BlockRoots {
		/// Proof for the header in `state.block_roots`
		block_roots_proof: BlockRootsProof,
		/// The proof for the reconstructed `hash_tree_root(state.block_roots)` in [`BeaconState`]
		block_roots_branch: Vec<Hash32>,
	},
	/// This variant defines the neccessary proofs for a beacon chain header in the
	/// `state.historical_roots`.
	HistoricalRoots {
		/// Proof for the header in `historical_batch.block_roots`
		block_roots_proof: BlockRootsProof,
		/// The proof for the `historical_batch.block_roots`, needed to reconstruct
		/// `hash_tree_root(historical_batch)`
		historical_batch_proof: Vec<Hash32>,
		/// The proof for the `hash_tree_root(historical_batch)` in `state.historical_roots`
		historical_roots_proof: Vec<Hash32>,
		/// The generalized index for the historical_batch in `state.historical_roots`.
		historical_roots_index: u64,
		/// The proof for the reconstructed `hash_tree_root(state.historical_roots)` in
		/// [`BeaconState`]
		historical_roots_branch: Vec<Hash32>,
	},
}

/// This defines the neccesary data needed to prove ancestor blocks, relative to the finalized
/// header.
#[derive(Debug, Clone)]
pub struct AncestorBlock {
	/// The actual beacon chain header
	pub header: BeaconBlockHeader,
	/// Associated execution header proofs
	pub execution_payload: ExecutionPayloadProof,
	/// Ancestry proofs of the beacon chain header.
	pub ancestry_proof: AncestryProof,
}

/// Holds the latest sync committee as well as an ssz proof for it's existence
/// in a finalized header.
#[derive(Debug, Clone)]
pub struct SyncCommitteeUpdate<const SYNC_COMMITTEE_SIZE: usize> {
	// actual sync committee
	pub next_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
	// sync committee, ssz merkle proof.
	pub next_sync_committee_branch: Vec<Hash32>,
}

/// Minimum state required by the light client to validate new sync committee attestations
#[derive(Debug, Clone)]
pub struct LightClientState<const SYNC_COMMITTEE_SIZE: usize> {
	/// The latest recorded finalized header
	pub finalized_header: BeaconBlockHeader,
	// Sync committees corresponding to the finalized header
	pub current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
	pub next_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
}

/// Minimum state required by the light client to validate new sync committee attestations
#[derive(Debug, Clone)]
pub struct FinalityProof {
	/// Epoch that was finalized
	pub finalized_epoch: Epoch,
	/// the ssz merkle proof for the finalized checkpoint in the attested header, finalized headers
	/// lag by 2 epochs.
	pub finality_branch: Vec<Hash32>,
}

/// Data required to advance the state of the light client.
#[derive(Debug, Clone)]
pub struct LightClientUpdate<const SYNC_COMMITTEE_SIZE: usize> {
	/// the header that the sync committee signed
	pub attested_header: BeaconBlockHeader,
	/// the sync committee has potentially changed, here's an ssz proof for that.
	pub sync_committee_update: Option<SyncCommitteeUpdate<SYNC_COMMITTEE_SIZE>>,
	/// the actual header which was finalized by the ethereum attestation protocol.
	pub finalized_header: BeaconBlockHeader,
	/// execution payload of the finalized header
	pub execution_payload: ExecutionPayloadProof,
	/// Finalized header proof
	pub finality_proof: FinalityProof,
	/// signature & participation bits
	pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
	/// slot at which signature was produced
	pub signature_slot: Slot,
	/// ancestors of the finalized block to be verified, may be empty.
	pub ancestor_blocks: Vec<AncestorBlock>,
}
