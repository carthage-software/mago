<?php

namespace pcov {
    /**
     * @return void
     */
    function start() {}

    /**
     * @return void
     */
    function stop() {}

    /**
     * @return array
     */
    function collect(int $type = all, array $filter = []) {}

    /**
     * @return void
     */
    function clear(bool $files = false) {}

    /**
     * @return array
     */
    function waiting() {}

    /**
     * @return int
     */
    function memory() {}

    const all = 0;

    const inclusive = 1;

    const exclusive = 2;

    const version = '1.0.6';
}
