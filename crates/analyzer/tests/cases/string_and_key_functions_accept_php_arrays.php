<?php

declare(strict_types=1);

/**
 * @param list<int> $numbers
 * @param array<array-key, string> $headers
 * @param array<array-key, mixed> $variables
 */
function string_and_key_functions_accept_php_arrays(
    array $numbers,
    array $headers,
    array $variables,
    ?string $email,
): string {
    $text = str_replace([1, 'b'], ['x', 2], 'a1b');
    $text .= str_ireplace(['a'], [null], 'A');
    $text .= strtr('%n% %e%', ['%n%' => 1, '%e%' => $email]);

    $matching = preg_grep('/^1/', $numbers);
    if ($matching !== false) {
        $text .= (string) array_sum($matching);
    }

    $lowercase = array_change_key_case($headers);
    foreach ($lowercase as $value) {
        $text .= $value;
    }

    return $text . extract($variables);
}
