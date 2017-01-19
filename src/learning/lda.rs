//! Latent Diriclhet Allocation Module
//!
//! Contains an implementation of Latent Diriclhet Allocation (LDA) using
//! [Gibbs Sampling](http://psiexp.ss.uci.edu/research/papers/sciencetopics.pdf)
//!
//! LDA is typically used for textual analysis.  It assumes that a topic can be modeled
//! as a distribution of words, and that a document can be modeled as a distribution of
//! topics.  Thus, the output is these distributions on the input documents.
//!
//! Gibbs sampling is a Morkov Chain Monto Carlo algorithm that iteratively approximates
//! the above distributions.
//!
//! This module doesn't use any training.  It uses unsupervised learning to estimate
//! the categories for a collection of documents.
//!
//! # Examples
//!
//! ```
//! use rusty_machine::linalg::{Matrix, BaseMatrix, Vector};
//! use rusty_machine::learning::UnSupModel;
//! use rusty_machine::learning::lda::LDA;
//!
//! // Create a basic input array
//! let input = Matrix::ones(5, 4);
//!
//! // Create a model that will find 5 topics, with parameters alpha=0.1, beta=0.1
//! let lda = LDA::new(5, 0.1, 0.1);
//!
//! // No need to train the model (doing so is a no-op)
//! lda.predict(&(input, 10)).unwrap();
//! ```


use linalg::{Matrix, Vector, BaseMatrix};
use learning::{LearningResult, UnSupModel};
use rulinalg::matrix::BaseMatrixMut;
use rand::{Rng, thread_rng};

use std::collections::HashMap;

/// Latent Dirichlet Allocation
#[derive(Debug)]
pub struct LDA {
    topic_count: usize,
    alpha: f64,
    beta: f64,
}

/// An object which holds the results of Gibbs Sampling.
///
/// This object can then be used to get the distrbutions of
/// topics over documents and words over topics
#[derive(Debug)]
pub struct LDAResult {
    document_topic_count: Matrix<f64>,
    topic_word_count: Matrix<f64>,
    topic_total_by_document: Vector<f64>,
    word_total_by_topic: Vector<f64>,
    topics: HashMap<(usize, usize), usize>,
    alpha: f64,
    beta: f64,
}

impl Default for LDA {
    fn default() -> LDA {
        LDA {
            topic_count: 10,
            alpha: 0.1,
            beta: 0.1
        }
    }
}

impl LDA {
    /// Creates a new object for finding LDA.
    /// alpha and beta are the symmetric priors for the algorithm.
    ///
    /// If you don't know what to use, try alpha = 50/topic_count and beta = 0.01
    pub fn new(topic_count: usize, alpha: f64, beta: f64) -> LDA {
        LDA {
            topic_count: topic_count,
            alpha: alpha,
            beta: beta
        }
    }
}

impl LDAResult {
    fn new(input: &Matrix<usize>, topic_count: usize, alpha: f64, beta: f64) -> LDAResult {
        let document_count = input.rows();
        let vocab_count = input.cols();
        let mut word_index:usize;
        let mut result = LDAResult {
            document_topic_count: Matrix::zeros(document_count, topic_count),
            topic_word_count: Matrix::zeros(topic_count, vocab_count),
            topic_total_by_document: Vector::zeros(document_count),
            word_total_by_topic: Vector::zeros(topic_count,),
            topics: HashMap::with_capacity(topic_count),
            alpha: alpha,
            beta: beta
        };
        let mut rng = thread_rng();
        for (document, row) in input.row_iter().enumerate() {
            word_index = 0;
            for (word, word_count) in row.iter().enumerate() {
                for _ in 0..*word_count{
                    let topic = rng.gen_range(0, topic_count);
                    result.document_topic_count[[document, topic]] += 1.0;
                    result.topic_total_by_document[document] += 1.0;
                    result.topic_word_count[[topic, word]] += 1.0;
                    result.word_total_by_topic[topic] += 1.0;
                    result.topics.insert((document, word_index),  topic);
                    word_index += 1;
                }
            }
        }
        result
    }

    /// Find the distribution of words over topics.  This gives a matrix where the rows are
    /// topics and the columns are words.  Each entry (topic, word) gives the probability of
    /// word given topic.
    pub fn phi(&self) -> Matrix<f64> {
        let mut distribution = self.topic_word_count.clone() + self.beta;
        let row_sum = distribution.sum_rows();
        for (mut row, sum) in distribution.row_iter_mut().zip(row_sum.iter()) {
            *row /= sum;
        }
        distribution
    }
}

impl LDA {
    fn conditional_distribution(&self, result: &LDAResult, document: usize, word: usize) -> Vector<f64> {
        let vocab_count = result.topic_word_count.cols();

        // Convert the column of the word count by topic into a vector
        let word_topic_count:Vector<f64> = result.topic_word_count.col(word).into();

        let left:Vector<f64> = (word_topic_count + self.beta).elediv(
            &(result.word_total_by_topic.clone() + self.beta * vocab_count as f64)
        );

        // Convert the row of the topic count by document into a vector
        let topic_document_count:Vector<f64> = result.document_topic_count.row(document).into();
        
        let right:Vector<f64> =  (topic_document_count + self.alpha) /
            (result.topic_total_by_document[document] + self.alpha * self.topic_count as f64);

        let mut probability:Vector<f64> = left.elemul(&right);
        probability /= probability.sum();
        return probability;
    }
}

impl UnSupModel<(Matrix<usize>, usize), LDAResult> for LDA {
    /// Predict categories from the input matrix.
        fn predict(&self, inputs: &(Matrix<usize>, usize)) -> LearningResult<LDAResult> {
            let ref matrix = inputs.0;
            let mut result = LDAResult::new(&matrix, self.topic_count, self.alpha, self.beta);
            let mut word_index:usize;
            for _ in 0..inputs.1 {
                for (document, row) in matrix.row_iter().enumerate() {
                    word_index = 0;
                    for (word, word_count) in row.iter().enumerate() {
                        for _ in 0..*word_count {

                            let mut topic = *result.topics.get(&(document, word_index)).unwrap();
                            //println!("document: {}, word: {}, old topic: {}", document, word, topic);
                            result.document_topic_count[[document, topic]] -= 1.0;
                            result.topic_total_by_document[document] -= 1.0;
                            result.topic_word_count[[topic, word]] -= 1.0;
                            result.word_total_by_topic[topic] -= 1.0;

                            let probability = self.conditional_distribution(&result, document, word);

                            topic = choose_from(probability);
                            //println!("document: {}, word: {}, new topic: {}", document, word, topic);
                            result.document_topic_count[[document, topic]] += 1.0;
                            result.topic_total_by_document[document] += 1.0;
                            result.topic_word_count[[topic, word]] += 1.0;
                            result.word_total_by_topic[topic] += 1.0;
                            result.topics.insert((document, word_index), topic);
                            word_index += 1;
                        }
                    }
                }
            }
            Ok(result)
        }

        /// Train the model using inputs  Does nothing on this model.
        fn train(&mut self, _: &(Matrix<usize>, usize)) -> LearningResult<()> {
            Ok(())
        }
}

/// This function models sampling from the categorical distribution.
/// That is given a series of discrete categories, each with different probability of occurring,
/// this function will choose a category according to their probabilities.
/// The sum of the probabilities must be 1, but since this is only used internally,
/// there is no need to verify that this is true.
fn choose_from(probability: Vector<f64>) -> usize {
    let mut rng = thread_rng();
    let selection:f64 = rng.gen_range(0.0, 1.0);
    let mut total:f64 = 0.0;
    for (index, p) in probability.iter().enumerate() {
        total += *p;
        if total >= selection {
            return index;
        }
    }
    return probability.size() - 1;
}

#[cfg(test)]
mod test {
    use super::{LDAResult, LDA};
    use linalg::{Matrix, Vector};
    use std::collections::HashMap;
    #[test]
    fn test_conditional_distribution() {
        let result = LDAResult {
            topic_word_count: Matrix::new(3, 4, vec![3.0, 3.0, 5.0, 0.0,
                                                     0.0, 1.0, 1.0, 0.0,
                                                     0.0, 0.0, 5.0, 2.0]),
            word_total_by_topic: Vector::new(vec![11.0, 2.0, 7.0]),
            document_topic_count: Matrix::new(2, 3, vec![1.0, 2.0, 3.0,
                                                         5.0, 5.0, 4.0]),
            topic_total_by_document: Vector::new(vec![6.0, 14.0]),
            alpha: 0.1,
            beta: 0.1,
            topics: HashMap::new()

        };
        let lda = LDA::new(3, 0.1, 0.1);
        let probability = lda.conditional_distribution(& result, 0, 2);
        assert_eq!(probability,
            Vector::new(vec![0.13703500146066358, 0.2680243410921803, 0.5949406574471561]));
    }
}
