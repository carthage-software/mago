<?php

declare(strict_types=1);

$t = ['some' => 'thing'];
// @mago-expect analysis:impossible-nonnull-entry-check
// @mago-expect analysis:impossible-condition
if (!empty($t['info'])) {
    echo "got some info\n";
}

$t = ['some' => 'other'];
// @mago-expect analysis:impossible-nonnull-entry-check
// @mago-expect analysis:impossible-condition
if (isset($t['info'])) {
    echo "got some info set\n";
}

$t = ['some' => 'third'];
// @mago-expect analysis:impossible-condition
if (array_key_exists('info', $t)) {
    echo "got some info set\n";
}
