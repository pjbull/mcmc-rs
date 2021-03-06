use crate::{Array1, Array2};
use anyhow::{anyhow, Error, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

/// Compute the arithmetic mean of an array.
pub(in crate) fn mean(arr: &[f64]) -> Result<f64, Error> {
    if arr.is_empty() {
        return Err(anyhow!("Can't take mean of empty array"));
    }
    let sum = arr.iter().sum::<f64>();
    let count = arr.len() as f64;
    Ok(sum / count)
}

/// Compute the sample variance of an array using Bessel's correction.
pub(in crate) fn sample_variance(arr: &[f64]) -> Result<f64, Error> {
    let xbar = mean(arr)?;
    Ok(arr.iter().map(|x| (x - xbar).powi(2)).sum::<f64>() / (arr.len() as f64 - 1.0))
}

/// Clone a 2D array into one long 1D array.
pub(in crate) fn flatten(chains: &Array2) -> Array1 {
    let mut flattened = Vec::new();
    for chain in chains {
        flattened.extend(chain);
    }
    flattened
}

/// Splits each chain into two chains of equal length.  When the
/// number of total draws N is odd, the (N+1)/2th draw is ignored.
///
/// See more details in Stan reference manual section
/// ["Effective Sample Size"](http://mc-stan.org/users/documentation).
///
/// Current implementation assumes chains are all of equal size.
pub fn split_chains(chains: Array2) -> Result<Array2, Error> {
    if chains.is_empty() {
        return Err(anyhow!("Can't split empty array of chains"));
    }
    let num_draws = chains.iter().map(|c| c.len()).min().unwrap();
    if num_draws < 1 {
        return Err(anyhow!("No samples to split"));
    }
    let (half, offset) = if num_draws % 2 == 0 {
        (num_draws / 2, 0)
    } else {
        ((num_draws - 1) / 2, 1)
    };
    let mut split_draws = Vec::new();
    for chain in chains {
        split_draws.push(chain[..half].to_vec());
        split_draws.push(chain[(half + offset)..].to_vec());
    }
    Ok(split_draws)
}

/// Simplified CSV reader for tesing purposes only; does not actually implement
/// parsing for headers, quotation, or other more advanced features. Assumes
/// that all values aside from the commas will be numeric.
///
/// # Arguments
/// * `skip_rows` - Number of rows to skip before numeric values. For example,
///                 if there is a header row you can pass in the value `1`.
/// * `n_rows` - Number of rows to read in. Use if you only want a certain
///              subset of rows or if there are improper rows after the numeric
///              rows (e.g. in Stan sample files there are commented rows at the end).
pub fn read_csv(path: &PathBuf, skip_rows: usize, n_rows: usize) -> Array2 {
    let mut result: Array2 = Vec::new();
    let f = File::open(&path).unwrap();
    let f = BufReader::new(f);
    for line in f.lines().skip(skip_rows).take(n_rows) {
        if let Ok(line) = line {
            for (idx, value) in line.split(',').enumerate() {
                if idx >= result.len() {
                    result.push(Vec::new())
                }
                result[idx].push(value.parse::<f64>().unwrap());
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Array1;

    #[test]
    fn test_stats() {
        // Test our basic stats functions using numbers computed with numpy.
        let arr = vec![
            2.13829088,
            -1.06214379,
            -0.79265699,
            -0.21300888,
            -1.07155142,
            -0.50425317,
            0.95708854,
            -1.23854172,
            1.37124938,
            1.17658286,
        ];
        let empty: Array1 = vec![];
        assert_abs_diff_eq!(
            sample_variance(&arr).unwrap(),
            1.492596054209826,
            epsilon = 1e-6
        );
        assert_abs_diff_eq!(mean(&arr).unwrap(), 0.07610557018217139, epsilon = 1e-6);

        assert!(sample_variance(&empty).is_err());
        assert!(mean(&empty).is_err());
    }

    #[test]
    fn test_split_empty_chains() {
        // Make sure the we Err on empty or minimum 0 length chains
        let a: Array1 = vec![1.0];
        let b: Array1 = vec![];
        let c: Array1 = vec![];
        let chains = vec![a, b, c];
        assert!(split_chains(chains).is_err());

        let a: Array1 = vec![];
        let b: Array1 = vec![];
        let chains = vec![a, b];
        assert!(split_chains(chains).is_err());
    }

    #[test]
    fn test_split_even_chains() {
        // Regular split with even numbers
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let chains = vec![a, b];
        let split = split_chains(chains).unwrap();
        assert_eq!(split[0], vec![1.0, 2.0]);
        assert_eq!(split[1], vec![3.0, 4.0]);
        assert_eq!(split[2], vec![5.0, 6.0]);
        assert_eq!(split[3], vec![7.0, 8.0]);
    }

    #[test]
    fn test_split_odd_chains() {
        // Make sure the middle value gets dropped per the Stan reference implementation
        let a = vec![1.0, 2.0, 3.0, 4.0, 4.5];
        let b = vec![5.0, 6.0, 7.0, 8.0, 8.5];
        let chains = vec![a, b];
        let split = split_chains(chains).unwrap();
        assert_eq!(split[0], vec![1.0, 2.0]);
        assert_eq!(split[1], vec![4.0, 4.5]);
        assert_eq!(split[2], vec![5.0, 6.0]);
        assert_eq!(split[3], vec![8.0, 8.5]);
    }

    #[test]
    fn test_flatten() {
        // Regular split with even numbers
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let chains = vec![a, b];
        let flattened = flatten(&chains);
        assert_eq!(flattened, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    }
}
