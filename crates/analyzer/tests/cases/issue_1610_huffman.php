<?php

/**
 * @param array<string, int> $trie
 *
 * @return array{list<list<array{int, int}>>, list<bool>}
 */
function build_nibble_table(array $trie): array
{
    $stateMap = [];
    $statePaths = [];
    $states = [];
    $stateId = 0;

    $stateMap[''] = $stateId++;
    $statePaths[] = '';
    $states[] = [];

    $queue = [''];

    while ($queue !== []) {
        $bitPath = array_shift($queue);
        $currentStateId = $stateMap[$bitPath];

        $transitions = [];
        for ($nibble = 0; $nibble < 16; $nibble++) {
            $path = $bitPath;
            $emit = -1;

            for ($bit = 3; $bit >= 0; $bit--) {
                $path .= (string) (($nibble >> $bit) & 1);

                if (isset($trie[$path]) && $trie[$path] >= 0) {
                    $sym = $trie[$path];
                    if ($sym === 256) {
                        $transitions[] = [-1, 256];
                        continue 2;
                    }

                    $emit = $sym;
                    $path = '';
                }
            }

            if (!isset($trie[$path])) {
                $transitions[] = [-1, -1];
                continue;
            }

            if (!isset($stateMap[$path])) {
                $stateMap[$path] = $stateId++;
                $statePaths[] = $path;
                $states[] = [];
                $queue[] = $path;
            }

            $transitions[] = [$stateMap[$path], $emit];
        }

        $states[$currentStateId] = $transitions;
    }

    $acceptFlags = [];
    foreach ($statePaths as $path) {
        $acceptFlags[] = $path === '' || strlen($path) <= 7 && !str_contains($path, '0');
    }

    return [$states, $acceptFlags];
}
