// <FILE>src/shuffle/fnc_overhand_shuffle.rs</FILE> - <DESC>Casual overhand shuffle simulation</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation of overhand shuffle with geometric chunks</CLOG>

use crate::rng::Rng;

/// Overhand shuffle simulation.
///
/// Simulates the casual overhand shuffling technique where small chunks are
/// repeatedly cut from the back and placed on top. Chunk sizes follow a
/// geometric-like distribution (small chunks more likely).
///
/// # Algorithm
///
/// For each pass:
/// 1. Start with full deck "in hand"
/// 2. Repeatedly cut random-sized chunks from back
/// 3. Place each chunk on top of result pile
/// 4. Repeat for specified number of passes
///
/// - Time complexity: O(n) per pass
/// - Space complexity: O(n) for temporary buffer
///
/// # Arguments
///
/// * `slice` - The slice to shuffle in place
/// * `passes` - Number of overhand passes (20+ recommended for good mixing)
/// * `rng` - Random number generator
///
/// # Mixing Quality
///
/// Overhand shuffle is weaker than riffle:
/// - 1-5 passes: Minimal mixing, clumping visible
/// - 10-20 passes: Moderate mixing
/// - 50+ passes: Approaching uniform
///
/// The characteristic "clumping" pattern makes it feel more casual/amateur.
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::overhand_shuffle;
///
/// let mut rng = Rng::with_seed(42);
/// let mut deck: Vec<i32> = (1..=52).collect();
///
/// overhand_shuffle(&mut deck, 20, &mut rng);
/// // deck is now shuffled with casual overhand mechanics
/// ```
pub fn overhand_shuffle<T: Clone>(slice: &mut [T], passes: usize, rng: &mut Rng) {
    let len = slice.len();
    if len <= 1 || passes == 0 {
        return;
    }

    for _ in 0..passes {
        overhand_once(slice, rng);
    }
}

/// Perform a single overhand shuffle pass.
fn overhand_once<T: Clone>(slice: &mut [T], rng: &mut Rng) {
    let len = slice.len();
    if len <= 1 {
        return;
    }

    let mut hand: Vec<T> = slice.to_vec();
    let mut chunks: Vec<Vec<T>> = Vec::new();

    // Cut chunks from back of hand and place on top of result
    while !hand.is_empty() {
        // Geometric-like chunk size: smaller chunks more likely
        // Average chunk size ~ len/5, range 1 to remaining
        let max_chunk = hand.len();
        let avg_chunk = (max_chunk / 5).max(1);

        // Use exponential-ish distribution: -ln(U) * avg scaled to max
        let u = rng.uniform(0.01, 1.0);
        let raw_size = (-u.ln() * avg_chunk as f32).ceil() as usize;
        let chunk_size = raw_size.clamp(1, max_chunk);

        // Take chunk from back
        let chunk_start = hand.len() - chunk_size;
        let chunk: Vec<T> = hand.drain(chunk_start..).collect();

        // Store chunks in removal order; we'll assemble in reverse to avoid O(n^2) prepends
        chunks.push(chunk);
    }

    // Assemble final order: last chunk drawn sits on top
    let mut result: Vec<T> = Vec::with_capacity(len);
    for chunk in chunks.into_iter().rev() {
        result.extend(chunk);
    }
    slice.clone_from_slice(&result);
}

// <FILE>src/shuffle/fnc_overhand_shuffle.rs</FILE> - <DESC>Casual overhand shuffle simulation</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
