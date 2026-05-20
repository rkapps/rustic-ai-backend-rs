//! Vector similarity search utilities.
//!
//! [`similarity`] provides [`similarity::cosine_similarity`] for scoring a
//! pair of vectors, and [`similarity::search`] for ranking a collection of
//! candidates against a query and returning the top-k results.  These are used
//! by `rustic-storage` to implement semantic search on top of any
//! [`rustic_storage::core::repository::Repository`].

pub mod similarity;
