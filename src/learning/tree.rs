//! Decision Tree Module
//!
//! Contains implementation of decision tree.
//!
//! The Decisin Tree models currently only support binary tree.
//! The model inputs should be a matrix and the training targets are
//! in the form of a vector of usize target labels, like 0, 1, 2...
//!
//! # Examples
//!
//! ```
//! use rusty_machine::learning::tree::DecisionTreeClassifier;
//! use rusty_machine::learning::SupModel;
//!
//! use rusty_machine::linalg::Matrix;
//! use rusty_machine::datasets::iris;
//!
//! let (inputs, targets) = iris::load_iris();
//! let mut tree = DecisionTreeClassifier::default();
//!
//! // Train the model
//! tree.train(&inputs, &targets).unwrap();
//!
//! // Now we'll predict a new point
//! let new_data = Matrix::new(1, 4, vec![4.2, 3.3, 1.6, 0.4]);
//! let output = tree.predict(&new_data).unwrap();
//!
//! // Hopefully we classified our new point correctly!
//! println!("{}", output[0]);
//! assert!(output[0] == 0, "Our classifier isn't very good!");
//! ```

use std::collections::BTreeMap;

use linalg::{Matrix, BaseMatrix};
use linalg::Vector;

use learning::{LearningResult, SupModel};
use learning::error::{Error, ErrorKind};

/// Tree node
#[derive(Debug)]
struct Node {
    feature_index: usize,
    threshold: f64,
    left: Link,
    right: Link
}

/// Tree link (leaf or branch)
///
/// Leaf contains a label to predict
#[derive(Debug)]
enum Link {
    Leaf(usize),
    Branch(Box<Node>),
}

/// Decision Tree
#[derive(Debug)]
pub struct DecisionTreeClassifier {

    criterion: Metrics,
    max_depth: Option<usize>,
    min_samples_split: Option<usize>,

    // params set after train
    n_classes: usize,
    n_features: usize,
    root: Option<Link>
}

/// The default Decision Tree.
///
/// The defaults are:
///
/// - `max_depth` = `None`
/// - `min_samples_split` = `None`
impl Default for DecisionTreeClassifier {
    fn default() -> Self {
        DecisionTreeClassifier{ criterion: Metrics::Gini,
                                max_depth: None,
                                min_samples_split: None,
                                n_classes: 0,
                                n_features: 0,
                                root: None }
    }
}

impl DecisionTreeClassifier {

    /// Constructs an untrained Decision Tree with specified
    ///
    /// - `criterion` - Decision tree criteria
    /// - `max_depth` - Maximum depth of the tree
    /// - `min_samples_split` - Minimum samples to split a branch.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_machine::learning::tree::{DecisionTreeClassifier, Metrics};
    ///
    /// let _ = DecisionTreeClassifier::new(Metrics::Gini, 3, 30);
    /// ```
    pub fn new(criterion: Metrics, max_depth: usize, min_samples_split: usize) -> Self {
        DecisionTreeClassifier{ criterion: criterion,
                                max_depth: Some(max_depth),
                                min_samples_split: Some(min_samples_split),
                                n_classes: 0,
                                n_features: 0,
                                root: None }
    }
}

impl DecisionTreeClassifier {

    /// Calculate metrics
    fn metrics_weighted(&self, target: &Vector<usize>) -> f64 {
        self.criterion.from_labels(target) * (target.size() as f64)
        // ToDo: add other metrics
    }

    /// Check termination criteria
    fn can_split(&self, current_target: &Vector<usize>, depth: usize) -> bool {
        match self.max_depth {
            None => {},
            Some(max_depth) => {
                if depth >= max_depth {
                    return false
                }
            }
        }
        match self.min_samples_split {
            None => {},
            Some(min_samples_split) => {
                if current_target.size() <= min_samples_split {
                    return false
                }
            }
        }
        true
    }

    /// Determine whether to split a node
    ///
    /// - `inputs` - Reference to the original data.
    /// - `target` - Reference to the original target.
    /// - `remains` - Index of rows to be considered.
    /// - `depth` - Depth of the node.
    fn split(&self, inputs: &Matrix<f64>, target: &Vector<usize>,
             remains: &Vector<usize>, depth: usize) -> Link {

        let current_target: Vector<usize> = target.select(&remains.data());
        let (labels, counts) = freq(&current_target);

        // stop splitting
        if counts.size() == 1 || !self.can_split(&current_target, depth) {
            let label = labels[counts.argmax().0];
            return Link::Leaf(label)
        }

        let mut split_col: usize = 0;
        let mut split_val: f64 = 0.;

        let mut criteria: f64 = self.metrics_weighted(&current_target);

        // define indexer for reusing after loop
        let mut split_indexer: Vec<bool> = vec![];

        for i in 0..inputs.cols() {
            // target feature
            let current_feature: Vec<f64> = inputs.select(remains.data(), &[i])
                                                  .into_vec();

            for v in get_splits(&current_feature) {
                let bindexer: Vec<bool> = current_feature.iter()
                                                         .map(|&x| x < v)
                                                         .collect();
                let (l, r) = split_slice(&current_target, &bindexer);
                let lc = self.metrics_weighted(&l);
                let rc = self.metrics_weighted(&r);

                let cr = lc + rc;
                // update splitter
                if cr < criteria {
                    split_col = i;
                    split_val = v;
                    criteria = cr;
                    split_indexer = bindexer;
                }
            }
        }
        let (li, ri) = split_slice(remains, &split_indexer);

        let ln = self.split(inputs, target, &li, depth + 1);
        let rn = self.split(inputs, target, &ri, depth + 1);
        Link::Branch(Box::new(Node{ feature_index: split_col,
                                    threshold: split_val,
                                    left: ln,
                                    right: rn }))
    }

    /// Predict a single row
    ///
    /// - `current` - Reference to the root link.
    /// - `row` - Reference to the single row (row slice of the input Matrix).
    fn predict_row(&self, mut current: &Link, row: &[f64]) -> usize {
        loop {
            match current {
                &Link::Leaf(label) => return label,
                &Link::Branch(ref n) => {
                    if row[n.feature_index] < n.threshold {
                        current = &n.left
                    } else {
                        current = &n.right
                    }
                }
            };
        }
    }
}


/// Train the model and predict the model output from new data.
impl SupModel<Matrix<f64>, Vector<usize>> for DecisionTreeClassifier {

    fn predict(&self, inputs: &Matrix<f64>) -> LearningResult<Vector<usize>> {
        match self.root {
            None => Err(Error::new_untrained()),
            Some(ref root) => {
                if self.n_features != inputs.cols() {
                    Err(Error::new(ErrorKind::InvalidData,
                                   "Input data do not have the same dimensions as training data"))
                } else {

                    let results: Vec<usize> = inputs.iter_rows()
                                                    .map(|x| self.predict_row(root, x))
                                                    .collect();
                    Ok(Vector::new(results))
                }
            }
        }
    }

    fn train(&mut self, data: &Matrix<f64>, target: &Vector<usize>) -> LearningResult<()> {
        // set feature and target params
        let (uniques, _) = freq(target);
        self.n_classes = uniques.size();
        self.n_features = data.cols();

        let all: Vec<usize> = (0..target.size()).collect();
        let root = self.split(data, target, &Vector::new(all), 0);
        self.root = Some(root);
        Ok(())
    }
}


/// Uniquify Vec<f64>, result is sorted
fn uniquify(values: &Vec<f64>) -> Vec<f64> {
    let mut values = values.clone();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values.dedup();
    values
}

/// Uniquify values, then get splitter values, i.e. midpoints of unique values
fn get_splits(values: &Vec<f64>) -> Vec<f64> {
    let uniques = uniquify(values);
    uniques[..uniques.len()].iter()
                            .zip(uniques[1..].iter())
                            .map(|(&x, &y)| (x + y) / 2.)
                            .collect()
}

/// Split Vec to left and right, depending on given bool Vec values
fn split_slice<T: Copy>(values: &Vector<T>, bindexer: &Vec<bool>) -> (Vector<T>, Vector<T>) {
    let mut left: Vec<T> = Vec::with_capacity(values.size());
    let mut right: Vec<T> = Vec::with_capacity(values.size());
    for (&v, &flag) in values.iter().zip(bindexer.iter()) {
        if flag {
            left.push(v);
        } else {
            right.push(v);
        }
    }
    left.shrink_to_fit();
    right.shrink_to_fit();
    (Vector::new(left), Vector::new(right))
}

fn xlogy(x: f64, y: f64) -> f64 {
    if x == 0. {
        0.
    } else {
        x * y.ln()
    }
}

/// Count target label frequencies
fn freq(labels: &Vector<usize>) -> (Vector<usize>, Vector<usize>) {
    let mut map: BTreeMap<usize, usize> = BTreeMap::new();
    for l in labels {
        let e = map.entry(*l).or_insert(0);
        *e += 1;
    }

    let mut uniques: Vec<usize> = Vec::with_capacity(map.len());
    let mut counts: Vec<usize> = Vec::with_capacity(map.len());
    for (&k, &v) in map.iter() {
        uniques.push(k);
        counts.push(v);
    }
    (Vector::new(uniques), Vector::new(counts))
}

/// Split criterias
#[derive(Debug)]
pub enum Metrics {
    /// Gini impurity
    Gini,
    /// Information gain
    Entropy
}

impl Metrics {

    /// calculate metrics from target labels
    pub fn from_labels(&self, labels: &Vector<usize>) -> f64 {
        let (_, counts) = freq(labels);
        let sum: f64 = labels.size() as f64;
        let probas: Vec<f64> = counts.iter().map(|&x| x as f64 / sum).collect();
        self.from_probas(&probas)
    }

    /// calculate metrics from label probabilities
    pub fn from_probas(&self, probas: &Vec<f64>) -> f64 {
      match self {
          &Metrics::Entropy => {
              let res: f64 = probas.iter().map(|&x| xlogy(x, x)).sum();
              - res
          },
          &Metrics::Gini => {
              let res: f64 =  probas.iter().map(|&x| x * x).sum();
              1.0 - res
          }
      }
    }
}

#[cfg(test)]
mod tests {

    use linalg::Vector;

    use super::{uniquify, get_splits, split_slice, xlogy, freq, Metrics};

    #[test]
    fn test_uniquify() {
        assert_eq!(uniquify(&vec![0.1, 0.2, 0.1]), vec![0.1, 0.2]);
        assert_eq!(uniquify(&vec![0.3, 0.1, 0.1, 0.1, 0.2, 0.2]), vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_get_splits() {
        assert_eq!(get_splits(&vec![0.1, 0.2, 0.1]), vec![0.15000000000000002]);
        assert_eq!(get_splits(&vec![0.3, 0.1, 0.1, 0.1, 0.2, 0.2]), vec![0.15000000000000002, 0.25]);
        assert_eq!(get_splits(&vec![1., 3., 7., 3., 7.]), vec![2., 5.]);
    }

    #[test]
    fn test_split_slice() {
        let (l, r) = split_slice(&Vector::new(vec![1, 2, 3]), &vec![true, false, true]);
        assert_eq!(l, Vector::new(vec![1, 3]));
        assert_eq!(r, Vector::new(vec![2]));

        let (l, r) = split_slice(&Vector::new(vec![1, 2, 3]), &vec![true, true, true]);
        assert_eq!(l, Vector::new(vec![1, 2, 3]));
        assert_eq!(r, Vector::new(vec![]));
    }

    #[test]
    fn test_xlogy() {
        assert_eq!(xlogy(3., 8.), 6.2383246250395068);
        assert_eq!(xlogy(0., 100.), 0.);
    }

    #[test]
    fn test_freq() {
        let (uniques, counts) = freq(&Vector::new(vec![1, 2, 3, 1, 2, 4]));
        assert_eq!(uniques, Vector::new(vec![1, 2, 3, 4]));
        assert_eq!(counts, Vector::new(vec![2, 2, 1, 1]));

        let (uniques, counts) = freq(&Vector::new(vec![1, 2, 2, 2, 2]));
        assert_eq!(uniques, Vector::new(vec![1, 2]));
        assert_eq!(counts, Vector::new(vec![1, 4]));
    }

    #[test]
    fn test_entropy() {
        assert_eq!(Metrics::Entropy.from_probas(&vec![1.]), 0.);
        assert_eq!(Metrics::Entropy.from_probas(&vec![1., 0., 0.]), 0.);
        assert_eq!(Metrics::Entropy.from_probas(&vec![0.5, 0.5]), 0.69314718055994529);
        assert_eq!(Metrics::Entropy.from_probas(&vec![1. / 3., 1. / 3., 1. / 3.]), 1.0986122886681096);
        assert_eq!(Metrics::Entropy.from_probas(&vec![0.4, 0.3, 0.3]), 1.0888999753452238);
    }

    #[test]
    fn test_gini_from_probas() {
        assert_eq!(Metrics::Gini.from_probas(&vec![1., 0., 0.]), 0.);
        assert_eq!(Metrics::Gini.from_probas(&vec![1. / 3., 1. / 3., 1. / 3.]), 0.6666666666666667);
        assert_eq!(Metrics::Gini.from_probas(&vec![0., 1. / 46., 45. / 46.]), 0.04253308128544431);
        assert_eq!(Metrics::Gini.from_probas(&vec![0., 49. / 54., 5. / 54.]), 0.16803840877914955);
    }

    #[test]
    fn test_entropy_from_labels() {
        assert_eq!(Metrics::Entropy.from_labels(&Vector::new(vec![1, 2, 3])), 1.0986122886681096);
        assert_eq!(Metrics::Entropy.from_labels(&Vector::new(vec![1, 1, 2, 2])), 0.69314718055994529);
    }

    #[test]
    fn test_gini_from_labels() {
        assert_eq!(Metrics::Gini.from_labels(&Vector::new(vec![1, 1, 1])), 0.);
        assert_eq!(Metrics::Gini.from_labels(&Vector::new(vec![1, 1, 2, 2, 3, 3])), 0.6666666666666667);
    }
}