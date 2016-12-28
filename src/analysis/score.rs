//! Functions for scoring a set of predictions, i.e. evaluating
//! how close predictions and truth are. All functions in this
//! module obey the convention that higher is better.

use libnum::{Zero, One};

use linalg::{BaseMatrix, Matrix};
use learning::toolkit::cost_fn::{CostFunc, MeanSqError};

// ************************************
// Classification Scores
// ************************************

/// Returns the fraction of outputs which match their target.
pub fn accuracy<I>(outputs: I, targets: I) -> f64
    where I: ExactSizeIterator,
          I::Item: PartialEq
{
    assert!(outputs.len() == targets.len());
    let len = outputs.len() as f64;
    let correct = outputs
        .zip(targets)
        .filter(|&(ref x, ref y)| x == y)
        .count();
    correct as f64 / len
}

/// Returns the fraction of outputs rows which match their target.
pub fn row_accuracy(outputs: &Matrix<f64>, targets: &Matrix<f64>) -> f64 {
    accuracy(outputs.iter_rows(), targets.iter_rows())
}

/// Returns the precision score for 2 class classification.
/// true-positive / (true-positive + false-positive)
pub fn precision<'a, I, T>(outputs: I, targets: I) -> f64
    where I: ExactSizeIterator<Item=&'a T>,
          T: 'a + PartialEq + Zero + One
{
    assert!(outputs.len() == targets.len());

    let mut tpfp = 0.0f64;
    let mut tp = 0.0f64;

    for (ref o, ref t) in outputs.zip(targets) {
        if *o == &T::one() {
            tpfp += 1.0f64;
            if *t == &T::one() {
                tp += 1.0f64;
            }
        }
        if ((*t != &T::zero()) & (*t != &T::one())) |
           ((*o != &T::zero()) & (*o != &T::one())) {
            panic!("precision must be used for 2 class classification")
        }
    }
    tp / tpfp
}

/// Returns the recall score for 2 class classification.
/// true-positive / (true-positive + false-negative)
pub fn recall<'a, I, T>(outputs: I, targets: I) -> f64
    where I: ExactSizeIterator<Item=&'a T>,
          T: 'a + PartialEq + Zero + One
{
    assert!(outputs.len() == targets.len());

    let mut tpfn = 0.0f64;
    let mut tp = 0.0f64;

    for (ref o, ref t) in outputs.zip(targets) {
        if *t == &T::one() {
            tpfn += 1.0f64;
            if *o == &T::one() {
                tp += 1.0f64;
            }
        }
        if ((*t != &T::zero()) & (*t != &T::one())) |
           ((*o != &T::zero()) & (*o != &T::one())) {
            panic!("recall must be used for 2 class classification")
        }
    }
    tp / tpfn
}

/// Returns the f1 score for 2 class classification
/// 2 * precision * recall / (precision + recall)
pub fn f1<'a, I, T>(outputs: I, targets: I) -> f64
    where I: ExactSizeIterator<Item=&'a T>,
          T: 'a + PartialEq + Zero + One
{
    assert!(outputs.len() == targets.len());

    let mut tpos = 0.0f64;
    let mut fpos = 0.0f64;
    let mut fneg = 0.0f64;

    for (ref o, ref t) in outputs.zip(targets) {
        if (*o == &T::one()) & (*t == &T::one()) {
            tpos += 1.0f64;
        } else if *t == &T::one() {
            fpos += 1.0f64;
        } else if *o == &T::one() {
            fneg += 1.0f64;
        }
        if ((*t != &T::zero()) & (*t != &T::one())) |
           ((*o != &T::zero()) & (*o != &T::one())) {
            panic!("f1-score must be used for 2 class classification")
        }
    }
    // precision
    let p = tpos / (tpos + fpos);
    // recall
    let r = tpos / (tpos + fneg);
    2.0 * p * r / (p + r)
}

// ************************************
// Regression Scores
// ************************************

// TODO: generalise to accept arbitrary iterators of diff-able things
/// Returns the additive inverse of the mean-squared-error of the
/// outputs. So higher is better, and the returned value is always
/// negative.
pub fn neg_mean_squared_error(outputs: &Matrix<f64>, targets: &Matrix<f64>) -> f64
{
    // MeanSqError divides the actual mean squared error by two.
    -2f64 * MeanSqError::cost(outputs, targets)
}

#[cfg(test)]
mod tests {
    use linalg::Matrix;
    use super::{accuracy, precision, recall, f1, neg_mean_squared_error};

    #[test]
    fn test_accuracy() {
        let outputs = [1, 2, 3, 4, 5, 6];
        let targets = [1, 2, 3, 3, 5, 1];
        assert_eq!(accuracy(outputs.iter(), targets.iter()), 2f64/3f64);

        let outputs = [1, 1, 1, 0, 0, 0];
        let targets = [1, 1, 1, 0, 0, 1];
        assert_eq!(accuracy(outputs.iter(), targets.iter()), 5.0f64 / 6.0f64);
    }

    #[test]
    fn test_precision() {
        let outputs = [1, 1, 1, 0, 0, 0];
        let targets = [1, 1, 0, 0, 1, 1];
        assert_eq!(precision(outputs.iter(), targets.iter()), 2.0f64 / 3.0f64);

        let outputs = [1, 1, 1, 0, 1, 1];
        let targets = [1, 1, 0, 0, 1, 1];
        assert_eq!(precision(outputs.iter(), targets.iter()), 0.8);

        let outputs = [0, 0, 0, 1, 1, 1];
        let targets = [1, 1, 1, 1, 1, 0];
        assert_eq!(precision(outputs.iter(), targets.iter()), 2.0f64 / 3.0f64);

        let outputs = [1, 1, 1, 1, 1, 0];
        let targets = [0, 0, 0, 1, 1, 1];
        assert_eq!(precision(outputs.iter(), targets.iter()), 0.4);
    }

    #[test]
    #[should_panic]
    fn test_precision_outputs_not_2class() {
        let outputs = [1, 2, 1, 0, 0, 0];
        let targets = [1, 1, 0, 0, 1, 1];
        precision(outputs.iter(), targets.iter());
    }

    #[test]
    #[should_panic]
    fn test_precision_targets_not_2class() {
        let outputs = [1, 0, 1, 0, 0, 0];
        let targets = [1, 2, 0, 0, 1, 1];
        precision(outputs.iter(), targets.iter());
    }

    #[test]
    fn test_recall() {
        let outputs = [1, 1, 1, 0, 0, 0];
        let targets = [1, 1, 0, 0, 1, 1];
        assert_eq!(recall(outputs.iter(), targets.iter()), 0.5);

        let outputs = [1, 1, 1, 0, 1, 1];
        let targets = [1, 1, 0, 0, 1, 1];
        assert_eq!(recall(outputs.iter(), targets.iter()), 1.0);

        let outputs = [0, 0, 0, 1, 1, 1];
        let targets = [1, 1, 1, 1, 1, 0];
        assert_eq!(recall(outputs.iter(), targets.iter()), 0.4);

        let outputs = [1, 1, 1, 1, 1, 0];
        let targets = [0, 0, 0, 1, 1, 1];
        assert_eq!(recall(outputs.iter(), targets.iter()), 2.0f64 / 3.0f64);
    }

    #[test]
    #[should_panic]
    fn test_recall_outputs_not_2class() {
        let outputs = [1, 2, 1, 0, 0, 0];
        let targets = [1, 1, 0, 0, 1, 1];
        recall(outputs.iter(), targets.iter());
    }

    #[test]
    #[should_panic]
    fn test_recall_targets_not_2class() {
        let outputs = [1, 0, 1, 0, 0, 0];
        let targets = [1, 2, 0, 0, 1, 1];
        recall(outputs.iter(), targets.iter());
    }

    #[test]
    fn test_f1() {
        let outputs = [1, 1, 1, 0, 0, 0];
        let targets = [1, 1, 0, 0, 1, 1];
        assert_eq!(f1(outputs.iter(), targets.iter()), 0.57142857142857151);

        let outputs = [1, 1, 1, 0, 1, 1];
        let targets = [1, 1, 0, 0, 1, 1];
        assert_eq!(f1(outputs.iter(), targets.iter()), 0.88888888888888895);

        let outputs = [0, 0, 0, 1, 1, 1];
        let targets = [1, 1, 1, 1, 1, 0];
        assert_eq!(f1(outputs.iter(), targets.iter()), 0.5);

        let outputs = [1, 1, 1, 1, 1, 0];
        let targets = [0, 0, 0, 1, 1, 1];
        assert_eq!(f1(outputs.iter(), targets.iter()), 0.5);
    }


    #[test]
    #[should_panic]
    fn test_f1_outputs_not_2class() {
        let outputs = [1, 2, 1, 0, 0, 0];
        let targets = [1, 1, 0, 0, 1, 1];
        f1(outputs.iter(), targets.iter());
    }

    #[test]
    #[should_panic]
    fn test_f1_targets_not_2class() {
        let outputs = [1, 0, 1, 0, 0, 0];
        let targets = [1, 2, 0, 0, 1, 1];
        f1(outputs.iter(), targets.iter());
    }

    #[test]
    fn test_neg_mean_squared_error_1d() {
        let outputs = Matrix::new(3, 1, vec![1f64, 2f64, 3f64]);
        let targets = Matrix::new(3, 1, vec![2f64, 4f64, 3f64]);
        assert_eq!(neg_mean_squared_error(&outputs, &targets), -5f64/3f64);
    }

    #[test]
    fn test_neg_mean_squared_error_2d() {
        let outputs = Matrix::new(3, 2, vec![
            1f64, 2f64,
            3f64, 4f64,
            5f64, 6f64
            ]);
        let targets = Matrix::new(3, 2, vec![
            1.5f64, 2.5f64,
            5f64,   6f64,
            5.5f64, 6.5f64
            ]);
        assert_eq!(neg_mean_squared_error(&outputs, &targets), -3f64);
    }
}
