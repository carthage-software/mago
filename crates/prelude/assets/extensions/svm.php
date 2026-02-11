<?php

class SVM
{
    public const C_SVC = 0;
    public const NU_SVC = 1;
    public const ONE_CLASS = 2;
    public const EPSILON_SVR = 3;
    public const NU_SVR = 4;
    public const KERNEL_LINEAR = 0;
    public const KERNEL_POLY = 1;
    public const KERNEL_RBF = 2;
    public const KERNEL_SIGMOID = 3;
    public const KERNEL_PRECOMPUTED = 4;
    public const OPT_TYPE = 101;
    public const OPT_KERNEL_TYPE = 102;
    public const OPT_DEGREE = 103;
    public const OPT_SHRINKING = 104;
    public const OPT_PROPABILITY = 105;
    public const OPT_GAMMA = 201;
    public const OPT_NU = 202;
    public const OPT_EPS = 203;
    public const OPT_P = 204;
    public const OPT_COEF_ZERO = 205;
    public const OPT_C = 206;
    public const OPT_CACHE_SIZE = 207;

    public function __construct() {}

    public function crossvalidate(array $problem, int $number_of_folds): float {}

    public function getOptions(): array {}

    public function setOptions(array $params): bool {}

    public function train(array $problem, array $weights = null): SVMModel {}
}

class SVMModel
{
    public function checkProbabilityModel(): bool {}

    public function __construct(string $filename = '') {}

    public function getLabels(): array {}

    public function getNrClass(): int {}

    public function getSvmType(): int {}

    public function getSvrProbability(): float {}

    public function load(string $filename): bool {}

    public function predict_probability(array $data): float {}

    public function predict(array $data): float {}

    public function save(string $filename): bool {}
}
