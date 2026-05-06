<?php

namespace {
    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled1(array $arr): array
    {
        if (count($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}

namespace {
    use function count;

    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled2(array $arr): array
    {
        if (count($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}

namespace {
    use function count as count_values;

    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled3(array $arr): array
    {
        if (count_values($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}

namespace X {
    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled4(array $arr): array
    {
        if (count($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}

namespace X {
    use function count;

    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled5(array $arr): array
    {
        if (count($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}

namespace X {
    use function count as count_values;

    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled6(array $arr): array
    {
        if (count_values($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}

namespace X {
    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled7(array $arr): array
    {
        if (\count($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}

namespace X {
    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled8(array $arr): array
    {
        if (\count($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}

namespace X {
    /**
     * @param array<int, string> $arr
     *
     * @return non-empty-array<int, string>
     */
    function filled9(array $arr): array
    {
        if (\count($arr) !== 0) {
            return $arr;
        }

        exit('Array is empty');
    }
}
