use criterion::{black_box, criterion_group, criterion_main, Benchmark};
use criterion::{BenchmarkId, Criterion};
use eth2_libp2p::types::{GossipEncoding, GossipKind};
use eth2_libp2p::{GossipTopic, PubsubMessage};
use libp2p::gossipsub::IdentTopic;
use ssz::Encode;
use state_processing::test_utils::BlockBuilder;
use types::{
    BeaconState, ChainSpec, EthSpec, MainnetEthSpec, MinimalEthSpec, SignedBeaconBlock, Slot,
};
use sha2::{Sha256, Digest};

pub const VALIDATORS_LOW: usize = 32_768;
pub const VALIDATORS_HIGH: usize = 300_032;

fn get_large_block<T: EthSpec>(validator_count: usize) -> (SignedBeaconBlock<T>, BeaconState<T>) {
    let spec = &mut T::default_spec();
    let mut builder: BlockBuilder<T> = BlockBuilder::new(validator_count, &spec);
    builder.maximize_block_operations();

    // FIXME: enable deposits once we can generate them with valid proofs.
    builder.num_deposits = 0;

    builder.set_slot(Slot::from(T::slots_per_epoch() * 3 - 2));
    builder.build_caches(&spec);
    builder.build(&spec)
}

fn get_average_block<T: EthSpec>(validator_count: usize) -> (SignedBeaconBlock<T>, BeaconState<T>) {
    let spec = &mut T::default_spec();
    let mut builder: BlockBuilder<T> = BlockBuilder::new(validator_count, &spec);
    // builder.num_attestations = T::MaxAttestations::to_usize();
    builder.num_attestations = 16;
    builder.set_slot(Slot::from(T::slots_per_epoch() * 3 - 2));
    builder.build_caches(&spec);
    builder.build(&spec)
}

pub fn decode_message_benchmark<T: EthSpec>(
    c: &mut Criterion,
    block: (SignedBeaconBlock<T>, BeaconState<T>),
    name: &str
) {
    let (block, state) = block;
    let message = PubsubMessage::BeaconBlock(Box::new(block));
    let encoded = message.encode(GossipEncoding::SSZSnappy).unwrap();

    println!("Message data size: {} bytes", encoded.len());

    let fork_digest =
        ChainSpec::compute_fork_digest(state.fork.current_version, state.genesis_validators_root);

    let topic: IdentTopic = GossipTopic::new(
        GossipKind::BeaconBlock,
        GossipEncoding::SSZSnappy,
        fork_digest,
    )
    .into();

    let topic_hashes = vec![topic.hash()];
    let slice = topic_hashes.as_slice();

    c.bench_function(name, |b| {
        b.iter(|| PubsubMessage::<MainnetEthSpec>::decode(black_box(slice), black_box(&encoded)));
    });
}

pub fn hash_message_benchmark<T: EthSpec>(
    c: &mut Criterion,
    block: (SignedBeaconBlock<T>, BeaconState<T>),
    name: &str
) {
    let (block, state) = block;
    let message = PubsubMessage::BeaconBlock(Box::new(block));
    let encoded = message.encode(GossipEncoding::SSZSnappy).unwrap();

    c.bench_function(name, |b| {
        b.iter(|| {
            Sha256::digest(black_box(&encoded));
        });
    });
}

fn all_benches(c: &mut Criterion) {
    decode_message_benchmark::<MainnetEthSpec>(
        c,
        get_large_block::<MainnetEthSpec>(VALIDATORS_HIGH),
        "ssz decode large block with high validators"
    );
    decode_message_benchmark::<MainnetEthSpec>(
        c,
        get_large_block::<MainnetEthSpec>(VALIDATORS_LOW),
        "ssz decode large block with low validators"
    );
    decode_message_benchmark::<MainnetEthSpec>(
        c,
        get_average_block::<MainnetEthSpec>(VALIDATORS_HIGH),
        "ssz decode average block with high validators"
    );
    decode_message_benchmark::<MainnetEthSpec>(
        c,
        get_average_block::<MainnetEthSpec>(VALIDATORS_LOW),
        "ssz decode average block with low validators"
    );

    hash_message_benchmark::<MainnetEthSpec>(
        c,
        get_large_block::<MainnetEthSpec>(VALIDATORS_HIGH),
        "hash large block with high validators"
    );
    hash_message_benchmark::<MainnetEthSpec>(
        c,
        get_large_block::<MainnetEthSpec>(VALIDATORS_LOW),
        "hash large block with low validators"
    );
    hash_message_benchmark::<MainnetEthSpec>(
        c,
        get_average_block::<MainnetEthSpec>(VALIDATORS_HIGH),
        "hash average block with high validators"
    );
    hash_message_benchmark::<MainnetEthSpec>(
        c,
        get_average_block::<MainnetEthSpec>(VALIDATORS_LOW),
        "hash average block with low validators"
    );
}

criterion_group!(benches, all_benches);
criterion_main!(benches);
