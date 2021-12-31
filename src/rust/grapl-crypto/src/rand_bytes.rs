use rand_core::{OsRng, RngCore};

// Takes E bytes of random data and hashes it into N bytes
// This helps to lessen the impact accidentally low entropy.
// E must be >= N
pub(crate) fn rand_array<const E: usize, const N: usize>() -> [u8; N] {
    let mut entropy = [0; E];
    let mut output = [0; N];

    // Can remove this when const generics allow for comparisons in where clauses
    assert!(entropy.len() >= output.len());

    OsRng.fill_bytes(&mut entropy);

    let mut hasher = blake3::Hasher::new();
    hasher.update(&entropy);

    let mut output_reader = hasher.finalize_xof();
    output_reader.fill(&mut output);
    output
}
