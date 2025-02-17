use std::collections::VecDeque;

pub fn mean(input: &[f32]) -> f32 {
    let l: f32 = input.len() as f32;
    let s: f32 = input.iter().sum();
    s / l
}

pub fn mean_abs(input: &[f32]) -> f32 {
    let l: f32 = input.len() as f32;
    let s: f32 = input.iter().map(|n| n.abs()).sum();
    s / l
}

pub fn mean_nonzero(input: VecDeque<f32>) -> f32 {
    let i = input.iter().map(|n| n.abs()).filter(|n| *n != 0.0);
    let s: f32 = i.clone().sum();
    let l: f32 = i.count() as f32;
    if l != 0.0 {
        s / l
    } else {
        1.0
    }
}

/// Window is:
///
/// 1 2 3 4 5 6 7
/// ~~~~|~~~~
/// ^~~^
///  |
/// window
/// On one side
pub fn moving_average<O>(input: Vec<f32>, window: usize) -> O
where
    // I: Index<std::ops::Range<usize>, Output = [f32]>,
    O: std::iter::FromIterator<f32>,
{
    let l = input.len();
    let mut res = Vec::new();

    for i in 0..l {
        let start = if i > window { i - window } else { 0 };
        let end = std::cmp::min(l, i + window);

        res.push(mean(&input[start..end]));
    }

    res.into_iter().collect()
}

pub fn smooth_directional(prev: Vec<f32>, new: Vec<f32>, k_up: f32, key_down: f32) -> Vec<f32> {
    std::iter::zip(prev, new)
        .map(|(p, n)| {
            let k = if n > p { k_up } else { key_down };

            p + (n - p) * k
        })
        .collect()
}

pub fn fade_linear<T: std::iter::FromIterator<f32>>(input: Vec<f32>, by: f32) -> T {
    input
        .iter()
        .map(|n| n - by)
        .map(|n| if n > 0.0 { n } else { 0.0 })
        .collect()
}

pub fn fade_exponent<T: std::iter::FromIterator<f32>>(input: Vec<f32>, by: f32) -> T {
    input
        .iter()
        .enumerate()
        .map(|(i, n)| n - by * i as f32)
        .map(|n| if n > 0.0 { n } else { 0.0 })
        .collect()
}
