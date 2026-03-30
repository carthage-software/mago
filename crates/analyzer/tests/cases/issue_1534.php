<?php

declare(strict_types=1);

function object_tabpreview(array $opts = []): void
{
    $callback = '';
    if (!empty($opts['post_content_callback'])) {
        /** @mago-expect analysis:mixed-assignment */
        $callback = $opts['post_content_callback'];
    }

    if ($callback) {
        echo 'some callback';
        return;
    }

    echo 'no callback';
}
