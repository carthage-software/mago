<?php

namespace {
    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x1(array $arr): array
    {
        if (array_key_exists('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}

namespace {
    use function array_key_exists;

    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x2(array $arr): array
    {
        if (array_key_exists('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}

namespace {
    use function array_key_exists as has_key;

    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x3(array $arr): array
    {
        if (has_key('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}

namespace X {
    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x4(array $arr): array
    {
        if (array_key_exists('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}

namespace X {
    use function array_key_exists;

    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x5(array $arr): array
    {
        if (array_key_exists('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}

namespace X {
    use function array_key_exists as has_key;

    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x6(array $arr): array
    {
        if (has_key('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}

namespace X {
    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x7(array $arr): array
    {
        if (\array_key_exists('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}

namespace X {
    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x8(array $arr): array
    {
        if (\array_key_exists('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}

namespace X {
    /**
     * @param array<string> $arr
     *
     * @return array{x: string, ...<array-key, string>}
     */
    function with_x9(array $arr): array
    {
        if (\array_key_exists('x', $arr)) {
            return $arr;
        }

        exit('x is not there.');
    }
}
