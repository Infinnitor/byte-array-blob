//! A crate for turning multiple slices of bytes (such as a slice returned by the `include_bytes` macro) into an encoded blob, and then decoding that blob back into a collection of slices.
//!
//! Any number of u8 slices can be grouped together as a 1 dimensional u8 slice with `blob_to_byte_arrays`, and then used for some other purpose (compression, storage, etc).
//!
//! The same encoded u8 slice can then be read into `byte_arrays_to_blob` and turned back into a Vec of u8 slices.
//!
//! # Examples
//! ```
//! use byte_array_blob::*;
//!
//! // Arbritrary data
//! let data1 = [1, 2, 3, 4, 5];
//! let data2 = [8; 64];
//!
//! let blob = byte_arrays_to_blob(&[&data1, &data2]);
//!
//! let decode = blob_to_byte_arrays(&blob);
//! let slices = decode.unwrap();
//! assert_eq!(slices[0], data1);
//! assert_eq!(slices[1], data2);
//! ```
use std::fmt;


/// Errors that can occur when reading a blob u8 slice
#[derive(Debug)]
pub enum BlobReadError {
	/// Occurs when a given index is invalid, either being outside the size of the slice, or pointing to earlier data.
	/// The enum yields the offending index
	///
	/// # Examples
	/// ```
	/// use byte_array_blob::{blob_to_byte_arrays, BlobReadError};
	///
	/// // The first four bytes point to u32::MAX, which is larger than the slice size
	/// let blob = [255u8, 255u8, 255u8, 255u8];
	/// match blob_to_byte_arrays(&blob).unwrap_err() {
	/// 	BlobReadError::InvalidEncodedIndex(idx) => assert_eq!(idx, u32::MAX as usize),
	/// 	_ => panic!("This should not happen")
	/// }
	/// ```
	InvalidEncodedIndex(usize),
	/// Occurs when the given slice is larger than `u32::MAX`. This is highly unlikely to occur
	TooLarge
}


impl std::error::Error for BlobReadError {}

impl fmt::Display for BlobReadError {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(formatter, "{:?}", self)
	}
}


/// Converts a blob u8 slice to a Vec of u8 slices
///
/// # Errors
///
/// The function returns `Err(BlobReadError)` when an index in the data is invalid, or when the length of the data is more than `u32::MAX`
///
/// # Examples
///
/// ```
/// use byte_array_blob::blob_to_byte_arrays;
///
/// // The first four bytes in the blob point to the end of the slice (index 8)
/// let blob = [8u8, 0u8, 0u8, 0u8, 255u8, 0u8, 0u8, 0u8];
/// let read_blob = byte_array_blob::blob_to_byte_arrays(&blob);
///
/// assert!(read_blob.is_ok());
/// if let Ok(data) = read_blob {
/// 	assert_eq!(data[0], &[255, 0, 0, 0]);
/// }
/// ```
pub fn blob_to_byte_arrays(blob: &[u8]) -> Result<Vec<&[u8]>, BlobReadError> {
	// It is difficult for this case to occur
	if blob.len() > u32::MAX as usize {
		return Err(BlobReadError::TooLarge);
	}

	let mut idx_end = 0;
	let mut idx_start = 0;

	let mut byte_arrays = Vec::new();

	// Read slices while the index is within bounds
	while idx_end < blob.len() {
		// Get encoded index as 4 bytes
		let idx_end_data = &blob[idx_start..idx_start+4];
		let idx_end_data: [u8; 4] = idx_end_data.try_into().unwrap();

		// Transform 4 bytes into u32, then usize
		idx_end = u32::from_ne_bytes(idx_end_data) as usize;
		idx_start += 4;

		// If the index end is invalid (more than the blob length, or less than the slice start) then return Err
		if idx_end > blob.len() || idx_end < idx_start {
			return Err(BlobReadError::InvalidEncodedIndex(idx_end));
		}

		// Push slice to byte_arrays
		byte_arrays.push(&blob[idx_start..idx_end]);
		idx_start = idx_end;
	}
	Ok(byte_arrays)
}


/// Converts a 2D array of u8 slices to a `Vec<u8>`.
///
/// An additional 4 bytes of data are allocated to the Vec for each slice, and are used to keep track of the indexes.
/// So the length of the Vec will be equal to the sum of the slices' lengths + 4*N, where N is the number of slices.
///
/// # Examples
/// ```
/// use byte_array_blob::*;
///
/// // Arbritrary data
/// let data1 = [1, 2, 3, 4, 5];
/// let data2 = [8; 64];
///
/// let blob = byte_arrays_to_blob(&[&data1, &data2]);
///
/// // An additional 4 bytes per slice are reserved to keep track of the indexes
/// let checksum_encoded = data1.len() + 4 + data2.len() + 4;
/// assert_eq!(blob.len(), checksum_encoded);
pub fn byte_arrays_to_blob(bytes_2d: &[&[u8]]) -> Vec<u8> {
	let mut idx_end = 0;
	let mut blob: Vec<u8> = Vec::new();

	for byte_arr in bytes_2d {
		idx_end += 4 + byte_arr.len();
		let write_end_idx: [u8; 4] = (idx_end as u32).to_ne_bytes();
		blob.extend(write_end_idx);
		blob.extend(byte_arr.into_iter());
	}
	blob
}




#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

	#[test]
	fn bytes_blob_test() {
		let bytes_arr: Vec<&[u8]> = vec![
			&[0u8, 0u8, 0u8, 0u8],
			&[0u8, 0u8, 0u8, 0u8]
		];

		let blob = byte_arrays_to_blob(&bytes_arr);
		assert_eq!(blob.len(), 16);
		assert_eq!(blob_to_byte_arrays(&blob).unwrap().len(), 2);
	}

	#[test]
	fn blob_read_invalid_index() {
		let blob = vec![255u8, 255u8, 255u8, 255u8];
		let read = blob_to_byte_arrays(&blob);

		if let Err(e) = blob_to_byte_arrays(&blob) {
			match e {
				BlobReadError::InvalidEncodedIndex(idx) => { assert_eq!(idx, u32::MAX as usize) },
				_ => { panic!("Error should be BlobReadError::InvalidEncodedIndex") }
			}
		}

		else {
			panic!("blob_to_byte_arrays() should return Err for invalid data");
		}
	}
}
