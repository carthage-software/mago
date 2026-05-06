<?php

enum Category: string
{
    case Science = 'science';
    case Art = 'art';
    case History = 'history';
}

function use_category(Category $category): void
{
    echo
        match ($category) {
            Category::Science => 'You selected Science',
            Category::Art => 'You selected Art',
            Category::History => 'You selected History',
        }
    ;
}

/**
 * @template I
 * @template O
 *
 * @param I $value
 * @param (callable(I): O) $transformer
 *
 * @return O
 */
function transform_value(mixed $value, callable $transformer): mixed
{
    return $transformer($value);
}

$result = transform_value('science', Category::from(...));
$result2 = transform_value('science', Category::tryFrom(...));

use_category($result);
if ($result2 !== null) {
    use_category($result2);
}
