<?php

function x()
{
    $a = <<<SQL
            SELECT * FROM {$table} t
            WHERE t.id = {$id}
        SQL;

    $b = <<<SQL
            UPDATE amount AS a
                SET posted_at = t.posted_at
            FROM entry e, {$table} t
                WHERE e.id = a.entry_id AND e.id = t.entry_id;
        SQL;

    $a = <<<SQL
            SELECT * FROM {$table} t
            WHERE t.id = {$id}
        SQL;

    $b = <<<SQL
            UPDATE amount AS a
                SET posted_at = t.posted_at
            FROM entry e, {$table} t
                WHERE e.id = a.entry_id AND e.id = t.entry_id;
        SQL;
}
