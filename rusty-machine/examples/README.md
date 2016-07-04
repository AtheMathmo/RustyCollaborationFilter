Sample Codes with rusty-machine
===============================

This directory gathers fully-fledged programs, each using a piece of
`rusty-machine`'s API.

## Overview

* [SVM](#svm)
* [Neural Networks](#neural-networks)

## The Examples

### SVM

#### Sign Learner

[Sign learner](svm-sign_learner.rs) constructs and evaluates a model that learns to recognize the sign of an input number.

The sample shows a basic usage of the SVM API. It also configures the SVM algorithm with a specific kernel (`HyperTan`). Evaluations are run in a loop to log individual predictions and do some book keeping for reporting the performance at the end. The salient part from `rusty-machine` is to use the `train` and `predict` methods of the SVM model.

The accuracy evaluation is simplistic, so the model manages 100% accuracy (which is *really* too simple an example).

Sample run:

```
cargo run --example svm-sign_learner
   Compiling rusty-machine v0.3.0 (file:///rusty-machine/rusty-machine)
     Running `target/debug/examples/svm-sign_learner`
Sign learner sample:
Training...
Evaluation...
-1000 -> -1: true
-900 -> -1: true
-800 -> -1: true
-700 -> -1: true
-600 -> -1: true
-500 -> -1: true
-400 -> -1: true
-300 -> -1: true
-200 -> -1: true
-100 -> -1: true
0 -> -1: true
100 -> 1: true
200 -> 1: true
300 -> 1: true
400 -> 1: true
500 -> 1: true
600 -> 1: true
700 -> 1: true
800 -> 1: true
900 -> 1: true
Performance report:
Hits: 20, Misses: 0
Accuracy: 100
```

### Neural Networks

#### AND Gate

[AND gate](nnet-and_gate.rs) makes an AND gate out of a perceptron.

The sample code generates random data to learn from. The input data is like an electric signal between 0 and 1, with some jitter that makes it not quite 0 or 1. By default, the code decides that any pair input "above" (0.7, 0.7) is labeled as 1.0 (AND gate passing), otherwise labeled as 0.0 (AND gate blocking). This means that the training set is biased toward learning the passing scenario: An AND gate passes 25% of the time on average, and we'd like it to learn it.

The test data uses only the 4 "perfect" inputs for a gate: (0.0, 0.0), (1.0, 0.0), etc.

The code generates 10,000 training data points by default. Please give it a try, and then change `SAMPLE`, the number of training data points, and `THRESHOLD`, the value for "deciding" for a passing gate.

Sample run:

```
> cargo run --example nnet-and_gate
   Compiling rusty-machine v0.3.0 (file:///rusty-machine/rusty-machine)
     Running `target/debug/examples/nnet-and_gate`
AND gate learner sample:
Generating 10000 training data and labels...
Training...
Evaluation...
Got  Expected
0.00  0
0.00  0
0.96  1
0.01  0
Hits: 4, Misses: 0
Accuracy: 100%
```

