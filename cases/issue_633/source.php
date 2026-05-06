<?php

/**
 * @param array{0: array{}} $array
 * @return array{}
 */
function i_accept_empty_arrays(array $array): array
{
    return $array[0];
}

$array = array_merge([[]], []);
$array = i_accept_empty_arrays($array);

i_accept_empty_arrays([0 => $array]);
