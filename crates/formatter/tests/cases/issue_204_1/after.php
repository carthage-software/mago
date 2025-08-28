<?php

function a()
{
    $promises = (
        /**
         * @return \Generator
         * @return \Generator
         */
        static function () use ($paginator): \Generator {
            yield $paginator->getCurrentPageResultsAsync();

            while ($paginator->hasNextPage()) {
                $paginator->nextPage();

                yield $paginator->getCurrentPageResultsAsync();
            }
        }
    )();
}
